pub mod fullscreen_pass;
pub mod render_target;
pub mod settings;

use self::fullscreen_pass::FullscreenPass;
use self::render_target::{GBuffer, RenderTarget};
use self::settings::PostProcessingSettings;
use crate::error::EngineError;
use crate::renderer::light::{MAX_POINT_LIGHTS, SceneLights};
use crate::renderer::shader;
use glow::{Context, HasContext};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ShaderDefines {
    debug_mode: i32,
    ssao_enabled: bool,
    fog_enabled: bool,
    tone_mapping_enabled: bool,
    vignette_enabled: bool,
}

impl ShaderDefines {
    fn from_settings(settings: &PostProcessingSettings) -> Self {
        Self {
            debug_mode: settings.debug_mode as i32,
            ssao_enabled: settings.ssao_enabled,
            fog_enabled: settings.fog_enabled,
            tone_mapping_enabled: settings.tone_mapping_enabled,
            vignette_enabled: settings.vignette_enabled,
        }
    }
}

struct UniformCache {
    screen_texture: Option<glow::UniformLocation>,
    depth_texture: Option<glow::UniformLocation>,
    normal_texture: Option<glow::UniformLocation>,
    near: Option<glow::UniformLocation>,
    far: Option<glow::UniformLocation>,
    inverse_vp: Option<glow::UniformLocation>,
    resolution: Option<glow::UniformLocation>,
    exposure: Option<glow::UniformLocation>,
    contrast: Option<glow::UniformLocation>,
    brightness: Option<glow::UniformLocation>,
    saturation: Option<glow::UniformLocation>,
    ssao_texture: Option<glow::UniformLocation>,
    vignette_intensity: Option<glow::UniformLocation>,
    vignette_radius: Option<glow::UniformLocation>,
    vignette_softness: Option<glow::UniformLocation>,
    time: Option<glow::UniformLocation>,

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
    ssao_samples: Option<glow::UniformLocation>,
}

struct SsaoBlurUniforms {
    ssao_texture: Option<glow::UniformLocation>,
    depth_texture: Option<glow::UniformLocation>,
    near: Option<glow::UniformLocation>,
    far: Option<glow::UniformLocation>,
    resolution: Option<glow::UniformLocation>,
    direction: Option<glow::UniformLocation>,
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
                exposure: gl.get_uniform_location(program, "u_Exposure"),
                contrast: gl.get_uniform_location(program, "u_Contrast"),
                brightness: gl.get_uniform_location(program, "u_Brightness"),
                saturation: gl.get_uniform_location(program, "u_Saturation"),
                ssao_texture: gl.get_uniform_location(program, "u_SSAOTexture"),
                vignette_intensity: gl.get_uniform_location(program, "u_VignetteIntensity"),
                vignette_radius: gl.get_uniform_location(program, "u_VignetteRadius"),
                vignette_softness: gl.get_uniform_location(program, "u_VignetteSoftness"),
                time: gl.get_uniform_location(program, "u_Time"),

                camera_pos: gl.get_uniform_location(program, "u_CameraPos"),
                ambient_color: gl.get_uniform_location(program, "u_AmbientColor"),
                ambient_intensity: gl.get_uniform_location(program, "u_AmbientIntensity"),
                dir_light_direction: gl.get_uniform_location(program, "u_DirLight.direction"),
                dir_light_color: gl.get_uniform_location(program, "u_DirLight.color"),
                dir_light_intensity: gl.get_uniform_location(program, "u_DirLight.intensity"),
                dir_light_enabled: gl.get_uniform_location(program, "u_DirLight.enabled"),
                point_lights_count: gl.get_uniform_location(program, "u_PointLightsCount"),

                point_light_positions: (0..MAX_POINT_LIGHTS)
                    .map(|i| {
                        gl.get_uniform_location(program, &format!("u_PointLights[{}].position", i))
                    })
                    .collect(),
                point_light_colors: (0..MAX_POINT_LIGHTS)
                    .map(|i| {
                        gl.get_uniform_location(program, &format!("u_PointLights[{}].color", i))
                    })
                    .collect(),
                point_light_intensities: (0..MAX_POINT_LIGHTS)
                    .map(|i| {
                        gl.get_uniform_location(program, &format!("u_PointLights[{}].intensity", i))
                    })
                    .collect(),
                point_light_radii: (0..MAX_POINT_LIGHTS)
                    .map(|i| {
                        gl.get_uniform_location(program, &format!("u_PointLights[{}].radius", i))
                    })
                    .collect(),
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
                ssao_samples: gl.get_uniform_location(program, "u_SSAOSamples"),
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
                direction: gl.get_uniform_location(program, "u_Direction"),
            }
        }
    }
}

