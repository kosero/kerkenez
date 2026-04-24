use crate::error::EngineError;
use glow::{Context, HasContext, NativeTexture};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(usize);

impl TextureId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn index(&self) -> usize {
        self.0
    }
}

pub struct Texture {
    gl: Rc<Context>,
    pub id: NativeTexture,
    pub width: u32,
    pub height: u32,
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.id);
        }
    }
}

impl Texture {
    pub fn load(gl: &Rc<Context>, path: &str) -> Self {
        match image::open(path) {
            Ok(img) => {
                let img = img.flipv().into_rgba8();
                let (width, height) = img.dimensions();
                let pixels = img.into_raw();
                Self::from_pixels(gl, width, height, &pixels).unwrap_or_else(|_| {
                    eprintln!(
                        "Failed to create GPU texture for '{}'. Using fallback.",
                        path
                    );
                    Self::error_fallback(gl)
                })
            }
            Err(e) => {
                eprintln!(
                    "Failed to load texture '{}': {}. Using fallback magenta texture.",
                    path, e
                );
                Self::error_fallback(gl)
            }
        }
    }

    pub fn error_fallback(gl: &Rc<Context>) -> Self {
        let pixels = vec![255, 0, 255, 255]; // Magenta
        Self::from_pixels(gl, 1, 1, &pixels).unwrap_or_else(|_| {
            panic!("Critical: Failed to create fallback texture");
        })
    }

    pub fn white(gl: &Rc<Context>) -> Self {
        let pixels = vec![255, 255, 255, 255];
        Self::from_pixels(gl, 1, 1, &pixels).unwrap_or_else(|_| {
            panic!("Critical: Failed to create white texture");
        })
    }

    pub fn from_pixels(
        gl: &Rc<Context>,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<Self, EngineError> {
        let id = unsafe {
            let texture = gl
                .create_texture()
                .map_err(EngineError::ResourceCreationError)?;
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::SRGB8_ALPHA8 as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(pixels)),
            );
            gl.generate_mipmap(glow::TEXTURE_2D);
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
            texture
        };

        Ok(Self {
            gl: gl.clone(),
            id,
            width,
            height,
        })
    }

    /// Bind this texture to the given texture unit using a pre-cached uniform location.
    pub fn bind_at(&self, gl: &glow::Context, location: Option<&glow::UniformLocation>, unit: u32) {
        unsafe {
            gl.active_texture(glow::TEXTURE0 + unit);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.id));
            if let Some(loc) = location {
                gl.uniform_1_i32(Some(loc), unit as i32);
            }
        }
    }

    /// Bind this texture to the given texture unit and set the sampler uniform (legacy lookup).
    pub fn bind(&self, gl: &Context, program: glow::NativeProgram, unit: u32) {
        unsafe {
            gl.active_texture(glow::TEXTURE0 + unit);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.id));
            let tex_loc = gl.get_uniform_location(program, "u_Texture");
            gl.uniform_1_i32(tex_loc.as_ref(), unit as i32);
        }
    }

    pub fn unbind(&self, gl: &Context) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }
}
