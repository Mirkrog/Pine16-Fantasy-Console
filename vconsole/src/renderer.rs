use pixels::Pixels;
use std::sync::Arc;
use winit::window::Window;

pub struct Renderer {
    pixel_buffer: Option<Pixels<'static>>,
}
impl Renderer {
    pub fn new() -> Self {
        Self { pixel_buffer: None }
    }
    pub fn resume(&mut self, surface_texture: pixels::SurfaceTexture<Arc<Window>>) {
        self.pixel_buffer = Some(Pixels::new(100, 100, surface_texture).unwrap());
    }
}
