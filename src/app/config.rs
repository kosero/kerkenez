pub struct Config {
    pub title: String,
    pub width: i32,
    pub height: i32,
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