struct ProgramVariant {
    program: glow::Program,
    uniforms: UniformCache,
}

pub struct PostProcessingManager {
    gl: Rc<Context>,
    fbo: GBuffer,
    ssao_target: RenderTarget,
    ssao_blur_target: RenderTarget,
    triangle: FullscreenPass,

    ssao_program: glow::Program,
    ssao_uniforms: SsaoUniforms,

    ssao_blur_program: glow::Program,
    ssao_blur_uniforms: SsaoBlurUniforms,

    variants: HashMap<ShaderDefines, ProgramVariant>,
    pub settings: PostProcessingSettings,
}

impl Drop for PostProcessingManager {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.ssao_program);
            self.gl.delete_program(self.ssao_blur_program);
            for variant in self.variants.values() {
                self.gl.delete_program(variant.program);
            }
        }
    }
}

impl PostProcessingManager {
    pub fn new(gl: &Rc<Context>, width: i32, height: i32) -> Result<Self, EngineError> {
        let fbo = GBuffer::new(gl, width, height)?;
        // SSAO at half resolution for performance
        let half_w = (width / 2).max(1);
        let half_h = (height / 2).max(1);

        let ssao_target = RenderTarget::new(
            gl,
            half_w,
            half_h,
            glow::R16F as i32,
            glow::RED,
            glow::HALF_FLOAT,
        )?;
        let ssao_blur_target = RenderTarget::new(
            gl,
            half_w,
            half_h,
            glow::R16F as i32,
            glow::RED,
            glow::HALF_FLOAT,
        )?;
        let triangle = FullscreenPass::new(gl)?;

        let vert_src = include_str!("../../../shaders/fullscreen.vert");
        let ssao_frag_src = include_str!("../../../shaders/ssao.frag");
        let ssao_blur_frag_src = include_str!("../../../shaders/ssao_blur.frag");

        let common_src = include_str!("../../../shaders/common.glsl");
        let inject_common = |src: &str| {
            let version = "#version 410 core\n";
            if let Some(stripped) = src.strip_prefix(version) {
                format!("{}{}\n{}", version, common_src, stripped)
            } else {
                format!("{}\n{}", common_src, src)
            }
        };

        let ssao_frag_final = inject_common(ssao_frag_src);
        let ssao_blur_frag_final = inject_common(ssao_blur_frag_src);

        let (ssao_program, ssao_blur_program) = unsafe {
            let sp = gl
                .create_program()
                .map_err(EngineError::ResourceCreationError)?;
            shader::create_shaders(gl, sp, vert_src, &ssao_frag_final)?;

            let sbp = gl
                .create_program()
                .map_err(EngineError::ResourceCreationError)?;
            shader::create_shaders(gl, sbp, vert_src, &ssao_blur_frag_final)?;

            (sp, sbp)
        };

        let ssao_uniforms = SsaoUniforms::new(gl, ssao_program);
        let ssao_blur_uniforms = SsaoBlurUniforms::new(gl, ssao_blur_program);

        Ok(Self {
            gl: gl.clone(),
            fbo,
            ssao_target,
            ssao_blur_target,
            triangle,
            ssao_program,
            ssao_uniforms,
            ssao_blur_program,
            ssao_blur_uniforms,
            variants: HashMap::new(),
            settings: PostProcessingSettings::default(),
        })
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

    fn get_variant(&mut self, gl: &Context) -> Result<&ProgramVariant, EngineError> {
        let defines = ShaderDefines::from_settings(&self.settings);

        if let std::collections::hash_map::Entry::Vacant(e) = self.variants.entry(defines) {
            let vert_src = include_str!("../../../shaders/fullscreen.vert");
            let frag_src = include_str!("../../../shaders/composite.frag");
            let common_src = include_str!("../../../shaders/common.glsl");

            let mut define_str = format!("#define DEBUG_MODE {}\n", defines.debug_mode);
            if defines.ssao_enabled {
                define_str.push_str("#define ENABLE_SSAO\n");
            }
            if defines.fog_enabled {
                define_str.push_str("#define ENABLE_FOG\n");
            }
            if defines.tone_mapping_enabled {
                define_str.push_str("#define ENABLE_TONEMAP\n");
            }
            if defines.vignette_enabled {
                define_str.push_str("#define ENABLE_VIGNETTE\n");
            }

            let version_pragma = "#version 410 core\n";
            let frag_src_modified = if let Some(stripped) = frag_src.strip_prefix(version_pragma) {
                format!("{version_pragma}{define_str}{common_src}\n{stripped}")
            } else {
                format!("{define_str}{common_src}\n{frag_src}")
            };

            unsafe {
                let program = gl
                    .create_program()
                    .map_err(EngineError::ResourceCreationError)?;
                shader::create_shaders(gl, program, vert_src, &frag_src_modified)?;
                let uniforms = UniformCache::new(gl, program);
                e.insert(ProgramVariant { program, uniforms });
            }
        }

        Ok(self.variants.get(&defines).unwrap())
    }

    /// Unbind the G-Buffer FBO and draw the fullscreen pass with all post-processing.
    pub fn end(
        &mut self,
        gl: &Context,
        window_width: i32,
        window_height: i32,
        camera: &crate::camera::Camera,
        lights: &SceneLights,
        time: f32,
    ) {
        if self.settings.enabled {
            self.fbo.unbind(gl);

            let (near, far) = match camera.projection() {
                crate::camera::CameraProjection::Perspective(p) => (p.near, p.far),
                crate::camera::CameraProjection::Orthographic(o) => (o.near, o.far),
            };
            let inv_vp = camera.view_projection_matrix().inverse();
            let camera_pos = camera.position();

            // 1. SSAO Pass
            if self.settings.ssao_enabled {
                unsafe {
                    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.ssao_target.fbo));
                    gl.viewport(0, 0, self.ssao_target.width, self.ssao_target.height);
                    gl.clear_color(1.0, 1.0, 1.0, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT);

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
                        self.ssao_target.width as f32,
                        self.ssao_target.height as f32,
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
                    gl.uniform_1_i32(
                        self.ssao_uniforms.ssao_samples.as_ref(),
                        self.settings.ssao_sample_count,
                    );

                    gl.disable(glow::DEPTH_TEST);
                    self.triangle.draw(gl);

                    // 2. SSAO Blur Pass - Horizontal
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
                        self.ssao_blur_target.width as f32,
                        self.ssao_blur_target.height as f32,
                    );
                    gl.uniform_1_f32(self.ssao_blur_uniforms.near.as_ref(), near);
                    gl.uniform_1_f32(self.ssao_blur_uniforms.far.as_ref(), far);

                    // Horizontal Pass
                    gl.uniform_2_f32(self.ssao_blur_uniforms.direction.as_ref(), 1.0, 0.0);
                    self.triangle.draw(gl);

                    // 3. SSAO Blur Pass - Vertical
                    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.ssao_target.fbo));
                    gl.active_texture(glow::TEXTURE0);
                    gl.bind_texture(glow::TEXTURE_2D, Some(self.ssao_blur_target.color_texture));

