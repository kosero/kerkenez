use glam::Vec3;
use crate::renderer::color::Color;

pub const MAX_POINT_LIGHTS: usize = 32;

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

#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub radius: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        Self::new()
    }
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

#[derive(Debug, Clone)]
pub struct SceneLights {
    pub ambient_color: Color,
    pub ambient_intensity: f32,
    pub directional: Option<DirectionalLight>,
    pub point_lights: Vec<PointLight>,
}

impl Default for SceneLights {
    fn default() -> Self {
        Self {
            ambient_color: Color::WHITE,
            ambient_intensity: 0.0,
            directional: None,
            point_lights: Vec::new(),
        }
    }
}
