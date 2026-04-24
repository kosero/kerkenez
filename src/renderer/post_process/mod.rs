pub mod fbo;
pub mod fullscreen_triangle;
pub mod settings;

use self::fbo::{FrameBuffer, RenderTarget};
use self::fullscreen_triangle::FullscreenTriangle;
use self::settings::{DebugMode, PostProcessSettings};
use crate::renderer::shader;
use crate::renderer::light::{SceneLights, MAX_POINT_LIGHTS};
use glow::{Context, HasContext};

struct UniformCache {
    screen_texture: Option<glow::UniformLocation>,
    depth_texture: Option<glow::UniformLocation>,
    normal_texture: Option<glow::UniformLocation>,
    near: Option<glow::UniformLocation>,
    far: Option<glow::UniformLocation>,
    inverse_vp: Option<glow::UniformLocation>,
    resolution: Option<glow::UniformLocation>,
    ssao_enabled: Option<glow::UniformLocation>,
    fog_enabled: Option<glow::UniformLocation>,
    fog_density: Option<glow::UniformLocation>,
    fog_color: Option<glow::UniformLocation>,
    tone_mapping_enabled: Option<glow::UniformLocation>,
    exposure: Option<glow::UniformLocation>,
    contrast: Option<glow::UniformLocation>,
    brightness: Option<glow::UniformLocation>,
    saturation: Option<glow::UniformLocation>,
    vignette_enabled: Option<glow::UniformLocation>,
    ssao_texture: Option<glow::UniformLocation>,
    vignette_intensity: Option<glow::UniformLocation>,
    
    // Lighting
    camera_pos: Option<glow::UniformLocation>,
    ambient_color: Option<glow::UniformLocation>,
    ambient_intensity: Option<glow::UniformLocation>,
    dir_light_direction: Option<glow::UniformLocation>,
    dir_light_color: Option<glow::UniformLocation>,
    dir_light_intensity: Option<glow::UniformLocation>,
    dir_light_enabled: Option<glow::UniformLocation>,
    point_lights_count: Option<glow::UniformLocation>,
    
    // Arrays of struct fields are looked up individually
    point_light_positions: Vec<Option<glow::UniformLocation>>,
    point_light_colors: Vec<Option<glow::UniformLocation>>,
    point_light_intensities: Vec<Option<glow::UniformLocation>>,
    point_light_radii: Vec<Option<glow::UniformLocation>>,
}

struct SsaoUniforms {
    depth_texture: Option<glow::UniformLocation>,
    normal_texture: Option<glow::UniformLocation>,
    near: Option<glow::UniformLocation>,
    far: Option<glow::UniformLocation>,
    inverse_vp: Option<glow::UniformLocation>,
    resolution: Option<glow::UniformLocation>,
    ssao_radius: Option<glow::UniformLocation>,
    ssao_intensity: Option<glow::UniformLocation>,
    ssao_bias: Option<glow::UniformLocation>,
}

struct SsaoBlurUniforms {
    ssao_texture: Option<glow::UniformLocation>,
    depth_texture: Option<glow::UniformLocation>,
    near: Option<glow::UniformLocation>,
    far: Option<glow::UniformLocation>,
    resolution: Option<glow::UniformLocation>,
}

