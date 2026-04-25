mod directional_light;
mod point_light;

use crate::renderer::color::Color;
pub use directional_light::DirectionalLight;
pub use point_light::{MAX_POINT_LIGHTS, PointLight};

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
