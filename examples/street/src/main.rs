use glow::HasContext;
use glutin::{
    config::{ConfigTemplateBuilder, GlConfig},
    context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext},
    display::GetGlDisplay,
    prelude::{GlDisplay, GlSurface, NotCurrentGlContext},
    surface::{Surface, SwapInterval, WindowSurface},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use kerkenez::prelude::*;
use raw_window_handle::HasWindowHandle;
use std::rc::Rc;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

// fn rotate_point(x: f32, y: f32, z: f32, angle: f32) -> (f32, f32, f32) {
//     let cos_a = angle.cos();
//     let sin_a = angle.sin();
//     (x * cos_a + z * sin_a, y, -x * sin_a + z * cos_a)
// }

struct StreetApp {
    renderer: Option<Renderer>,
    gl_surface: Option<Surface<WindowSurface>>,
    gl_context: Option<PossiblyCurrentContext>,
    window: Option<Window>,
    start_time: Instant,
}

impl ApplicationHandler for StreetApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let title = "Street";
        let width = 1280;
        let height = 720;

        let window_attrs = Window::default_attributes()
            .with_title(title)
            .with_inner_size(LogicalSize::new(width, height));

        let template = ConfigTemplateBuilder::new().with_depth_size(24);
        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attrs));

        let (window, gl_config) = display_builder
            .build(event_loop, template, |configs| {
                configs
                    .reduce(|a, b| {
                        if b.num_samples() > a.num_samples() {
                            b
                        } else {
                            a
                        }
                    })
                    .unwrap()
            })
            .unwrap();

        let window = window.expect("Failed to create window");
        let raw_window_handle = window.window_handle().ok().map(|h| h.as_raw());

        let gl_display = gl_config.display();
        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(glutin::context::Version {
                major: 4,
                minor: 5,
            })))
            .build(raw_window_handle);

        let (gl, gl_surface, gl_context) = unsafe {
            let not_current = gl_display
                .create_context(&gl_config, &context_attributes)
                .expect("Failed to create OpenGL context");

            let attrs = window
                .build_surface_attributes(Default::default())
                .expect("Failed to build surface attributes");

            let gl_surface = gl_display
                .create_window_surface(&gl_config, &attrs)
                .expect("Failed to create window surface");

            let gl_context = not_current
                .make_current(&gl_surface)
                .expect("Failed to make context current");

            let gl = Rc::new(glow::Context::from_loader_function_cstr(|s| {
                gl_display.get_proc_address(s)
            }));

            gl_surface
                .set_swap_interval(&gl_context, SwapInterval::DontWait)
                .ok();

            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);

            (gl, gl_surface, gl_context)
        };

        let mut renderer = Renderer::new(gl.clone(), width, height).unwrap();

        renderer.set_ambient_light(0.1, 0.1, 0.1, 0.1);
        renderer.set_directional_light(
            DirectionalLight::new()
                .direction(0.5, -0.8, -0.2)
                .intensity(0.3),
        );

        self.renderer = Some(renderer);
        self.gl_surface = Some(gl_surface);
        self.gl_context = Some(gl_context);
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let (Some(renderer), Some(gl_surface), Some(gl_context), Some(_window)) = (
            self.renderer.as_mut(),
            self.gl_surface.as_ref(),
            self.gl_context.as_ref(),
            self.window.as_ref(),
        ) else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    gl_surface.resize(
                        gl_context,
                        size.width.try_into().unwrap(),
                        size.height.try_into().unwrap(),
                    );
                    renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                let time = self.start_time.elapsed().as_secs_f32();
                // let y_cord = -20f32.to_radians();

                renderer.begin_drawing();

                // Road
                renderer.draw(
                    DrawCommand::cube()
                        .at(0.0, -2.0, -10.0)
                        .scale_xyz(12.0, 0.1, 80.0)
                        .tint(0.15, 0.15, 0.15, 1.0),
                );

                // Road markings (dashed line)
                for i in 0..20 {
                    let z = 5.0 - (i as f32) * 4.0;
                    renderer.draw(
                        DrawCommand::cube()
                            .at(0.0, -1.94, z)
                            .scale_xyz(0.2, 0.05, 2.0)
                            .tint(0.9, 0.9, 0.9, 1.0),
                    );
                }

                // Sidewalks
                renderer.draw(
                    DrawCommand::cube()
                        .at(-7.5, -1.9, -10.0)
                        .scale_xyz(3.0, 0.2, 80.0)
                        .tint(0.3, 0.3, 0.3, 1.0),
                );
                renderer.draw(
                    DrawCommand::cube()
                        .at(7.5, -1.9, -10.0)
                        .scale_xyz(3.0, 0.2, 80.0)
                        .tint(0.3, 0.3, 0.3, 1.0),
                );

                // Buildings
                let building_colors = [
                    (0.6, 0.3, 0.2), // Brick red
                    (0.8, 0.8, 0.7), // Beige
                    (0.3, 0.4, 0.5), // Blueish
                    (0.7, 0.6, 0.5), // Brown
                    (0.9, 0.9, 0.9), // White
                ];

                for side in 0..2 {
                    let sign = if side == 0 { -1.0 } else { 1.0 };
                    let x_pos = sign * 11.5;

                    for i in 0..12 {
                        let z = 5.0 - (i as f32) * 6.0;
                        let color_idx = (i + side) % building_colors.len();
                        let (r, g, b) = building_colors[color_idx];

                        // Randomize height a bit
                        let height = 10.0 + ((i * 3) % 5) as f32 * 2.5;

                        // Building main block
                        renderer.draw(
                            DrawCommand::cube()
                                .at(x_pos, -1.8 + height / 2.0, z)
                                .scale_xyz(5.0, height, 5.0)
                                .tint(r, g, b, 1.0),
                        );

                        // Windows
                        let floors = (height / 2.5) as i32;
                        for f in 1..floors {
                            let y = -1.8 + (f as f32) * 2.5;
                            for w in 0..2 {
                                let wx = x_pos - sign * 2.5; // slightly protruding to show glass
                                let wz = z - 1.5 + (w as f32) * 3.0;

                                // Window glass
                                renderer.draw(
                                    DrawCommand::cube()
                                        .at(wx, y, wz)
                                        .scale_xyz(0.1, 1.2, 1.0)
                                        .tint(0.5, 0.8, 1.0, 1.0), // Light blue glass
                                );

                                // Balcony (only some floors)
                                if (f + i as i32) % 2 == 0 {
                                    renderer.draw(
                                        DrawCommand::cube()
                                            .at(wx - sign * 0.6, y - 0.6, wz)
                                            .scale_xyz(1.4, 0.2, 1.6)
                                            .tint(0.2, 0.2, 0.2, 1.0),
                                    );
                                }
                            }
                        }

                        // Shop at ground floor
                        renderer.draw(
                            DrawCommand::cube()
                                .at(x_pos - sign * 2.5, -0.5, z)
                                .scale_xyz(0.2, 2.0, 4.0)
                                .tint(0.05, 0.05, 0.05, 1.0), // Dark glass front
                        );
                    }
                }

                // A car
                // Body
                renderer.draw(
                    DrawCommand::cube()
                        .at(2.5, -1.3, -2.0)
                        .scale_xyz(2.0, 0.8, 4.2)
                        .tint(0.8, 0.1, 0.1, 1.0), // Red car
                );
                // Cabin
                renderer.draw(
                    DrawCommand::cube()
                        .at(2.5, -0.5, -2.2)
                        .scale_xyz(1.8, 0.8, 2.0)
                        .tint(0.1, 0.1, 0.1, 1.0), // Black windows
                );
                // Wheels
                for wx in [-1.0, 1.0].iter() {
                    for wz in [-1.5, 1.5].iter() {
                        renderer.draw(
                            DrawCommand::cube()
                                .at(2.5 + wx * 1.0, -1.6, -2.0 + wz)
                                .scale_xyz(0.4, 0.6, 0.6)
                                .tint(0.05, 0.05, 0.05, 1.0),
                        );
                    }
                }

                // A truck on the other side
                renderer.draw(
                    DrawCommand::cube()
                        .at(-2.5, -0.5, -12.0)
                        .scale_xyz(2.5, 2.5, 6.0)
                        .tint(0.2, 0.4, 0.8, 1.0), // Blue truck
                );
                renderer.draw(
                    DrawCommand::cube()
                        .at(-2.5, -0.8, -8.0)
                        .scale_xyz(2.5, 1.8, 2.0)
                        .tint(0.9, 0.9, 0.9, 1.0), // White cabin
                );

                // Street lights
                for i in 0..6 {
                    let z = 2.0 - (i as f32) * 12.0;
                    for side in [-1.0, 1.0].iter() {
                        let x = side * 5.5;
                        // Pole
                        renderer.draw(
                            DrawCommand::cube()
                                .at(x, 1.5, z)
                                .scale_xyz(0.2, 7.0, 0.2)
                                .tint(0.2, 0.2, 0.2, 1.0),
                        );
                        // Arm
                        renderer.draw(
                            DrawCommand::cube()
                                .at(x - side * 1.0, 4.9, z)
                                .scale_xyz(2.0, 0.15, 0.2)
                                .tint(0.2, 0.2, 0.2, 1.0),
                        );
                        // Lamp
                        renderer.draw(
                            DrawCommand::cube()
                                .at(x - side * 1.8, 4.8, z)
                                .scale_xyz(0.4, 0.1, 0.4)
                                .tint(1.0, 1.0, 0.6, 1.0), // Yellow light
                        );
                    }
                }

                // Some trees (cubist style) on the sidewalk
                for i in 0..8 {
                    let z = 0.0 - (i as f32) * 9.0;
                    for side in [-1.0, 1.0].iter() {
                        let x = side * 7.5;
                        // Trunk
                        renderer.draw(
                            DrawCommand::cube()
                                .at(x, -1.0, z)
                                .scale_xyz(0.3, 2.0, 0.3)
                                .tint(0.4, 0.2, 0.1, 1.0),
                        );
                        // Leaves
                        renderer.draw(
                            DrawCommand::cube()
                                .at(x, 0.5, z)
                                .scale_xyz(1.5, 1.5, 1.5)
                                .tint(0.2, 0.6, 0.2, 1.0),
                        );
                        renderer.draw(
                            DrawCommand::cube()
                                .at(x, 1.5, z)
                                .scale_xyz(1.0, 1.0, 1.0)
                                .tint(0.3, 0.7, 0.3, 1.0),
                        );
                    }
                }

                // // body
                // renderer.draw(
                //     DrawCommand::cube()
                //         .scale_xyz(2.5, 2.5, 1.0)
                //         .tint(1.0, 1.0, 0.0, 1.0)
                //         .rotate(0.0, y_cord, 0.0),
                // );
                // // legs
                // let (lx, ly, lz) = rotate_point(-0.5, -1.5, 0.0, y_cord);
                // renderer.draw(
                //     DrawCommand::cube()
                //         .scale_xyz(0.5, 1.0, 1.0)
                //         .tint(1.0, 1.0, 0.0, 1.0)
                //         .at(lx, ly, lz)
                //         .rotate(0.0, y_cord, 0.0),
                // );
                // let (rx, ry, rz) = rotate_point(0.5, -1.5, 0.0, y_cord);
                // renderer.draw(
                //     DrawCommand::cube()
                //         .scale_xyz(0.5, 1.0, 1.0)
                //         .tint(1.0, 1.0, 0.0, 1.0)
                //         .at(rx, ry, rz)
                //         .rotate(0.0, y_cord, 0.0),
                // );
                // // arms
                // let (ax, ay, az) = rotate_point(1.5, 0.0, 0.0, y_cord);
                // renderer.draw(
                //     DrawCommand::cube()
                //         .scale_xyz(0.5, 1.5, 1.0)
                //         .tint(1.0, 1.0, 0.0, 1.0)
                //         .at(ax, ay, az)
                //         .rotate(0.0, y_cord, 45f32.to_radians()),
                // );
                // let (ax2, ay2, az2) = rotate_point(-1.5, 0.0, 0.0, y_cord);
                // renderer.draw(
                //     DrawCommand::cube()
                //         .scale_xyz(0.5, 1.5, 1.0)
                //         .tint(1.0, 1.0, 0.0, 1.0)
                //         .at(ax2, ay2, az2)
                //         .rotate(0.0, y_cord, -45f32.to_radians()),
                // );
                // // eyes
                // let (ex1, ey1, ez1) = rotate_point(-0.6, 1.0, 0.8, y_cord);
                // renderer.draw(
                //     DrawCommand::cube()
                //         .scale_xyz(1.0, 1.5, 1.0)
                //         .tint(1.0, 1.0, 1.0, 1.0)
                //         .at(ex1, ey1, ez1)
                //         .rotate(0.0, y_cord, 0.0),
                // );
                // let (ex2, ey2, ez2) = rotate_point(0.6, 1.0, 0.8, y_cord);
                // renderer.draw(
                //     DrawCommand::cube()
                //         .scale_xyz(1.0, 1.5, 1.0)
                //         .tint(1.0, 1.0, 1.0, 1.0)
                //         .at(ex2, ey2, ez2)
                //         .rotate(0.0, y_cord, 0.0),
                // );
                // let (px1, py1, pz1) = rotate_point(0.6, 1.0, 1.0, y_cord);
                // renderer.draw(
                //     DrawCommand::cube()
                //         .scale_xyz(0.5, 0.8, 1.0)
                //         .tint(0.0, 0.0, 0.0, 1.0)
                //         .at(px1, py1, pz1)
                //         .rotate(0.0, y_cord, 0.0),
                // );
                // let (px2, py2, pz2) = rotate_point(-0.6, 1.0, 1.0, y_cord);
                // renderer.draw(
                //     DrawCommand::cube()
                //         .scale_xyz(0.5, 0.8, 1.0)
                //         .tint(0.0, 0.0, 0.0, 1.0)
                //         .at(px2, py2, pz2)
                //         .rotate(0.0, y_cord, 0.0),
                // );
                // // mouth
                // let (mx, my, mz) = rotate_point(0.0, -0.4, 0.8, y_cord);
                // renderer.draw(
                //     DrawCommand::cube()
                //         .scale_xyz(1.0, 0.8, 0.5)
                //         .tint(0.0, 0.0, 0.0, 1.0)
                //         .at(mx, my, mz)
                //         .rotate(0.0, y_cord, 0.0),
                // );

                renderer.draw(DrawCommand::sphere().at(0.0, 0.0, -8.0).scale(1.0));

                renderer.end_drawing(time);

                gl_surface.swap_buffers(gl_context).unwrap();
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = StreetApp {
        renderer: None,
        gl_surface: None,
        gl_context: None,
        window: None,
        start_time: Instant::now(),
    };

    event_loop.run_app(&mut app).unwrap();
}
