use crate::error::EngineError;
use crate::mesh::Mesh;
use glow::{Context, HasContext};

pub fn setup_mesh_buffers(
    gl: &Context,
    mesh: &Mesh,
) -> Result<(glow::VertexArray, glow::Buffer, glow::Buffer), EngineError> {
    unsafe {
        let vao = gl
            .create_vertex_array()
            .map_err(EngineError::ResourceCreationError)?;
        gl.bind_vertex_array(Some(vao));

        let vbo = gl
            .create_buffer()
            .map_err(EngineError::ResourceCreationError)?;
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(&mesh.vertices),
            glow::STATIC_DRAW,
        );

        let ebo = gl
            .create_buffer()
            .map_err(EngineError::ResourceCreationError)?;
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));

        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(&mesh.indices),
            glow::STATIC_DRAW,
        );

        Ok((vao, vbo, ebo))
    }
}

pub fn setup_instance_buffer(gl: &Context) -> Result<glow::Buffer, EngineError> {
    unsafe {
        let buffer = gl
            .create_buffer()
            .map_err(EngineError::ResourceCreationError)?;
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
        Ok(buffer)
    }
}
