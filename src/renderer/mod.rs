pub mod buffer;
pub mod context;
pub mod draw_command;
pub mod light;
pub mod material;
pub mod pipeline;
pub mod post_process;
pub mod shader;
pub mod texture;

use self::draw_command::DrawCommand;
use self::light::SceneLights;
use self::material::{Material, MaterialId};
use crate::camera::Camera;
use crate::mesh::{Instance, Mesh, MeshType};
use glow::{Context, HasContext};
use glutin::context::PossiblyCurrentContext;
use glutin::prelude::GlSurface;
use glutin::surface::{Surface, WindowSurface};
use std::collections::HashMap;
use std::num::NonZeroU32;
use texture::{Texture, TextureId};
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
#[allow(dead_code)]
pub struct MeshBatch {
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
    instance_buffer: glow::Buffer,
    indices_count: i32,
}

// GL State Cache
/// Tracks the currently bound OpenGL objects to avoid redundant
/// state changes (state thrashing).
struct GlStateCache {
    current_vao: Option<glow::VertexArray>,
    current_program: Option<glow::Program>,
}

impl GlStateCache {
    fn new() -> Self {
        Self {
            current_vao: None,
            current_program: None,
        }
    }

    /// Bind VAO only if it differs from the currently bound one.
    unsafe fn bind_vao(&mut self, gl: &Context, vao: glow::VertexArray) {
        if self.current_vao != Some(vao) {
            unsafe {
                gl.bind_vertex_array(Some(vao));
            }
            self.current_vao = Some(vao);
        }
    }

    /// Use program only if it differs from the currently active one.
    unsafe fn use_program(&mut self, gl: &Context, program: glow::Program) {
        if self.current_program != Some(program) {
            unsafe {
                gl.use_program(Some(program));
            }
            self.current_program = Some(program);
        }
    }

    /// Invalidate all cached state (e.g. after post-processing pass).
    fn invalidate(&mut self) {
        self.current_vao = None;
        self.current_program = None;
    }
}

pub struct GraphicsContext {
    pub gl_surface: Surface<WindowSurface>,
    pub gl_context: PossiblyCurrentContext,
    pub window: Window,
}

pub struct RenderState {
    gl: Context,
    ctx: GraphicsContext,
    program: glow::Program,
    pub camera: Camera,

    batches: HashMap<MeshType, MeshBatch>,
    materials: HashMap<MaterialId, Material>,

    // Texture handle system: path → id for dedup, id → GPU texture for runtime
    textures: Vec<Texture>,
    texture_path_index: HashMap<String, TextureId>,

    state_cache: GlStateCache,
    pub post_process: post_process::PostProcessManager,
}

impl RenderState {
    pub fn new(
        event_loop: &ActiveEventLoop,
        title: &str,
        width: i32,
        height: i32,
        vert_src: &str,
        frag_src: &str,
    ) -> Self {
        let (gl, gl_surface, gl_context, window) =
            context::init_context(event_loop, title, width, height);

        let program = unsafe {
            let program = gl.create_program().unwrap();
            shader::create_shaders(&gl, program, vert_src, frag_src);
            gl.use_program(Some(program));
            program
        };

        let physical_size = window.inner_size();
        let post_process = post_process::PostProcessManager::new(
            &gl,
            physical_size.width as i32,
            physical_size.height as i32,
        );

        let mut state = Self {
            gl,
            ctx: GraphicsContext {
                gl_surface,
                gl_context,
                window,
            },
            program,
            camera: Camera::new_perspective(45.0, (width as f32) / (height as f32), 0.1, 1000.0),
            batches: HashMap::new(),
            materials: HashMap::new(),
            textures: Vec::new(),
            texture_path_index: HashMap::new(),
            state_cache: GlStateCache::new(),
            post_process,
        };
        // RH: camera at +Z looking toward -Z (origin)
        state.camera.set_position(glam::vec3(0.0, 0.0, 10.0));
        state.camera.update();

        // Pre-register basic meshes
        state.register_mesh(MeshType::Square, &Mesh::square());
        state.register_mesh(MeshType::Triangle, &Mesh::triangle());
        state.register_mesh(MeshType::Cube, &Mesh::cube());

        // Register default material
        state.register_material(
            MaterialId(0),
            Material::new("default", glam::Vec4::ONE, None),
        );

        state
    }

    /// Load a texture by path (deduplicated) and return its handle.
    fn load_texture(&mut self, path: &str) -> TextureId {
        if let Some(&id) = self.texture_path_index.get(path) {
            return id;
        }
        let texture = Texture::load(&self.gl, path);
        let id = TextureId(self.textures.len());
        self.textures.push(texture);
        self.texture_path_index.insert(path.to_string(), id);
        id
    }

    pub fn register_material(&mut self, id: MaterialId, mut material: Material) {
        if let Some(path) = material.texture_path.take() {
            let tex_id = self.load_texture(&path);
            material.albedo_texture = Some(tex_id);
        }
        self.materials.insert(id, material);
    }

