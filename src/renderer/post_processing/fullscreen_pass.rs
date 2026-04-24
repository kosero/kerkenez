use glow::{Context, HasContext};

pub struct FullscreenPass {
    vao: glow::VertexArray,
}

impl FullscreenPass {
    pub fn new(gl: &Context) -> Self {
        unsafe {
            let vao = gl.create_vertex_array().expect("Failed to create VAO");
            Self { vao }
        }
    }

    pub fn draw(&self, gl: &Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }

    pub fn delete(&self, gl: &Context) {
        unsafe {
            gl.delete_vertex_array(self.vao);
        }
    }
}
