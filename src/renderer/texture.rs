use glow::{Context, HasContext, NativeTexture};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(pub usize);

pub struct Texture {
    pub id: NativeTexture,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn load(gl: &Context, path: &str) -> Self {
        let img = image::open(path)
            .unwrap_or_else(|e| panic!("Texture not loaded '{path}': {e}"))
            .flipv()
            .into_rgba8();
        let (width, height) = img.dimensions();
        let pixels = img.into_raw();

        Self::from_pixels(gl, width, height, &pixels)
    }

    pub fn white(gl: &Context) -> Self {
        let pixels = vec![255, 255, 255, 255];
        Self::from_pixels(gl, 1, 1, &pixels)
    }

    pub fn from_pixels(gl: &Context, width: u32, height: u32, pixels: &[u8]) -> Self {
        let id = unsafe {
            let texture = gl.create_texture().expect("Texture not created");
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
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::REPEAT as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::REPEAT as i32,
            );
            texture
        };

        Self { id, width, height }
    }

    /// Bind this texture to the given texture unit and set the sampler uniform.
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

    pub fn delete(&self, gl: &Context) {
        unsafe {
            gl.delete_texture(self.id);
        }
    }
}
