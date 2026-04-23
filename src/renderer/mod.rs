pub mod buffer;
pub mod context;
pub mod pipeline;
pub mod shader;

use glow::{Context, HasContext};
use std::num::NonZeroU32;
use glutin::context::PossiblyCurrentContext;
use glutin::prelude::GlSurface;
use glutin::surface::{Surface, WindowSurface};
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub struct RenderState {
    gl: Context,
    gl_surface: Surface<WindowSurface>,
    gl_context: PossiblyCurrentContext,
    window: Window,
    program: glow::Program,
    pub camera: crate::camera::Camera,
}

impl RenderState {
    pub fn new(event_loop: &ActiveEventLoop, title: &str, width: i32, height: i32) -> Self {
        let (gl, gl_surface, gl_context, window) =
            context::init_context(event_loop, title, width, height);

        let program = unsafe {
            let (_vao, _vbo, _ebo) = buffer::setup_buffers(&gl);

            let program = gl.create_program().unwrap();
            shader::create_shaders(&gl, program);
            gl.use_program(Some(program));

            pipeline::setup_pipeline(&gl);

            program
        };

        let mut camera = crate::camera::Camera::new_perspective(
            45.0,
            (width as f32) / (height as f32),
            0.1,
            100.0,
        );
        camera.set_position(glam::vec3(0.0, 0.0, -2.0));

        Self {
            gl,
            gl_surface,
            gl_context,
            window,
            program,
            camera,
        }
    }

    pub fn render(&self) {
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

            pipeline::draw(&self.gl, buffer::INDICES.len() as i32);

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
}
