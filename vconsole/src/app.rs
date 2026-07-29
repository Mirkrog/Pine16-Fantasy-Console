use pixels::SurfaceTexture;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::platform::windows::{Color, WindowAttributesExtWindows};
use winit::window::{Window, WindowAttributes, WindowId};

use crate::console::Console;

pub struct App {
    window: Option<Arc<Window>>,
    console: Console,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            console: Console::default(),
        }
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("Pine16Console")
                        .with_title_background_color(Some(Color::from_rgb(128, 128, 128))),
                )
                .unwrap(),
        );

        self.console.resume_renderer(SurfaceTexture::new(
            window.inner_size().width,
            window.inner_size().height,
            window.clone(),
        ));

        self.window = Some(window);
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.console.render();
            }
            WindowEvent::Resized(size) => {
                self.console.resize_renderer(size);
            }
            _ => (),
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
        if !self.console.is_rom_loaded() {
            self.console.load_rom_from_path("test.o").unwrap();
        }
        let stopwatch = simple_stopwatch::Stopwatch::start_new();
        self.console.step();

        println!("steps took: {}ms", stopwatch.ms());

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
