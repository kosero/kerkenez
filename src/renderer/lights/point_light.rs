use crate::renderer::color::Color;
use glam::Vec3;

pub const MAX_POINT_LIGHTS: usize = 32;

#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub radius: f32,
}

impl PointLight {
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            color: Color::WHITE,
            intensity: 1.0,
            radius: 10.0,
        }
    }

    pub fn at(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = Vec3::new(x, y, z);
        self
    }

    pub fn color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = Color::rgb(r, g, b).to_linear();
        self
    }

    pub fn color_srgb(mut self, color: Color) -> Self {
        self.color = color.to_linear();
        self
    }

    pub fn intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
}

impl Default for PointLight {
    fn default() -> Self {
        Self::new()
    }
}
