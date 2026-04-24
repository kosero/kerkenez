use crate::error::EngineError;
use glow::{Context, HasContext, Program};

pub fn create_shaders(
    gl: &Context,
    program: Program,
    vert_src: &str,
    frag_src: &str,
) -> Result<(), EngineError> {
    let shader_srcs = [
        (glow::VERTEX_SHADER, vert_src),
        (glow::FRAGMENT_SHADER, frag_src),
    ];

    let mut shaders = Vec::with_capacity(shader_srcs.len());

    unsafe {
        for (shader_type, src) in &shader_srcs {
            let shader = gl
                .create_shader(*shader_type)
                .map_err(EngineError::ResourceCreationError)?;
            gl.shader_source(shader, src);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                let info = gl.get_shader_info_log(shader);
                // Clean up before returning
                for s in &shaders {
                    gl.detach_shader(program, *s);
                    gl.delete_shader(*s);
                }
                gl.delete_shader(shader);
                return Err(EngineError::ShaderCompileError(info));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            let info = gl.get_program_info_log(program);
            // Clean up before returning
            for s in shaders {
                gl.detach_shader(program, s);
                gl.delete_shader(s);
            }
            return Err(EngineError::ShaderLinkError(info));
        }

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }
    }

    Ok(())
}