impl UniformCache {
    fn new(gl: &Context, program: glow::Program) -> Self {
        unsafe {
            Self {
                screen_texture: gl.get_uniform_location(program, "u_ScreenTexture"),
                depth_texture: gl.get_uniform_location(program, "u_DepthTexture"),
                normal_texture: gl.get_uniform_location(program, "u_NormalTexture"),
                near: gl.get_uniform_location(program, "u_Near"),
                far: gl.get_uniform_location(program, "u_Far"),
                inverse_vp: gl.get_uniform_location(program, "u_InverseVP"),
                resolution: gl.get_uniform_location(program, "u_Resolution"),
                ssao_enabled: gl.get_uniform_location(program, "u_SSAOEnabled"),
                fog_enabled: gl.get_uniform_location(program, "u_FogEnabled"),
                fog_density: gl.get_uniform_location(program, "u_FogDensity"),
                fog_color: gl.get_uniform_location(program, "u_FogColor"),
                tone_mapping_enabled: gl.get_uniform_location(program, "u_ToneMappingEnabled"),
                exposure: gl.get_uniform_location(program, "u_Exposure"),
                contrast: gl.get_uniform_location(program, "u_Contrast"),
                brightness: gl.get_uniform_location(program, "u_Brightness"),
                saturation: gl.get_uniform_location(program, "u_Saturation"),
                vignette_enabled: gl.get_uniform_location(program, "u_VignetteEnabled"),
                ssao_texture: gl.get_uniform_location(program, "u_SSAOTexture"),
                vignette_intensity: gl.get_uniform_location(program, "u_VignetteIntensity"),
                
                camera_pos: gl.get_uniform_location(program, "u_CameraPos"),
                ambient_color: gl.get_uniform_location(program, "u_AmbientColor"),
                ambient_intensity: gl.get_uniform_location(program, "u_AmbientIntensity"),
                dir_light_direction: gl.get_uniform_location(program, "u_DirLight.direction"),
                dir_light_color: gl.get_uniform_location(program, "u_DirLight.color"),
                dir_light_intensity: gl.get_uniform_location(program, "u_DirLight.intensity"),
                dir_light_enabled: gl.get_uniform_location(program, "u_DirLight.enabled"),
                point_lights_count: gl.get_uniform_location(program, "u_PointLightsCount"),
                
                point_light_positions: (0..MAX_POINT_LIGHTS).map(|i| gl.get_uniform_location(program, &format!("u_PointLights[{}].position", i))).collect(),
                point_light_colors: (0..MAX_POINT_LIGHTS).map(|i| gl.get_uniform_location(program, &format!("u_PointLights[{}].color", i))).collect(),
                point_light_intensities: (0..MAX_POINT_LIGHTS).map(|i| gl.get_uniform_location(program, &format!("u_PointLights[{}].intensity", i))).collect(),
                point_light_radii: (0..MAX_POINT_LIGHTS).map(|i| gl.get_uniform_location(program, &format!("u_PointLights[{}].radius", i))).collect(),
            }
        }
    }
}

impl SsaoUniforms {
    fn new(gl: &Context, program: glow::Program) -> Self {
        unsafe {
            Self {
                depth_texture: gl.get_uniform_location(program, "u_DepthTexture"),
                normal_texture: gl.get_uniform_location(program, "u_NormalTexture"),
                near: gl.get_uniform_location(program, "u_Near"),
                far: gl.get_uniform_location(program, "u_Far"),
                inverse_vp: gl.get_uniform_location(program, "u_InverseVP"),
                resolution: gl.get_uniform_location(program, "u_Resolution"),
                ssao_radius: gl.get_uniform_location(program, "u_SSAORadius"),
                ssao_intensity: gl.get_uniform_location(program, "u_SSAOIntensity"),
                ssao_bias: gl.get_uniform_location(program, "u_SSAOBias"),
            }
        }
    }
}

