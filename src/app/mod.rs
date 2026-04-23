pub mod config;

use self::config::Config;
use crate::renderer::RenderState;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

pub struct App {
    config: Config,
    state: Option<RenderState>,
}

impl App {
    pub fn new(title: &str, width: i32, height: i32) -> Self {
        Self {
            config: Config {
                title: title.to_string(),
                width,
                height,
            },
            state: None,
        }
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().expect("Failed to create event loop");
        event_loop.run_app(&mut self).unwrap();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.state = Some(RenderState::new(
            event_loop,
            &self.config.title,
            self.config.width,
            self.config.height,
        ));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => state.render(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            _ => (),
        }
    }
}
