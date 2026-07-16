use pixels::Pixels;
use std::sync::Arc;
use winit::window::Window;

const PALETTE_OFFSET: usize = 300;
const TILESHEET_OFFSET: usize = 1000;
const TILEMAP_OFFSET: usize = TILESHEET_OFFSET + 2560;

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
    pub fn draw_tile(&mut self, ram_slice: &[u16], id: usize, flags: usize, x: usize, y: usize) {
        if id >= 255 {
            panic!("Tile ID: {id} is out of bounds")
        }
        for (y, row) in ram_slice[TILESHEET_OFFSET + id..4 * 8]
            .chunks_exact(4)
            .enumerate()
        {}
    }
    /// reads the consoles memory to draw sprites to the Pixel buffer
    pub fn draw(&mut self, ram_slice: &[u16]) {
        let background_map = &ram_slice[TILESHEET_OFFSET..1000];
        let spritelayer = &ram_slice[TILESHEET_OFFSET..2000];
        let foreground_layer = &ram_slice[TILESHEET_OFFSET..3000];

        for (i, tile_word) in background_map.chunks_exact(3).enumerate() {
            self.draw_tile(
                ram_slice,
                tile_word[2] as usize & 0xFF00,
                tile_word[2] as usize & 0x00FF,
                tile_word[0] as usize,
                tile_word[1] as usize,
            );
        }
    }
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
