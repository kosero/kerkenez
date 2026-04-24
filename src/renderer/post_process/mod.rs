pub mod fbo;
pub mod fullscreen_triangle;
pub mod settings;

use self::fbo::FrameBuffer;
use self::fullscreen_triangle::FullscreenTriangle;
use self::settings::{DebugMode, PostProcessSettings};
use crate::renderer::shader;
use glow::{Context, HasContext};

struct UniformCache {
    screen_texture: Option<glow::UniformLocation>,
    depth_texture: Option<glow::UniformLocation>,
    near: Option<glow::UniformLocation>,
    far: Option<glow::UniformLocation>,
    inverse_vp: Option<glow::UniformLocation>,
    resolution: Option<glow::UniformLocation>,
    ssao_enabled: Option<glow::UniformLocation>,
    ssao_radius: Option<glow::UniformLocation>,
    ssao_intensity: Option<glow::UniformLocation>,
    ssao_bias: Option<glow::UniformLocation>,
    fog_enabled: Option<glow::UniformLocation>,
    fog_density: Option<glow::UniformLocation>,
    fog_color: Option<glow::UniformLocation>,
    tone_mapping_enabled: Option<glow::UniformLocation>,
    exposure: Option<glow::UniformLocation>,
    contrast: Option<glow::UniformLocation>,
    brightness: Option<glow::UniformLocation>,
    saturation: Option<glow::UniformLocation>,
    vignette_enabled: Option<glow::UniformLocation>,
    vignette_intensity: Option<glow::UniformLocation>,
}

impl UniformCache {
    fn new(gl: &Context, program: glow::Program) -> Self {
        unsafe {
            Self {
                screen_texture: gl.get_uniform_location(program, "u_ScreenTexture"),
                depth_texture: gl.get_uniform_location(program, "u_DepthTexture"),
                near: gl.get_uniform_location(program, "u_Near"),
                far: gl.get_uniform_location(program, "u_Far"),
                inverse_vp: gl.get_uniform_location(program, "u_InverseVP"),
                resolution: gl.get_uniform_location(program, "u_Resolution"),
                ssao_enabled: gl.get_uniform_location(program, "u_SSAOEnabled"),
                ssao_radius: gl.get_uniform_location(program, "u_SSAORadius"),
                ssao_intensity: gl.get_uniform_location(program, "u_SSAOIntensity"),
                ssao_bias: gl.get_uniform_location(program, "u_SSAOBias"),
                fog_enabled: gl.get_uniform_location(program, "u_FogEnabled"),
                fog_density: gl.get_uniform_location(program, "u_FogDensity"),
                fog_color: gl.get_uniform_location(program, "u_FogColor"),
                tone_mapping_enabled: gl.get_uniform_location(program, "u_ToneMappingEnabled"),
                exposure: gl.get_uniform_location(program, "u_Exposure"),
                contrast: gl.get_uniform_location(program, "u_Contrast"),
                brightness: gl.get_uniform_location(program, "u_Brightness"),
                saturation: gl.get_uniform_location(program, "u_Saturation"),
                vignette_enabled: gl.get_uniform_location(program, "u_VignetteEnabled"),
                vignette_intensity: gl.get_uniform_location(program, "u_VignetteIntensity"),
            }
        }
    }
}

struct ProgramVariant {
    program: glow::Program,
    uniforms: UniformCache,
}

pub struct PostProcessManager {
    fbo: FrameBuffer,
    triangle: FullscreenTriangle,
    variants: [ProgramVariant; 4],
    pub settings: PostProcessSettings,
}

impl PostProcessManager {
    pub fn new(gl: &Context, width: i32, height: i32) -> Self {
        let fbo = FrameBuffer::new(gl, width, height);
        let triangle = FullscreenTriangle::new(gl);

        let vert_src = include_str!("../../../shaders/screen_quad.vert");
        let frag_src = include_str!("../../../shaders/post_fragment.frag");

        let create_variant = |mode: i32| -> ProgramVariant {
            unsafe {
                let program = gl
                    .create_program()
                    .expect("Failed to create post-process program");

                // Inject the #define right after the version pragma
                let version_pragma = "#version 410 core\n";
                let frag_src_modified =
                    if let Some(stripped) = frag_src.strip_prefix(version_pragma) {
                        format!("{version_pragma}#define DEBUG_MODE {mode}\n{stripped}")
                    } else {
                        format!("#define DEBUG_MODE {mode}\n{frag_src}")
                    };

                shader::create_shaders(gl, program, vert_src, &frag_src_modified);
                let uniforms = UniformCache::new(gl, program);
                ProgramVariant { program, uniforms }
            }
        };

        let variants = [
            create_variant(0),
            create_variant(1),
            create_variant(2),
            create_variant(3),
        ];

        Self {
            fbo,
            triangle,
            variants,
            settings: PostProcessSettings::default(),
        }
    }

