pub mod buffer;
pub mod context;
pub mod pipeline;
pub mod shader;

use glow::{Context, HasContext};
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
}

impl RenderState {
    pub fn new(event_loop: &ActiveEventLoop, title: &str, width: i32, height: i32) -> Self {
        let (gl, gl_surface, gl_context, window) =
            context::init_context(event_loop, title, width, height);

        unsafe {
            let (_vao, _vbo, _ebo) = buffer::setup_buffers(&gl);

            let program = gl.create_program().unwrap();
            shader::create_shaders(&gl, program);
            gl.use_program(Some(program));

            pipeline::setup_pipeline(&gl);
        }

        Self {
            gl,
            gl_surface,
            gl_context,
            window,
        }
    }

    pub fn render(&self) {
        unsafe {
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            pipeline::draw(&self.gl, buffer::INDICES.len() as i32);

            self.gl_surface.swap_buffers(&self.gl_context).unwrap();
            self.window.request_redraw();
        }
    }
}
