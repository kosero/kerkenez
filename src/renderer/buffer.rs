use crate::mesh::{Mesh, Vertex};
use glow::{Context, HasContext};

pub fn setup_mesh_buffers(
    gl: &Context,
    mesh: &Mesh,
) -> (glow::VertexArray, glow::Buffer, glow::Buffer) {
    unsafe {
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));

        let vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

        let slice = std::slice::from_raw_parts(
            mesh.vertices.as_ptr() as *const u8,
            mesh.vertices.len() * std::mem::size_of::<Vertex>(),
        );
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, slice, glow::STATIC_DRAW);

        let ebo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));

        let indices_slice = std::slice::from_raw_parts(
            mesh.indices.as_ptr() as *const u8,
            mesh.indices.len() * std::mem::size_of::<u32>(),
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
