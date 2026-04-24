#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebugMode {
    None = 0,
    Depth = 1,
    Normals = 2,
    SSAO = 3,
}

#[derive(Debug, Clone)]
pub struct PostProcessSettings {
    pub enabled: bool,
    pub debug_mode: DebugMode,

    // SSAO
    pub ssao_enabled: bool,
    pub ssao_radius: f32,
    pub ssao_intensity: f32,
    pub ssao_bias: f32,

    // Fog
    pub fog_enabled: bool,
    pub fog_density: f32,
    pub fog_color: [f32; 3],

    // Tone Mapping & Color Grading
    pub tone_mapping_enabled: bool,
    pub exposure: f32,
    pub contrast: f32,
    pub brightness: f32,
    pub saturation: f32,

    // Vignette
    pub vignette_enabled: bool,
    pub vignette_intensity: f32,
}

impl Default for PostProcessSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            debug_mode: DebugMode::None,

            ssao_enabled: true,
            ssao_radius: 0.9,
            ssao_intensity: 1.0,
            ssao_bias: 0.015,

            fog_enabled: true,
            fog_density: 0.02,
            fog_color: [0.1, 0.1, 0.1],

            tone_mapping_enabled: true,
            exposure: 1.0,
            contrast: 1.0,
            brightness: 0.0,
            saturation: 1.0,

            vignette_enabled: false,
            vignette_intensity: 0.75,
        }
    }
}
