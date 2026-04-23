use crate::mesh::Vertex;
use glow::{Context, HasContext};

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.5, 0.5, 0.0],
        tex_coords: [1.0, 1.0],
    }, // Top Right
    Vertex {
        position: [0.5, -0.5, 0.0],
        tex_coords: [1.0, 0.0],
    }, // Bottom Right
    Vertex {
        position: [-0.5, -0.5, 0.0],
        tex_coords: [0.0, 0.0],
    }, // Bottom Left
    Vertex {
        position: [-0.5, 0.5, 0.0],
        tex_coords: [0.0, 1.0],
    }, // Top Left
];

pub const INDICES: &[u32] = &[0, 1, 3, 1, 2, 3];

pub fn setup_buffers(gl: &Context) -> (glow::VertexArray, glow::Buffer, glow::Buffer) {
    unsafe {
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));

        let vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

        let slice = std::slice::from_raw_parts(
            VERTICES.as_ptr() as *const u8,
            std::mem::size_of_val(VERTICES),
        );
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, slice, glow::STATIC_DRAW);

        let ebo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));

        let indices_slice = std::slice::from_raw_parts(
            INDICES.as_ptr() as *const u8,
            std::mem::size_of_val(INDICES),
        );
        gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, indices_slice, glow::STATIC_DRAW);

        (vao, vbo, ebo)
    }
}

pub fn setup_instance_buffer(gl: &Context) -> glow::Buffer {
    unsafe {
        let buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
        buffer
    }
}
