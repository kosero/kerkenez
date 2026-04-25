use crate::error::KerkenezError;
use glow::{Context, HasContext, PixelUnpackData};
use std::rc::Rc;

pub struct RenderTarget {
    gl: Rc<Context>,
    pub fbo: glow::Framebuffer,
    pub color_texture: glow::Texture,
    pub width: i32,
    pub height: i32,
}

impl RenderTarget {
    pub fn new(
        gl: &Rc<Context>,
        width: u32,
        height: u32,
        internal_format: i32,
        format: u32,
        data_type: u32,
    ) -> Result<Self, KerkenezError> {
        let (width, height) = (width as i32, height as i32);
        unsafe {
            let fbo = gl
                .create_framebuffer()
                .map_err(KerkenezError::ResourceCreationError)?;
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));

            let color_texture = gl
                .create_texture()
                .map_err(KerkenezError::ResourceCreationError)?;
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
                return Err(KerkenezError::FramebufferIncomplete(
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
            self.gl
                .bind_texture(glow::TEXTURE_2D, Some(self.color_texture));
            self.gl.tex_image_2d(
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

impl Drop for RenderTarget {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_framebuffer(self.fbo);
            self.gl.delete_texture(self.color_texture);
        }
    }
}
