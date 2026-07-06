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
        let mut pixel_buffer = Pixels::new(320, 200, surface_texture).unwrap();

        pixel_buffer.set_scaling_mode(pixels::ScalingMode::Fill);

        self.pixel_buffer = Some(pixel_buffer);
    }
    pub fn resize_surface(&mut self, width: u32, height: u32) {
        self.pixel_buffer
            .as_mut()
            .expect("Tried to resize, but pixelbuffer is uninitialized")
            .resize_surface(width, height)
            .unwrap();
    }
    /// takes the consoles memory to draw sprites to the Pixel buffer
    pub fn draw(&mut self, ram_slice: &[u16]) {}
    pub fn render(&mut self) {
        let pixel_buffer = match &mut self.pixel_buffer {
            Some(buffer) => buffer,
            None => {
                println!("Tried to render without pixel_buffer!");
                return;
            }
        };

        pixel_buffer.render().unwrap();
    }
}
