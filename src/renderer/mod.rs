pub mod buffer;
pub mod context;
pub mod pipeline;
pub mod shader;

use glow::{Context, HasContext};
use glutin::context::PossiblyCurrentContext;
use glutin::prelude::GlSurface;
use glutin::surface::{Surface, WindowSurface};
use std::num::NonZeroU32;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub struct RenderState {
    gl: Context,
    gl_surface: Surface<WindowSurface>,
    gl_context: PossiblyCurrentContext,
    window: Window,
    program: glow::Program,
    pub camera: crate::camera::Camera,
    _instance_buffer: glow::Buffer,
    vao: glow::VertexArray,
    pub instances: Vec<crate::mesh::Instance>,
}

impl RenderState {
    pub fn new(event_loop: &ActiveEventLoop, title: &str, width: i32, height: i32) -> Self {
        let (gl, gl_surface, gl_context, window) =
            context::init_context(event_loop, title, width, height);

        let (program, instance_buffer, vao) = unsafe {
            let (vao, vbo, ebo) = buffer::setup_buffers(&gl);
            gl.bind_vertex_array(Some(vao));

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            pipeline::setup_pipeline(&gl);

            let instance_buffer = buffer::setup_instance_buffer(&gl);
            pipeline::setup_instancing(&gl);

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));

            let program = gl.create_program().unwrap();
            shader::create_shaders(&gl, program);
            gl.use_program(Some(program));

            (program, instance_buffer, vao)
        };

        let mut camera = crate::camera::Camera::new_perspective(
            45.0,
            (width as f32) / (height as f32),
            0.1,
            1000.0,
        );
        camera.set_position(glam::vec3(0.0, 0.0, -10.0));

        let instances = Vec::new();

        Self {
            gl,
            gl_surface,
            gl_context,
            window,
            program,
            camera,
            _instance_buffer: instance_buffer,
            vao,
            instances,
        }
    }

    pub fn update_instances(&mut self, instances: &[crate::mesh::Instance]) {
        self.instances.clear();
        self.instances.extend_from_slice(instances);

        unsafe {
            self.gl
                .bind_buffer(glow::ARRAY_BUFFER, Some(self._instance_buffer));
            let slice = std::slice::from_raw_parts(
                self.instances.as_ptr() as *const u8,
                std::mem::size_of_val(&self.instances[..]),
            );
            self.gl
                .buffer_data_u8_slice(glow::ARRAY_BUFFER, slice, glow::DYNAMIC_DRAW);
        }
    }

    pub fn render(&self) {
        unsafe {
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            // Upload matrices
            let vp = self.camera.view_projection_matrix();
            let location = self
                .gl
                .get_uniform_location(self.program, "u_ViewProjection");
            self.gl
                .uniform_matrix_4_f32_slice(location.as_ref(), false, &vp.to_cols_array());

            // Dispatch draw call
            self.gl.bind_vertex_array(Some(self.vao));
            pipeline::draw_instanced(
                &self.gl,
                buffer::INDICES.len() as i32,
                self.instances.len() as i32,
            );

            self.gl_surface.swap_buffers(&self.gl_context).unwrap();
            self.window.request_redraw();
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
