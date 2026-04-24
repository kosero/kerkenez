pub mod buffer;
pub mod context;
pub mod material;
pub mod pipeline;
pub mod shader;
pub mod texture;

use self::material::{Material, MaterialId};
use crate::camera::Camera;
use crate::mesh::{Instance, Mesh, MeshType, RenderCommand};
use glow::{Context, HasContext};
use glutin::context::PossiblyCurrentContext;
use glutin::prelude::GlSurface;
use glutin::surface::{Surface, WindowSurface};
use std::collections::HashMap;
use std::num::NonZeroU32;
use texture::Texture;
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
    textures: HashMap<String, Texture>,
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
            textures: HashMap::new(),
        };
        state.camera.set_position(glam::vec3(0.0, 0.0, -10.0));

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

    pub fn register_material(&mut self, id: MaterialId, material: Material) {
        if let Some(path) = material
            .texture_path
            .as_ref()
            .filter(|p| !self.textures.contains_key(*p))
        {
            let texture = Texture::load(&self.gl, path);
            self.textures.insert(path.to_string(), texture);
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

    pub fn render(&mut self, render_queue: &[RenderCommand]) {
        unsafe {
            self.setup_frame();

            let groups = self.prepare_batches(render_queue);

            for ((mesh_type, material_id), instances) in groups {
                self.draw_batch(mesh_type, material_id, &instances);
            }

            self.ctx
                .gl_surface
                .swap_buffers(&self.ctx.gl_context)
                .unwrap();
        }
    }

    unsafe fn setup_frame(&self) {
        unsafe {
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
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
        render_queue: &[RenderCommand],
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
                    color: cmd.color,
                });
        }
        groups
    }

    unsafe fn draw_batch(
        &self,
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

                if let Some(path) = &material.texture_path {
                    if let Some(texture) = self.textures.get(path) {
                        texture.bind(&self.gl, self.program);
                        self.gl.uniform_1_u32(has_texture_loc.as_ref(), 1);
                    }
                } else {
                    self.gl.bind_texture(glow::TEXTURE_2D, None);
                    self.gl.uniform_1_u32(has_texture_loc.as_ref(), 0);
                }

                self.gl.bind_vertex_array(Some(batch.vao));
                self.gl
                    .bind_buffer(glow::ARRAY_BUFFER, Some(batch.instance_buffer));

                let slice = std::slice::from_raw_parts(
                    instances.as_ptr() as *const u8,
                    std::mem::size_of_val(instances),
                );
                self.gl
                    .buffer_data_u8_slice(glow::ARRAY_BUFFER, slice, glow::DYNAMIC_DRAW);

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
        }
    }

    pub fn request_redraw(&self) {
        self.ctx.window.request_redraw();
    }
}
