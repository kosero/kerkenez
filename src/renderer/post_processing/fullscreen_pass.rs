use std::rc::Rc;

use glow::{Context, HasContext};

use crate::error::KerkenezError;

pub struct FullscreenPass {
    gl: Rc<Context>,
    pub vao: glow::VertexArray,
}

impl FullscreenPass {
    pub fn new(gl: &Rc<Context>) -> Result<Self, KerkenezError> {
        unsafe {
            let vao = gl
                .create_vertex_array()
                .map_err(KerkenezError::ResourceCreationError)?;

            Ok(Self {
                gl: gl.clone(),
                vao,
            })
        }
    }

    pub fn draw(&self, gl: &Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }
}

impl Drop for FullscreenPass {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vao);
        }
    }
}
