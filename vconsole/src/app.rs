use pixels::SurfaceTexture;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
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
                .create_window(WindowAttributes::default())
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
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
