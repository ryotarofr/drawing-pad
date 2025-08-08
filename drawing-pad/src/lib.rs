use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextAttributesBuilder, NotCurrentGlContext};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::SwapInterval;
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasWindowHandle;
use skia_safe::gpu::gl::FramebufferInfo;
use skia_safe::gpu::{self, DirectContext, SurfaceOrigin};
use skia_safe::{Color, Paint, Path, Point};
use std::ffi::CString;
use std::num::NonZeroU32;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

type WindowedContext = glutin::context::PossiblyCurrentContext;

// OpenGL bindings generated at build time
mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub struct DrawingApp {
    window: Option<Window>,
    gl_context: Option<WindowedContext>,
    gl_surface: Option<glutin::surface::Surface<glutin::surface::WindowSurface>>,
    gr_context: Option<DirectContext>,
    fb_info: Option<FramebufferInfo>,
    surface: Option<skia_safe::Surface>,
    current_path: Path,
    is_drawing: bool,
    window_size: PhysicalSize<u32>,
    cursor_position: Option<winit::dpi::PhysicalPosition<f64>>,
}

impl DrawingApp {
    pub fn new() -> Self {
        Self {
            window: None,
            gl_context: None,
            gl_surface: None,
            gr_context: None,
            fb_info: None,
            surface: None,
            current_path: Path::new(),
            is_drawing: false,
            window_size: PhysicalSize::new(800, 600),
            cursor_position: None,
        }
    }

    pub fn init_gl(&mut self, event_loop: &ActiveEventLoop) {
        let window_attrs = WindowAttributes::default()
            .with_title("Drawing Pad")
            .with_inner_size(self.window_size);

        let template = ConfigTemplateBuilder::new();

        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attrs));

        let (window, gl_config) = display_builder
            .build(event_loop, template, |configs| {
                configs.max_by_key(|c| c.num_samples()).unwrap()
            })
            .unwrap();

        let window = window.unwrap();

        let raw_window_handle = window.window_handle().unwrap().as_raw();

        let gl_display = gl_config.display();

        let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));

        let gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap()
        };

        let attrs = glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(self.window_size.width).unwrap(),
            NonZeroU32::new(self.window_size.height).unwrap(),
        );

        let gl_surface = unsafe {
            gl_config.display().create_window_surface(&gl_config, &attrs).unwrap()
        };

        let gl_context = gl_context.make_current(&gl_surface).unwrap();

        // Load GL function pointers
        gl::load_with(|s| {
            let c_str = CString::new(s).unwrap();
            gl_display.get_proc_address(&c_str) as *const _
        });

        gl_surface.set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap())).ok();

        self.gl_context = Some(gl_context);
        self.gl_surface = Some(gl_surface);
        self.window = Some(window);

        self.init_skia();
    }

    pub fn init_skia(&mut self) {
        let interface = gpu::gl::Interface::new_load_with(|name| {
            if name == "eglGetCurrentDisplay" {
                return std::ptr::null();
            }
            if let Ok(c_str) = CString::new(name) {
                self.gl_context
                    .as_ref()
                    .unwrap()
                    .display()
                    .get_proc_address(&c_str) as *const _
            } else {
                std::ptr::null()
            }
        })
        .expect("Could not create interface");

        let mut gr_context = gpu::direct_contexts::make_gl(interface, None).unwrap();

        let fb_info = {
            let mut fboid: gl::types::GLint = 0;
            unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };

            FramebufferInfo {
                fboid: fboid as u32,
                format: gpu::gl::Format::RGBA8.into(),
                ..Default::default()
            }
        };

        let backend_render_target = gpu::backend_render_targets::make_gl(
            (self.window_size.width as i32, self.window_size.height as i32),
            None,
            0,
            fb_info,
        );

        let surface = gpu::surfaces::wrap_backend_render_target(
            &mut gr_context,
            &backend_render_target,
            SurfaceOrigin::BottomLeft,
            skia_safe::ColorType::RGBA8888,
            None,
            None,
        )
        .unwrap();

        self.gr_context = Some(gr_context);
        self.fb_info = Some(fb_info);
        self.surface = Some(surface);
    }

    pub fn draw(&mut self) {
        if let Some(surface) = &mut self.surface {
            let canvas = surface.canvas();
            
            canvas.clear(Color::from_rgb(240, 240, 240));

            let mut paint = Paint::default();
            paint.set_color(Color::from_rgb(0, 0, 0));
            paint.set_stroke_width(3.0);
            paint.set_anti_alias(true);
            paint.set_style(skia_safe::PaintStyle::Stroke);

            canvas.draw_path(&self.current_path, &paint);

            if let Some(gr_context) = &mut self.gr_context {
                gr_context.flush_and_submit();
            }
        }

        if let (Some(gl_context), Some(gl_surface)) = (&self.gl_context, &self.gl_surface) {
            gl_surface.swap_buffers(gl_context).unwrap();
        }
    }

    pub fn handle_mouse_input(&mut self, position: winit::dpi::PhysicalPosition<f64>, state: ElementState) {
        let point = Point::new(position.x as f32, position.y as f32);

        match state {
            ElementState::Pressed => {
                self.is_drawing = true;
                self.current_path.move_to(point);
            }
            ElementState::Released => {
                self.is_drawing = false;
            }
        }
    }

    pub fn handle_mouse_move(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        if self.is_drawing {
            let point = Point::new(position.x as f32, position.y as f32);
            self.current_path.line_to(point);
            self.window.as_ref().unwrap().request_redraw();
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.window_size = new_size;

        if let (Some(gl_context), Some(gl_surface)) = (&self.gl_context, &self.gl_surface) {
            gl_surface.resize(
                gl_context,
                NonZeroU32::new(new_size.width).unwrap(),
                NonZeroU32::new(new_size.height).unwrap(),
            );

            if let Some(gr_context) = &mut self.gr_context {
                let fb_info = self.fb_info.unwrap();
                let backend_render_target = gpu::backend_render_targets::make_gl(
                    (new_size.width as i32, new_size.height as i32),
                    None,
                    0,
                    fb_info,
                );
                
                let surface = gpu::surfaces::wrap_backend_render_target(
                    gr_context,
                    &backend_render_target,
                    SurfaceOrigin::BottomLeft,
                    skia_safe::ColorType::RGBA8888,
                    None,
                    None,
                )
                .unwrap();
                self.surface = Some(surface);
            }
        }
    }
}

impl ApplicationHandler for DrawingApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.init_gl(event_loop);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                self.draw();
            }
            WindowEvent::Resized(size) => {
                self.resize(size);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::MouseInput { state, .. } => {
                if let Some(cursor_position) = self.cursor_position {
                    self.handle_mouse_input(cursor_position, state);
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = Some(position);
                self.handle_mouse_move(position);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.logical_key == winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space) {
                    self.current_path = Path::new();
                    self.window.as_ref().unwrap().request_redraw();
                }
            }
            _ => {}
        }
    }
}