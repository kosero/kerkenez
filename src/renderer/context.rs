use glow::{Context, HasContext};
use glutin::{
    config::{ConfigTemplateBuilder, GlConfig},
    context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext},
    display::GetGlDisplay,
    prelude::{GlDisplay, GlSurface, NotCurrentGlContext},
    surface::{Surface, SwapInterval, WindowSurface},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use std::num::NonZeroU32;
use winit::{dpi::LogicalSize, event_loop::ActiveEventLoop, window::Window};

pub fn init_context(
    event_loop: &ActiveEventLoop,
    title: &str,
    width: i32,
    height: i32,
) -> (
    Context,
    Surface<WindowSurface>,
    PossiblyCurrentContext,
    Window,
) {
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

        let gl = glow::Context::from_loader_function_cstr(|s| gl_display.get_proc_address(s));

        gl_surface
            .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
            .ok();

        gl.enable(glow::DEPTH_TEST);
        gl.depth_func(glow::LESS);

        (gl, gl_surface, gl_context)
    };

    (gl, gl_surface, gl_context, window)
}
