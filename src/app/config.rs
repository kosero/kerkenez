pub struct Config {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: "Window Title".to_string(),
            width: 800,
            height: 600,
        }
    }
}
