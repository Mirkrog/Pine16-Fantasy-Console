use pixels::Pixels;
use std::sync::Arc;
use winit::window::Window;

const PALETTE_OFFSET: usize = 300;
const TILESHEET_OFFSET: usize = 1000;
const TILEMAP_OFFSET: usize = TILESHEET_OFFSET + 2560;

const CANVAS_WIDTH: usize = 320;
const CAVAS_HEIGHT: usize = 200;

pub struct Renderer {
    pixel_buffer: Option<Pixels<'static>>,
}
impl Renderer {
    pub fn new() -> Self {
        Self { pixel_buffer: None }
    }
    pub fn resume(&mut self, surface_texture: pixels::SurfaceTexture<Arc<Window>>) {
        let mut pixel_buffer =
            Pixels::new(CANVAS_WIDTH as u32, CAVAS_HEIGHT as u32, surface_texture).unwrap();

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
        if id >= 256 {
            panic!("Tile ID: {id} is out of bounds")
        }
        if id == 0 {
            return;
        }
        let pixel_buffer = self
            .pixel_buffer
            .as_mut()
            .expect("Tried to draw to uninitialized Pixels canvas")
            .frame_mut();
        for (pixel_y, row) in ram_slice[TILESHEET_OFFSET + id..4 * 8]
            .chunks_exact(4)
            .enumerate()
        {
            for (pixel_x, palette_index) in row.iter().enumerate() {
                let target_x = x + pixel_x;
                let target_y = y + pixel_y;
                let pixel = pixel_buffer
                    .chunks_exact_mut(8)
                    .nth(target_y * CANVAS_WIDTH + target_x)
                    .expect("Canvas pixel out of bounds: {target_x}, {target_y}");
                if *palette_index >= 256 {
                    panic!("Palette index out of bounds: {palette_index}")
                }
                if *palette_index == 0 {
                    continue;
                }
                let color1 = ram_slice[PALETTE_OFFSET + *palette_index as usize] & 0xFF00;
                pixel[0] = (color1 >> 11) as u8;
                pixel[1] = ((color1 & 0b00000111111) >> 6) as u8;
                pixel[2] = (color1 & 0b0000000000011111) as u8;
                pixel[3] = 0xFF;
                let color2 = ram_slice[PALETTE_OFFSET + *palette_index as usize] & 0x00FF;
                pixel[4] = (color2 >> 11) as u8;
                pixel[5] = ((color2 & 0b00000111111) >> 6) as u8;
                pixel[6] = (color2 & 0b0000000000011111) as u8;
                pixel[7] = 0xFF;
            }
        }
    }
    /// reads the consoles memory to draw sprites to the Pixel buffer
    pub fn draw(&mut self, ram_slice: &[u16]) {
        let background_map = &ram_slice[TILESHEET_OFFSET..1000];
        let spritelayer = &ram_slice[TILESHEET_OFFSET..2000];
        let foreground_layer = &ram_slice[TILESHEET_OFFSET..3000];

        for (i, tile_word) in background_map.iter().enumerate() {
            self.draw_tile(
                ram_slice,
                (tile_word & 0xFF00) as usize,
                (tile_word & 0x00FF) as usize,
                i % CANVAS_WIDTH,
                i / CANVAS_WIDTH,
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
