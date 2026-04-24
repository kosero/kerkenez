use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialId(pub usize);

pub struct Material {
    pub name: String,
    pub albedo_color: Vec3,
    pub texture_path: Option<String>,
}

impl Material {
    pub fn new(name: &str, color: Vec3, texture_path: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            albedo_color: color,
            texture_path: texture_path.map(|s| s.to_string()),
        }
    }
}
