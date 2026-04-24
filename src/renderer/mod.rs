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

pub struct MeshBatch {
    vao: glow::VertexArray,
    instance_buffer: glow::Buffer,
    indices_count: i32,
}

pub struct RenderState {
    gl: Context,
    gl_surface: Surface<WindowSurface>,
    gl_context: PossiblyCurrentContext,
    window: Window,
    program: glow::Program,
    pub camera: Camera,

    batches: HashMap<MeshType, MeshBatch>,
    materials: HashMap<MaterialId, Material>,
    textures: HashMap<String, Texture>,
}

impl RenderState {
    pub fn new(event_loop: &ActiveEventLoop, title: &str, width: i32, height: i32) -> Self {
        let (gl, gl_surface, gl_context, window) =
            context::init_context(event_loop, title, width, height);

        let program = unsafe {
            let program = gl.create_program().unwrap();
            shader::create_shaders(&gl, program);
            gl.use_program(Some(program));
            program
        };

        let mut state = Self {
            gl,
            gl_surface,
            gl_context,
            window,
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

        // Register default material as plain white, no texture
        state.register_material(
            MaterialId(0),
            Material::new("default", glam::Vec3::ONE, None),
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
        let (vao, _vbo, _ebo) = buffer::setup_mesh_buffers(&self.gl, mesh);
        unsafe {
            self.gl.bind_vertex_array(Some(vao));

            pipeline::setup_pipeline(&self.gl);
            let instance_buffer = buffer::setup_instance_buffer(&self.gl);
            pipeline::setup_instancing(&self.gl);

            self.batches.insert(
                mesh_type,
                MeshBatch {
                    vao,
                    instance_buffer,
                    indices_count: mesh.indices.len() as i32,
                },
            );
        }
    }

    pub fn render(&mut self, render_queue: &[RenderCommand]) {
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

            // Batching: Group by (MeshType, MaterialId)
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

            // Draw each batch
            for ((mesh_type, material_id), instances) in groups {
                if let (Some(batch), Some(material)) = (
                    self.batches.get(&mesh_type),
                    self.materials.get(&material_id),
                ) {
                    let has_texture_loc =
                        self.gl.get_uniform_location(self.program, "u_HasTexture");

                    // Bind material texture if exists
                    if let Some(path) = &material.texture_path {
                        if let Some(texture) = self.textures.get(path) {
                            texture.bind(&self.gl, self.program);
                            self.gl.uniform_1_u32(has_texture_loc.as_ref(), 1);
                        } else {
                            self.gl.uniform_1_u32(has_texture_loc.as_ref(), 0);
                        }
                    } else {
                        // Unbind if no texture is specified for this material
                        self.gl.bind_texture(glow::TEXTURE_2D, None);
                        self.gl.uniform_1_u32(has_texture_loc.as_ref(), 0);
                    }

                    self.gl.bind_vertex_array(Some(batch.vao));
                    self.gl
                        .bind_buffer(glow::ARRAY_BUFFER, Some(batch.instance_buffer));

                    let instances_slice: &[Instance] = &instances;
                    let slice = std::slice::from_raw_parts(
                        instances_slice.as_ptr() as *const u8,
                        instances.len() * std::mem::size_of::<Instance>(),
                    );
                    self.gl
                        .buffer_data_u8_slice(glow::ARRAY_BUFFER, slice, glow::DYNAMIC_DRAW);

                    pipeline::draw_instanced(&self.gl, batch.indices_count, instances.len() as i32);
                }
            }

            self.gl_surface.swap_buffers(&self.gl_context).unwrap();
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.gl_surface.resize(
                &self.gl_context,
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
        self.window.request_redraw();
    }
}
