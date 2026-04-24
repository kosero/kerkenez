use crate::error::EngineError;
use glow::{Context, HasContext, PixelUnpackData};
use std::rc::Rc;

/// G-Buffer backed framebuffer with Multiple Render Targets (MRT).
/// Attachments:
/// - COLOR_ATTACHMENT0: Albedo (RGBA16F)
/// - COLOR_ATTACHMENT1: World-space Normal (RGBA16F)
/// - DEPTH_ATTACHMENT:  Depth (DEPTH_COMPONENT32F)
pub struct GBuffer {
    gl: Rc<Context>,
    pub fbo: glow::Framebuffer,
    pub color_texture: glow::Texture,
    pub normal_texture: glow::Texture,
    pub depth_texture: glow::Texture,
    pub width: i32,
    pub height: i32,
}

impl Drop for GBuffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_framebuffer(self.fbo);
            self.gl.delete_texture(self.color_texture);
            self.gl.delete_texture(self.normal_texture);
            self.gl.delete_texture(self.depth_texture);
        }
    }
}

impl GBuffer {
    pub fn new(gl: &Rc<Context>, width: i32, height: i32) -> Result<Self, EngineError> {
        unsafe {
            let fbo = gl
                .create_framebuffer()
                .map_err(EngineError::ResourceCreationError)?;
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));

            // --- RT0: HDR Albedo (RGBA16F) ---
            let color_texture = gl
                .create_texture()
                .map_err(EngineError::ResourceCreationError)?;
            gl.bind_texture(glow::TEXTURE_2D, Some(color_texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA16F as i32,
                width,
                height,
                0,
                glow::RGBA,
                glow::HALF_FLOAT,
                PixelUnpackData::Slice(None),
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(color_texture),
                0,
            );

            // --- RT1: World-space Normal (RGBA16F) ---
            let normal_texture = gl
                .create_texture()
                .map_err(EngineError::ResourceCreationError)?;
            gl.bind_texture(glow::TEXTURE_2D, Some(normal_texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA16F as i32,
                width,
                height,
                0,
                glow::RGBA,
                glow::HALF_FLOAT,
                PixelUnpackData::Slice(None),
            );
            // NEAREST filtering for G-Buffer normals — no cross-surface interpolation
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT1,
                glow::TEXTURE_2D,
                Some(normal_texture),
                0,
            );

            // --- Depth attachment (DEPTH_COMPONENT32F) ---
            let depth_texture = gl
                .create_texture()
                .map_err(EngineError::ResourceCreationError)?;
            gl.bind_texture(glow::TEXTURE_2D, Some(depth_texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::DEPTH_COMPONENT32F as i32,
                width,
                height,
                0,
                glow::DEPTH_COMPONENT,
                glow::FLOAT,
                PixelUnpackData::Slice(None),
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::DEPTH_ATTACHMENT,
                glow::TEXTURE_2D,
                Some(depth_texture),
                0,
            );

            // Enable MRT: tell OpenGL we're writing to two color attachments
            gl.draw_buffers(&[glow::COLOR_ATTACHMENT0, glow::COLOR_ATTACHMENT1]);

            if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
                return Err(EngineError::FramebufferIncomplete(
                    "G-Buffer Framebuffer is not complete!".to_string(),
                ));
            }

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            Ok(Self {
                gl: gl.clone(),
                fbo,
                color_texture,
                normal_texture,
                depth_texture,
                width,
                height,
            })
        }
    }

    pub fn bind(&self, gl: &Context) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));
            gl.viewport(0, 0, self.width, self.height);
            // Re-assert MRT draw buffers (safety: FBO state should persist, but
            // other code might change it via shared GL context)
            gl.draw_buffers(&[glow::COLOR_ATTACHMENT0, glow::COLOR_ATTACHMENT1]);
            gl.clear_depth_f32(1.0);
        }
    }

    pub fn unbind(&self, gl: &Context) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }

    pub fn resize(&mut self, gl: &Context, width: i32, height: i32) {
        if self.width == width && self.height == height {
            return;
        }
        self.width = width;
        self.height = height;

        unsafe {
            // Resize albedo texture
            gl.bind_texture(glow::TEXTURE_2D, Some(self.color_texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA16F as i32,
                width,
                height,
                0,
                glow::RGBA,
                glow::HALF_FLOAT,
                PixelUnpackData::Slice(None),
            );

            // Resize normal texture
            gl.bind_texture(glow::TEXTURE_2D, Some(self.normal_texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA16F as i32,
                width,
                height,
                0,
                glow::RGBA,
                glow::HALF_FLOAT,
                PixelUnpackData::Slice(None),
            );

            // Resize depth texture
            gl.bind_texture(glow::TEXTURE_2D, Some(self.depth_texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::DEPTH_COMPONENT32F as i32,
                width,
                height,
                0,
                glow::DEPTH_COMPONENT,
                glow::FLOAT,
                PixelUnpackData::Slice(None),
            );
        }
    }
}

pub struct RenderTarget {
    gl: Rc<Context>,
    pub fbo: glow::Framebuffer,
    pub color_texture: glow::Texture,
    pub width: i32,
    pub height: i32,
}

impl Drop for RenderTarget {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_framebuffer(self.fbo);
            self.gl.delete_texture(self.color_texture);
        }
    }
}

impl RenderTarget {
    pub fn new(
        gl: &Rc<Context>,
        width: i32,
        height: i32,
        internal_format: i32,
        format: u32,
        data_type: u32,
    ) -> Result<Self, EngineError> {
        unsafe {
            let fbo = gl
                .create_framebuffer()
                .map_err(EngineError::ResourceCreationError)?;
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));

            let color_texture = gl
                .create_texture()
                .map_err(EngineError::ResourceCreationError)?;
            gl.bind_texture(glow::TEXTURE_2D, Some(color_texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                internal_format,
                width,
                height,
                0,
                format,
                data_type,
                PixelUnpackData::Slice(None),
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );

            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(color_texture),
                0,
            );

            if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
                return Err(EngineError::FramebufferIncomplete(
                    "RenderTarget Framebuffer is not complete!".to_string(),
                ));
            }

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            Ok(Self {
                gl: gl.clone(),
                fbo,
                color_texture,
                width,
                height,
            })
        }
    }

    pub fn resize(
        &mut self,
        gl: &Context,
        width: i32,
        height: i32,
        internal_format: i32,
        format: u32,
        data_type: u32,
    ) {
        if self.width == width && self.height == height {
            return;
        }
        self.width = width;
        self.height = height;

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.color_texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                internal_format,
                width,
                height,
                0,
                format,
                data_type,
                PixelUnpackData::Slice(None),
            );
        }
    }
}
