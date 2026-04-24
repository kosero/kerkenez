use glam::Vec4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialId(pub usize);

pub struct Material {
    pub name: &'static str,
    pub albedo_color: Vec4,
    pub texture_path: Option<String>,
}

impl Material {
    pub fn new(name: &'static str, color: Vec4, texture_path: Option<&str>) -> Self {
        Self {
            name,
            albedo_color: color,
            texture_path: texture_path.map(|s| s.to_string()),
        }
    }
}
