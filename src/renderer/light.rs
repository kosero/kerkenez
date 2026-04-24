use glam::Vec3;

pub const MAX_POINT_LIGHTS: usize = 32;

#[derive(Debug, Clone, Copy)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
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
        self.color = Vec3::new(r, g, b);
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
            color: Vec3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub radius: f32,
}

impl PointLight {
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            color: Vec3::ONE,
            intensity: 1.0,
            radius: 10.0,
        }
    }

    pub fn at(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = Vec3::new(x, y, z);
        self
    }

    pub fn color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = Vec3::new(r, g, b);
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
    pub ambient_color: Vec3,
    pub ambient_intensity: f32,
    pub directional: Option<DirectionalLight>,
    pub point_lights: Vec<PointLight>,
}

impl Default for SceneLights {
    fn default() -> Self {
        Self {
            ambient_color: Vec3::new(1.0, 1.0, 1.0),
            ambient_intensity: 0.0,
            directional: None,
            point_lights: Vec::new(),
        }
    }
}
