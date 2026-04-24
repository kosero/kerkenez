pub mod config;

use self::config::Config;
use crate::renderer::RenderState;
use crate::renderer::draw_command::DrawCommand;
use crate::renderer::light::{DirectionalLight, PointLight, SceneLights};

use crate::renderer::material::{Material, MaterialId};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

pub struct App {
    config: Config,
    state: Option<RenderState>,
    render_queue: Vec<DrawCommand>,
    materials: Vec<(MaterialId, Material)>,
    next_material_id: usize,
    pub lights: SceneLights,
    pub post_process_settings: crate::renderer::post_process::settings::PostProcessSettings,
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
            next_material_id: 1,
            lights: SceneLights::default(),
            post_process_settings:
                crate::renderer::post_process::settings::PostProcessSettings::default(),
        }
    }

    pub fn set_ambient_light(&mut self, r: f32, g: f32, b: f32, intensity: f32) {
        self.lights.ambient_color = glam::vec3(r, g, b);
        self.lights.ambient_intensity = intensity;
    }

    pub fn set_directional_light(&mut self, light: DirectionalLight) {
        self.lights.directional = Some(light);
    }

    pub fn add_light(&mut self, light: PointLight) {
        self.lights.point_lights.push(light);
    }

    pub fn add_material(&mut self, material: Material) -> MaterialId {
        let id = MaterialId(self.next_material_id);
        self.next_material_id += 1;
        self.register_material(id, material);
        id
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

    pub fn draw(&mut self, command: DrawCommand) {
        self.render_queue.push(command);
    }

    pub fn camera_mut(&mut self) -> Option<&mut crate::camera::Camera> {
        self.state.as_mut().map(|s| &mut s.camera)
    }

    pub fn post_process_mut(
        &mut self,
    ) -> Option<&mut crate::renderer::post_process::PostProcessManager> {
        self.state.as_mut().map(|s| &mut s.post_process)
    }

    pub fn run<F>(&mut self, update: F)
    where
        F: FnMut(&mut App),
    {
        let event_loop = EventLoop::new().expect("Failed to create event loop");
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let mut runner = Runner { app: self, update };
        event_loop.run_app(&mut runner).unwrap();
    }
}

struct Runner<'a, F> {
    app: &'a mut App,
    update: F,
}

impl<'a, F: FnMut(&mut App)> ApplicationHandler for Runner<'a, F> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let vert_src = include_str!("../../shaders/vertex.vert");
        let frag_src = include_str!("../../shaders/fragment.frag");

        let mut state = RenderState::new(
            event_loop,
            &self.app.config.title,
            self.app.config.width,
            self.app.config.height,
            vert_src,
            frag_src,
        );

        // Register queued materials
        for (id, material) in self.app.materials.drain(..) {
            state.register_material(id, material);
        }

        state.post_process.settings = self.app.post_process_settings.clone();
        self.app.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(state) = self.app.state.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                state.render(&self.app.render_queue, &self.app.lights);
            }
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.app.state.is_some() {
            // Clear the render queue for the new frame
            self.app.render_queue.clear();
            
            // Run user update logic
            (self.update)(&mut self.app);
            
            // Request redraw with the new queue
            if let Some(state) = self.app.state.as_mut() {
                state.request_redraw();
            }
        }
    }
}