                    // Vertical Pass
                    gl.uniform_2_f32(self.ssao_blur_uniforms.direction.as_ref(), 0.0, 1.0);
                    self.triangle.draw(gl);
                }
            }

            // 3. Final Compose Pass
            let settings = self.settings.clone();
            let albedo_tex = self.fbo.color_texture;
            let depth_tex = self.fbo.depth_texture;
            let normal_tex = self.fbo.normal_texture;
            let ssao_blur_tex = self.ssao_target.color_texture;

            let variant = match self.get_variant(gl) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!(
                        "Failed to get post-process variant: {}. Skipping compose pass.",
                        e
                    );
                    return;
                }
            };
            let program = variant.program;

            unsafe {
                gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                gl.viewport(0, 0, window_width, window_height);
                gl.clear_color(0.0, 0.0, 0.0, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT);

                gl.use_program(Some(program));

                gl.active_texture(glow::TEXTURE0);
                gl.bind_texture(glow::TEXTURE_2D, Some(albedo_tex));
                gl.uniform_1_i32(variant.uniforms.screen_texture.as_ref(), 0);

                gl.active_texture(glow::TEXTURE1);
                gl.bind_texture(glow::TEXTURE_2D, Some(depth_tex));
                gl.uniform_1_i32(variant.uniforms.depth_texture.as_ref(), 1);

                gl.active_texture(glow::TEXTURE2);
                gl.bind_texture(glow::TEXTURE_2D, Some(normal_tex));
                gl.uniform_1_i32(variant.uniforms.normal_texture.as_ref(), 2);

                if settings.ssao_enabled {
                    gl.active_texture(glow::TEXTURE3);
                    gl.bind_texture(glow::TEXTURE_2D, Some(ssao_blur_tex));
                    gl.uniform_1_i32(variant.uniforms.ssao_texture.as_ref(), 3);
                }

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

                gl.uniform_3_f32(
                    variant.uniforms.ambient_color.as_ref(),
                    lights.ambient_color.r,
                    lights.ambient_color.g,
                    lights.ambient_color.b,
                );
                gl.uniform_1_f32(
                    variant.uniforms.ambient_intensity.as_ref(),
                    lights.ambient_intensity,
                );

                if let Some(dir) = &lights.directional {
                    gl.uniform_3_f32(
                        variant.uniforms.dir_light_direction.as_ref(),
                        dir.direction.x,
                        dir.direction.y,
                        dir.direction.z,
                    );
                    gl.uniform_3_f32(
                        variant.uniforms.dir_light_color.as_ref(),
                        dir.color.r,
                        dir.color.g,
                        dir.color.b,
                    );
                    gl.uniform_1_f32(variant.uniforms.dir_light_intensity.as_ref(), dir.intensity);
                    gl.uniform_1_i32(variant.uniforms.dir_light_enabled.as_ref(), 1);
                } else {
                    gl.uniform_1_i32(variant.uniforms.dir_light_enabled.as_ref(), 0);
                }

                gl.uniform_1_i32(
                    variant.uniforms.point_lights_count.as_ref(),
                    lights.point_lights.len() as i32,
                );
                for (i, light) in lights.point_lights.iter().enumerate() {
                    if i >= MAX_POINT_LIGHTS {
                        break;
                    }
                    gl.uniform_3_f32(
                        variant.uniforms.point_light_positions[i].as_ref(),
                        light.position.x,
                        light.position.y,
                        light.position.z,
                    );
                    gl.uniform_3_f32(
                        variant.uniforms.point_light_colors[i].as_ref(),
                        light.color.r,
                        light.color.g,
                        light.color.b,
                    );
                    gl.uniform_1_f32(
                        variant.uniforms.point_light_intensities[i].as_ref(),
                        light.intensity,
                    );
                    gl.uniform_1_f32(variant.uniforms.point_light_radii[i].as_ref(), light.radius);
                }
                gl.uniform_1_f32(variant.uniforms.exposure.as_ref(), settings.exposure);
                gl.uniform_1_f32(variant.uniforms.contrast.as_ref(), settings.contrast);
                gl.uniform_1_f32(variant.uniforms.brightness.as_ref(), settings.brightness);
                gl.uniform_1_f32(variant.uniforms.saturation.as_ref(), settings.saturation);
                gl.uniform_1_f32(variant.uniforms.time.as_ref(), time);

                if settings.fog_enabled {
                    gl.uniform_1_f32(
                        gl.get_uniform_location(program, "u_FogDensity").as_ref(),
                        settings.fog_density,
                    );
                    gl.uniform_3_f32(
                        gl.get_uniform_location(program, "u_FogColor").as_ref(),
                        settings.fog_color.r,
                        settings.fog_color.g,
                        settings.fog_color.b,
                    );
                }

                if settings.vignette_enabled {
                    gl.uniform_1_f32(
                        variant.uniforms.vignette_intensity.as_ref(),
                        settings.vignette_intensity,
                    );
                    gl.uniform_1_f32(
                        variant.uniforms.vignette_radius.as_ref(),
                        settings.vignette_radius,
                    );
                    gl.uniform_1_f32(
                        variant.uniforms.vignette_softness.as_ref(),
                        settings.vignette_softness,
                    );
                }

                gl.disable(glow::DEPTH_TEST);
                self.triangle.draw(gl);
                gl.enable(glow::DEPTH_TEST);
            }
        }
    }

    pub fn resize(&mut self, gl: &Context, width: i32, height: i32) {
        self.fbo.resize(gl, width, height);
        let half_w = (width / 2).max(1);
        let half_h = (height / 2).max(1);
        self.ssao_target.resize(
            gl,
            half_w,
            half_h,
            glow::R16F as i32,
            glow::RED,
            glow::HALF_FLOAT,
        );
        self.ssao_blur_target.resize(
            gl,
            half_w,
            half_h,
            glow::R16F as i32,
            glow::RED,
            glow::HALF_FLOAT,
        );
    }
}
