use crate::mesh::Vertex;
use glow::{Context, HasContext};

pub unsafe fn setup_pipeline(gl: &Context) {
    unsafe {
        let stride = std::mem::size_of::<Vertex>() as i32;
        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride, 8);
        gl.enable_vertex_attrib_array(1);

        gl.clear_color(0.1, 0.1, 0.1, 1.0);
    }
}

pub unsafe fn draw(gl: &Context, count: i32) {
    unsafe {
        gl.draw_elements(glow::TRIANGLES, count, glow::UNSIGNED_INT, 0);
    }
}
