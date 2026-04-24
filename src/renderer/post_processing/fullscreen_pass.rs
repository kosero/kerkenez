use glow::{Context, HasContext};
use std::rc::Rc;

pub struct FullscreenPass {
    gl: Rc<Context>,
    vao: glow::VertexArray,
}

impl Drop for FullscreenPass {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vao);
        }
    }
}

impl FullscreenPass {
    pub fn new(gl: &Rc<Context>) -> Self {
        unsafe {
            let vao = gl.create_vertex_array().expect("Failed to create VAO");
            Self {
                gl: gl.clone(),
                vao,
            }
        }
    }

    pub fn draw(&self, gl: &Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }

}
