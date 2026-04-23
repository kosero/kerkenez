pub mod config;

use self::config::Config;
use crate::{
    behaviour::{self, Behaviour},
    input::keyboard::KeyboardState,
    renderer::RenderState,
};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    window::WindowId,
};

pub struct App {
    config: Config,
    state: Option<RenderState>,
    keyboard: KeyboardState,
    behaviours: Vec<Box<dyn Behaviour>>,
    started: bool,
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
            keyboard: KeyboardState::new(),
            behaviours: Vec::new(),
            started: false,
        }
    }

    pub fn with_behaviour(mut self, behaviour: impl Behaviour + 'static) -> Self {
        self.behaviours.push(Box::new(behaviour));
        self
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

        // Generate 10k random instances
        let mut instances = Vec::with_capacity(10_000);
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for _ in 0..10_000 {
            let x = rng.gen_range(-50.0..50.0);
            let y = rng.gen_range(-50.0..50.0);
            let z = rng.gen_range(0.0..100.0);

            instances.push(crate::mesh::Instance {
                model_matrix: glam::Mat4::from_translation(glam::vec3(x, y, z))
                    * glam::Mat4::from_scale(glam::vec3(0.1, 0.1, 0.1)),
                color: glam::Vec3::ZERO,
            });
        }

        state.update_instances(&instances);
        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => {
                for b in self.behaviours.iter_mut() {
                    b.exit();
                }
                event_loop.exit()
            }
            WindowEvent::RedrawRequested => state.render(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(kc),
                        state,
                        ..
                    },
                ..
            } => {
                self.keyboard.process(kc, state);
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let ctx = behaviour::Context {
            keyboard: &self.keyboard,
        };

        if !self.started {
            for b in self.behaviours.iter_mut() {
                b.init(&ctx);
            }
            self.started = true;
        }

        for b in self.behaviours.iter_mut() {
            b.update(&ctx);
        }

        self.keyboard.end_frame();

        if let Some(state) = self.state.as_mut() {
            state.request_redraw();
        }
    }
}
