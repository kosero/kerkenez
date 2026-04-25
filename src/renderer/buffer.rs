use glow::{
    ARRAY_BUFFER, Buffer, Context, ELEMENT_ARRAY_BUFFER, HasContext, STATIC_DRAW, VertexArray,
};

use crate::{error::KerkenezError, mesh::Mesh};

pub fn setup_mesh_buffers(
    gl: &Context,
    mesh: &Mesh,
) -> Result<(VertexArray, Buffer, Buffer), KerkenezError> {
    unsafe {
        let vao = gl
            .create_vertex_array()
            .map_err(KerkenezError::ResourceCreationError)?;
        gl.bind_vertex_array(Some(vao));

        let vbo = gl
            .create_buffer()
            .map_err(KerkenezError::ResourceCreationError)?;
        gl.bind_buffer(ARRAY_BUFFER, Some(vbo));

        gl.buffer_data_u8_slice(
            ARRAY_BUFFER,
            bytemuck::cast_slice(&mesh.vertices),
            STATIC_DRAW,
        );

        let ebo = gl
            .create_buffer()
            .map_err(KerkenezError::ResourceCreationError)?;
        gl.bind_buffer(ELEMENT_ARRAY_BUFFER, Some(ebo));

        gl.buffer_data_u8_slice(
            ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(&mesh.indices),
            STATIC_DRAW,
        );

        Ok((vao, vbo, ebo))
    }
}

pub fn setup_instance_buffer(gl: &Context) -> Result<Buffer, KerkenezError> {
    unsafe {
        let buffer = gl
            .create_buffer()
            .map_err(KerkenezError::ResourceCreationError)?;
        gl.bind_buffer(ARRAY_BUFFER, Some(buffer));
        Ok(buffer)
    }
}
