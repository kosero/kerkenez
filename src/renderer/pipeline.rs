use crate::mesh::{Instance, Vertex};
use glow::{Context, HasContext};

pub fn setup_pipeline(gl: &Context) {
    unsafe {
        let stride = std::mem::size_of::<Vertex>() as i32;

        // location 0: position (vec3)
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
        gl.enable_vertex_attrib_array(0);

        // location 1: tex_coords (vec2)
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, 12);
        gl.enable_vertex_attrib_array(1);

        gl.clear_color(0.1, 0.1, 0.1, 1.0);
    }
}

pub fn setup_instancing(gl: &Context) {
    unsafe {
        let stride = std::mem::size_of::<Instance>() as i32;
        for i in 0..4 {
            let index = 2 + i;
            gl.enable_vertex_attrib_array(index);
            gl.vertex_attrib_pointer_f32(index, 4, glow::FLOAT, false, stride, (i * 16) as i32);
            gl.vertex_attrib_divisor(index, 1);
        }

        gl.enable_vertex_attrib_array(6);
        gl.vertex_attrib_pointer_f32(6, 3, glow::FLOAT, false, stride, 64);
        gl.vertex_attrib_divisor(6, 1);
    }
}

pub fn draw(gl: &Context, count: i32) {
    unsafe {
        gl.draw_elements(glow::TRIANGLES, count, glow::UNSIGNED_INT, 0);
    }
}

pub fn draw_instanced(gl: &Context, count: i32, instance_count: i32) {
    unsafe {
        gl.draw_elements_instanced(
            glow::TRIANGLES,
            count,
            glow::UNSIGNED_INT,
            0,
            instance_count,
        );
    }
}