impl SsaoBlurUniforms {
    fn new(gl: &Context, program: glow::Program) -> Self {
        unsafe {
            Self {
                ssao_texture: gl.get_uniform_location(program, "u_SSAOTexture"),
                depth_texture: gl.get_uniform_location(program, "u_DepthTexture"),
                near: gl.get_uniform_location(program, "u_Near"),
                far: gl.get_uniform_location(program, "u_Far"),
                resolution: gl.get_uniform_location(program, "u_Resolution"),
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
    ssao_target: RenderTarget,
    ssao_blur_target: RenderTarget,
    triangle: FullscreenTriangle,

    ssao_program: glow::Program,
    ssao_uniforms: SsaoUniforms,

    ssao_blur_program: glow::Program,
    ssao_blur_uniforms: SsaoBlurUniforms,

    variants: [ProgramVariant; 4],
    pub settings: PostProcessSettings,
}

impl PostProcessManager {
    pub fn new(gl: &Context, width: i32, height: i32) -> Self {
        let fbo = FrameBuffer::new(gl, width, height);
        // SSAO textures only need a single channel. We use R16F for precision, though R8 could work.
        let ssao_target = RenderTarget::new(
            gl,
            width,
            height,
            glow::R16F as i32,
            glow::RED,
            glow::HALF_FLOAT,
        );
        let ssao_blur_target = RenderTarget::new(
            gl,
            width,
            height,
            glow::R16F as i32,
            glow::RED,
            glow::HALF_FLOAT,
        );
        let triangle = FullscreenTriangle::new(gl);

        let vert_src = include_str!("../../../shaders/screen_quad.vert");
        let frag_src = include_str!("../../../shaders/post_fragment.frag");
        let ssao_frag_src = include_str!("../../../shaders/ssao.frag");
        let ssao_blur_frag_src = include_str!("../../../shaders/ssao_blur.frag");

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

        let (ssao_program, ssao_blur_program) = unsafe {
            let sp = gl.create_program().expect("Failed to create ssao program");
            shader::create_shaders(gl, sp, vert_src, ssao_frag_src);

            let sbp = gl
                .create_program()
                .expect("Failed to create ssao blur program");
            shader::create_shaders(gl, sbp, vert_src, ssao_blur_frag_src);

            (sp, sbp)
        };

        let ssao_uniforms = SsaoUniforms::new(gl, ssao_program);
        let ssao_blur_uniforms = SsaoBlurUniforms::new(gl, ssao_blur_program);

        Self {
            fbo,
            ssao_target,
            ssao_blur_target,
            triangle,
            ssao_program,
            ssao_uniforms,
            ssao_blur_program,
            ssao_blur_uniforms,
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

    /// Bind the off-screen G-Buffer FBO so all subsequent draw calls render into it.
    pub fn begin(&self, gl: &Context) {
        if self.settings.enabled {
            self.fbo.bind(gl);
            unsafe {
                gl.clear_color(0.1, 0.1, 0.1, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            }
        }
    }

    /// Unbind the G-Buffer FBO and draw the fullscreen pass with all post-processing.
    pub fn end(
        &self,
        gl: &Context,
        window_width: i32,
        window_height: i32,
        near: f32,
        far: f32,
        inv_vp: glam::Mat4,
        camera_pos: glam::Vec3,
        lights: &SceneLights,
    ) {
        if self.settings.enabled {
            self.fbo.unbind(gl);

            // 1. SSAO Generation Pass
            if self.settings.ssao_enabled {
                unsafe {
                    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.ssao_target.fbo));
                    gl.viewport(0, 0, self.ssao_target.width, self.ssao_target.height);

                    gl.use_program(Some(self.ssao_program));

                    // Depth texture — unit 0
                    gl.active_texture(glow::TEXTURE0);
                    gl.bind_texture(glow::TEXTURE_2D, Some(self.fbo.depth_texture));
                    gl.uniform_1_i32(self.ssao_uniforms.depth_texture.as_ref(), 0);

                    // G-Buffer normal texture — unit 1
                    gl.active_texture(glow::TEXTURE1);
                    gl.bind_texture(glow::TEXTURE_2D, Some(self.fbo.normal_texture));
                    gl.uniform_1_i32(self.ssao_uniforms.normal_texture.as_ref(), 1);

                    gl.uniform_1_f32(self.ssao_uniforms.near.as_ref(), near);
                    gl.uniform_1_f32(self.ssao_uniforms.far.as_ref(), far);
                    gl.uniform_matrix_4_f32_slice(
                        self.ssao_uniforms.inverse_vp.as_ref(),
                        false,
                        &inv_vp.to_cols_array(),
                    );
                    gl.uniform_2_f32(
                        self.ssao_uniforms.resolution.as_ref(),
                        window_width as f32,
                        window_height as f32,
                    );

                    gl.uniform_1_f32(
                        self.ssao_uniforms.ssao_radius.as_ref(),
                        self.settings.ssao_radius,
                    );
                    gl.uniform_1_f32(
                        self.ssao_uniforms.ssao_intensity.as_ref(),
                        self.settings.ssao_intensity,
                    );
                    gl.uniform_1_f32(
                        self.ssao_uniforms.ssao_bias.as_ref(),
                        self.settings.ssao_bias,
                    );

                    gl.disable(glow::DEPTH_TEST);
                    self.triangle.draw(gl);

                    // 2. SSAO Blur Pass
                    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.ssao_blur_target.fbo));
                    gl.viewport(
                        0,
                        0,
                        self.ssao_blur_target.width,
                        self.ssao_blur_target.height,
                    );

                    gl.use_program(Some(self.ssao_blur_program));

                    gl.active_texture(glow::TEXTURE0);
                    gl.bind_texture(glow::TEXTURE_2D, Some(self.ssao_target.color_texture));
                    gl.uniform_1_i32(self.ssao_blur_uniforms.ssao_texture.as_ref(), 0);

                    gl.active_texture(glow::TEXTURE1);
                    gl.bind_texture(glow::TEXTURE_2D, Some(self.fbo.depth_texture));
                    gl.uniform_1_i32(self.ssao_blur_uniforms.depth_texture.as_ref(), 1);

                    gl.uniform_2_f32(
                        self.ssao_blur_uniforms.resolution.as_ref(),
                        window_width as f32,
                        window_height as f32,
                    );
                    gl.uniform_1_f32(self.ssao_blur_uniforms.near.as_ref(), near);
                    gl.uniform_1_f32(self.ssao_blur_uniforms.far.as_ref(), far);

                    self.triangle.draw(gl);
                }
            }

            // 3. Final Compose Pass
            let variant_idx = match self.settings.debug_mode {
                DebugMode::None => 0,
                DebugMode::Depth => 1,
                DebugMode::Normals => 2,
                DebugMode::SSAO => 3,
            };
            let variant = &self.variants[variant_idx];

            unsafe {
                gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                gl.viewport(0, 0, window_width, window_height);
                gl.clear_color(0.0, 0.0, 0.0, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT);

                gl.use_program(Some(variant.program));

                // Bind G-Buffer textures
                gl.active_texture(glow::TEXTURE0);
                gl.bind_texture(glow::TEXTURE_2D, Some(self.fbo.color_texture));
                gl.uniform_1_i32(variant.uniforms.screen_texture.as_ref(), 0);

                gl.active_texture(glow::TEXTURE1);
                gl.bind_texture(glow::TEXTURE_2D, Some(self.fbo.depth_texture));
                gl.uniform_1_i32(variant.uniforms.depth_texture.as_ref(), 1);

                gl.active_texture(glow::TEXTURE2);
                gl.bind_texture(glow::TEXTURE_2D, Some(self.ssao_blur_target.color_texture));
                gl.uniform_1_i32(variant.uniforms.ssao_texture.as_ref(), 2);

                // G-Buffer normal texture — unit 3
                gl.active_texture(glow::TEXTURE3);
                gl.bind_texture(glow::TEXTURE_2D, Some(self.fbo.normal_texture));
                gl.uniform_1_i32(variant.uniforms.normal_texture.as_ref(), 3);

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
                gl.uniform_3_f32(
                    variant.uniforms.camera_pos.as_ref(),
                    camera_pos.x,
                    camera_pos.y,
                    camera_pos.z,
                );

                // Lighting
                gl.uniform_3_f32(
                    variant.uniforms.ambient_color.as_ref(),
                    lights.ambient_color.x,
                    lights.ambient_color.y,
                    lights.ambient_color.z,
                );
                gl.uniform_1_f32(
                    variant.uniforms.ambient_intensity.as_ref(),
                    lights.ambient_intensity,
                );
                
                if let Some(dir_light) = &lights.directional {
                    gl.uniform_1_i32(variant.uniforms.dir_light_enabled.as_ref(), 1);
                    gl.uniform_3_f32(
                        variant.uniforms.dir_light_direction.as_ref(),
                        dir_light.direction.x,
                        dir_light.direction.y,
                        dir_light.direction.z,
                    );
                    gl.uniform_3_f32(
                        variant.uniforms.dir_light_color.as_ref(),
                        dir_light.color.x,
                        dir_light.color.y,
                        dir_light.color.z,
                    );
                    gl.uniform_1_f32(
                        variant.uniforms.dir_light_intensity.as_ref(),
                        dir_light.intensity,
                    );
                } else {
                    gl.uniform_1_i32(variant.uniforms.dir_light_enabled.as_ref(), 0);
                }

                let point_lights_len = lights.point_lights.len().min(MAX_POINT_LIGHTS);
                gl.uniform_1_i32(variant.uniforms.point_lights_count.as_ref(), point_lights_len as i32);
                for (i, light) in lights.point_lights.iter().take(MAX_POINT_LIGHTS).enumerate() {
                    gl.uniform_3_f32(
                        variant.uniforms.point_light_positions[i].as_ref(),
                        light.position.x,
                        light.position.y,
                        light.position.z,
                    );
                    gl.uniform_3_f32(
                        variant.uniforms.point_light_colors[i].as_ref(),
                        light.color.x,
                        light.color.y,
                        light.color.z,
                    );
                    gl.uniform_1_f32(
                        variant.uniforms.point_light_intensities[i].as_ref(),
                        light.intensity,
                    );
                    gl.uniform_1_f32(
                        variant.uniforms.point_light_radii[i].as_ref(),
                        light.radius,
                    );
                }

                // SSAO enabled flag for main shader
                gl.uniform_1_i32(
                    variant.uniforms.ssao_enabled.as_ref(),
                    self.settings.ssao_enabled as i32,
                );

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
        self.ssao_target.resize(
            gl,
            width,
            height,
            glow::R16F as i32,
            glow::RED,
            glow::HALF_FLOAT,
        );
        self.ssao_blur_target.resize(
            gl,
            width,
            height,
            glow::R16F as i32,
            glow::RED,
            glow::HALF_FLOAT,
        );
    }

    pub fn delete(&self, gl: &Context) {
        self.fbo.delete(gl);
        self.ssao_target.delete(gl);
        self.ssao_blur_target.delete(gl);
        self.triangle.delete(gl);
        unsafe {
            gl.delete_program(self.ssao_program);
            gl.delete_program(self.ssao_blur_program);
            for variant in &self.variants {
                gl.delete_program(variant.program);
            }
        }
    }
}