    pub fn width(&self) -> i32 {
        self.fbo.width
    }

    pub fn height(&self) -> i32 {
        self.fbo.height
    }

    /// Bind the off-screen FBO so all subsequent draw calls render into it.
    pub fn begin(&self, gl: &Context) {
        if self.settings.enabled {
            self.fbo.bind(gl);
            unsafe {
                gl.clear_color(0.1, 0.1, 0.1, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            }
        }
    }

    /// Unbind the FBO and draw the fullscreen pass with all post-processing.
    pub fn end(
        &self,
        gl: &Context,
        window_width: i32,
        window_height: i32,
        near: f32,
        far: f32,
        inv_vp: glam::Mat4,
    ) {
        if self.settings.enabled {
            self.fbo.unbind(gl);

            let variant_idx = match self.settings.debug_mode {
                DebugMode::None => 0,
                DebugMode::Depth => 1,
                DebugMode::Normals => 2,
                DebugMode::SSAO => 3,
            };
            let variant = &self.variants[variant_idx];

            unsafe {
                gl.viewport(0, 0, window_width, window_height);
                gl.clear_color(0.0, 0.0, 0.0, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT);

                gl.use_program(Some(variant.program));

                // Bind textures
                gl.active_texture(glow::TEXTURE0);
                gl.bind_texture(glow::TEXTURE_2D, Some(self.fbo.color_texture));
                gl.uniform_1_i32(variant.uniforms.screen_texture.as_ref(), 0);

                gl.active_texture(glow::TEXTURE1);
                gl.bind_texture(glow::TEXTURE_2D, Some(self.fbo.depth_texture));
                gl.uniform_1_i32(variant.uniforms.depth_texture.as_ref(), 1);

                // Camera uniforms
                gl.uniform_1_f32(variant.uniforms.near.as_ref(), near);
                gl.uniform_1_f32(variant.uniforms.far.as_ref(), far);
                gl.uniform_matrix_4_f32_slice(
                    variant.uniforms.inverse_vp.as_ref(),
                    false,
                    &inv_vp.to_cols_array(),
                );
                gl.uniform_2_f32(
                    variant.uniforms.resolution.as_ref(),
                    window_width as f32,
                    window_height as f32,
                );

                // SSAO
                gl.uniform_1_i32(
                    variant.uniforms.ssao_enabled.as_ref(),
                    self.settings.ssao_enabled as i32,
                );
                gl.uniform_1_f32(
                    variant.uniforms.ssao_radius.as_ref(),
                    self.settings.ssao_radius,
                );
                gl.uniform_1_f32(
                    variant.uniforms.ssao_intensity.as_ref(),
                    self.settings.ssao_intensity,
                );
                gl.uniform_1_f32(variant.uniforms.ssao_bias.as_ref(), self.settings.ssao_bias);

                // Fog
                gl.uniform_1_i32(
                    variant.uniforms.fog_enabled.as_ref(),
                    self.settings.fog_enabled as i32,
                );
                gl.uniform_1_f32(
                    variant.uniforms.fog_density.as_ref(),
                    self.settings.fog_density,
                );
                gl.uniform_3_f32(
                    variant.uniforms.fog_color.as_ref(),
                    self.settings.fog_color[0],
                    self.settings.fog_color[1],
                    self.settings.fog_color[2],
                );

                // Tone mapping & color grading
                gl.uniform_1_i32(
                    variant.uniforms.tone_mapping_enabled.as_ref(),
                    self.settings.tone_mapping_enabled as i32,
                );
                gl.uniform_1_f32(variant.uniforms.exposure.as_ref(), self.settings.exposure);
                gl.uniform_1_f32(variant.uniforms.contrast.as_ref(), self.settings.contrast);
                gl.uniform_1_f32(
                    variant.uniforms.brightness.as_ref(),
                    self.settings.brightness,
                );
                gl.uniform_1_f32(
                    variant.uniforms.saturation.as_ref(),
                    self.settings.saturation,
                );

                // Vignette
                gl.uniform_1_i32(
                    variant.uniforms.vignette_enabled.as_ref(),
                    self.settings.vignette_enabled as i32,
                );
                gl.uniform_1_f32(
                    variant.uniforms.vignette_intensity.as_ref(),
                    self.settings.vignette_intensity,
                );

                // Draw fullscreen triangle
                gl.disable(glow::DEPTH_TEST);
                self.triangle.draw(gl);
                gl.enable(glow::DEPTH_TEST);
            }
        }
    }

    pub fn resize(&mut self, gl: &Context, width: i32, height: i32) {
        self.fbo.resize(gl, width, height);
    }

    pub fn delete(&self, gl: &Context) {
        self.fbo.delete(gl);
        self.triangle.delete(gl);
        unsafe {
            for variant in &self.variants {
                gl.delete_program(variant.program);
            }
        }
    }
}
