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
    render_queue: Vec<crate::mesh::RenderCommand>,
    materials: Vec<(
        crate::renderer::material::MaterialId,
        crate::renderer::material::Material,
    )>,
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
            render_queue: Vec::new(),
            materials: Vec::new(),
        }
    }

    pub fn register_material(
        &mut self,
        id: crate::renderer::material::MaterialId,
        material: crate::renderer::material::Material,
    ) {
        if let Some(state) = self.state.as_mut() {
            state.register_material(id, material);
        } else {
            self.materials.push((id, material));
        }
    }

    pub fn draw(&mut self, command: crate::mesh::RenderCommand) {
        self.render_queue.push(command);
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().expect("Failed to create event loop");
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop.run_app(&mut self).unwrap();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut state = RenderState::new(
            event_loop,
            &self.config.title,
            self.config.width,
            self.config.height,
        );

        // Register queued materials
        for (id, material) in self.materials.drain(..) {
            state.register_material(id, material);
        }

        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                state.render(&self.render_queue);
            }
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = self.state.as_mut() {
            state.request_redraw();
        }
    }
}