    fn register_mesh(&mut self, mesh_type: MeshType, mesh: &Mesh) {
        let (vao, vbo, ebo) = buffer::setup_mesh_buffers(&self.gl, mesh);
        unsafe {
            self.gl.bind_vertex_array(Some(vao));

            pipeline::setup_pipeline(&self.gl);
            let instance_buffer = buffer::setup_instance_buffer(&self.gl);
            pipeline::setup_instancing(&self.gl);

            self.batches.insert(
                mesh_type,
                MeshBatch {
                    vao,
                    vbo,
                    ebo,
                    instance_buffer,
                    indices_count: mesh.indices.len() as i32,
                },
            );
        }
    }

    pub fn render(&mut self, render_queue: &[DrawCommand], lights: &SceneLights) {
        let size = self.ctx.window.inner_size();
        if size.width > 0
            && size.height > 0
            && (size.width != self.post_process.width() as u32
                || size.height != self.post_process.height() as u32)
        {
            self.resize(size.width, size.height);
        }

        self.camera.update();

        unsafe {
            self.post_process.begin(&self.gl);

            // Post-process pass changes GL state — invalidate cache
            self.state_cache.invalidate();

            self.setup_frame();

            let groups = self.prepare_batches(render_queue);

            for ((mesh_type, material_id), instances) in groups {
                self.draw_batch(mesh_type, material_id, &instances);
            }

            // Post-process pass invalidates state
            self.state_cache.invalidate();

            self.post_process.end(
                &self.gl,
                self.ctx.window.inner_size().width as i32,
                self.ctx.window.inner_size().height as i32,
                &self.camera,
                lights,
            );

            self.ctx
                .gl_surface
                .swap_buffers(&self.ctx.gl_context)
                .unwrap();
        }
    }

    unsafe fn setup_frame(&mut self) {
        unsafe {
            self.state_cache.use_program(&self.gl, self.program);
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl.clear_depth_f32(1.0);
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            let vp = self.camera.view_projection_matrix();
            let location = self
                .gl
                .get_uniform_location(self.program, "u_ViewProjection");
            self.gl
                .uniform_matrix_4_f32_slice(location.as_ref(), false, &vp.to_cols_array());
        }
    }

    fn prepare_batches(
        &self,
        render_queue: &[DrawCommand],
    ) -> HashMap<(MeshType, MaterialId), Vec<Instance>> {
        let mut groups: HashMap<(MeshType, MaterialId), Vec<Instance>> = HashMap::new();
        for cmd in render_queue {
            let model_matrix = glam::Mat4::from_translation(cmd.position)
                * glam::Mat4::from_quat(cmd.rotation)
                * glam::Mat4::from_scale(cmd.scale);

            groups
                .entry((cmd.mesh_type, cmd.material))
                .or_default()
                .push(Instance {
                    model_matrix,
                    tint: cmd.tint,
                });
        }
        groups
    }

    unsafe fn draw_batch(
        &mut self,
        mesh_type: MeshType,
        material_id: MaterialId,
        instances: &[Instance],
    ) {
        unsafe {
            if let (Some(batch), Some(material)) = (
                self.batches.get(&mesh_type),
                self.materials.get(&material_id),
            ) {
                let has_texture_loc = self.gl.get_uniform_location(self.program, "u_HasTexture");
                let albedo_color_loc = self.gl.get_uniform_location(self.program, "u_AlbedoColor");
                
                self.gl.uniform_4_f32(albedo_color_loc.as_ref(), material.albedo_color.x, material.albedo_color.y, material.albedo_color.z, material.albedo_color.w);

                if let Some(tex_id) = &material.albedo_texture {
                    self.textures[tex_id.0].bind(&self.gl, self.program, 0);
                    self.gl.uniform_1_u32(has_texture_loc.as_ref(), 1);
                } else {
                    self.gl.bind_texture(glow::TEXTURE_2D, None);
                    self.gl.uniform_1_u32(has_texture_loc.as_ref(), 0);
                }

                // State-cached VAO bind — skips if already bound
                self.state_cache.bind_vao(&self.gl, batch.vao);
                self.gl
                    .bind_buffer(glow::ARRAY_BUFFER, Some(batch.instance_buffer));

                // Buffer Orphaning: allocate with null first, then sub_data.
                // Tells the driver "I don't need the old data" so it can
                // pipeline the upload without stalling.
                let byte_len = std::mem::size_of_val(instances);
                self.gl
                    .buffer_data_size(glow::ARRAY_BUFFER, byte_len as i32, glow::DYNAMIC_DRAW);
                self.gl.buffer_sub_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    0,
                    bytemuck::cast_slice(instances),
                );

                pipeline::draw_instanced(&self.gl, batch.indices_count, instances.len() as i32);
            }
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.ctx.gl_surface.resize(
                &self.ctx.gl_context,
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            );
            unsafe {
                self.gl.viewport(0, 0, width as i32, height as i32);
            }
            self.camera.resize(width as f32, height as f32);
            self.post_process
                .resize(&self.gl, width as i32, height as i32);
        }
    }

    pub fn request_redraw(&self) {
        self.ctx.window.request_redraw();
    }
}
