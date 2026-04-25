use glam::Vec3;

use crate::renderer::color::Color;

#[derive(Debug, Clone, Copy)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Color,
    pub intensity: f32,
}

impl DirectionalLight {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn direction(mut self, x: f32, y: f32, z: f32) -> Self {
        self.direction = Vec3::new(x, y, z);
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
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vec3::new(-0.5, -1.0, -0.5),
            color: Color::WHITE,
            intensity: 1.0,
        }
    }
}
