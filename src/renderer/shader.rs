use glow::{Context, HasContext, Program};

pub fn create_shaders(gl: &Context, program: Program) {
    let shader_srcs = [
        (
            glow::VERTEX_SHADER,
            include_str!("../../shaders/vertex.vert"),
        ),
        (
            glow::FRAGMENT_SHADER,
            include_str!("../../shaders/fragment.frag"),
        ),
    ];

    let mut shaders = Vec::with_capacity(shader_srcs.len());

    unsafe {
        for (shader_type, src) in &shader_srcs {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, src);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!("{}", gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("{}", gl.get_program_info_log(program));
        }

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }
    }
}
