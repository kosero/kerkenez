use crate::renderer::{color::Color, texture::TextureId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialId(usize);

impl MaterialId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn index(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Material {
    pub name: &'static str,
    pub albedo_color: Color,
    pub texture_path: Option<String>,
    pub albedo_texture: Option<TextureId>,
}

impl Material {
    pub fn new(name: &'static str, color: Color, texture_path: Option<&str>) -> Self {
        Self {
            name,
            albedo_color: color.to_linear(),
            texture_path: texture_path.map(|s| s.to_string()),
            albedo_texture: None,
        }
    }
}
