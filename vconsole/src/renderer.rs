use pixels::Pixels;
use std::{
    sync::Arc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
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
    pub fn render(&mut self) {
        let pixel_buffer = match &mut self.pixel_buffer {
            Some(buffer) => buffer,
            None => {
                print!("Tried to print without pixel_buffer!");
                return;
            }
        };
        let canvas: &mut [u8] = pixel_buffer.frame_mut();

        let time = ((std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as i32)
            / 10000) as u32;

        for (index, pixel) in canvas.chunks_exact_mut(4).enumerate() {
            pixel[0] = index as u8; // R
            pixel[1] = (index as u8).wrapping_add((time % 255) as u8); // G
            pixel[2] = (index as u8).wrapping_mul(101); // B
            pixel[3] = 0xff; // A
        }
        pixel_buffer.render().unwrap();
    }
}
