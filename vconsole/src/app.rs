use pixels::SurfaceTexture;
use std::{
    sync::Arc,
    thread::sleep,
    time::{Duration, Instant},
};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
#[cfg(windows)]
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

fn window_attributes() -> WindowAttributes {
    let attributes = WindowAttributes::default().with_title("Pine16Console");

    // The title-bar background color is a Windows-only winit extension.
    #[cfg(windows)]
    let attributes = attributes.with_title_background_color(Some(Color::from_rgb(128, 128, 128)));

    attributes
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(window_attributes()).unwrap());

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
        let now = Instant::now();
        self.console.step();

        if let Some(window) = &self.window {
            window.request_redraw();
        }

        sleep(Duration::from_millis(
            ((1.0 / 30.0) * 1000.0) as u64 - now.elapsed().subsec_millis() as u64,
        ));
    }
}
