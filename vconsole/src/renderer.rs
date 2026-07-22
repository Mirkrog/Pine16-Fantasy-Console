use pixels::{Pixels, wgpu::naga::back::spv::SourceLanguage::HERO_C};
use std::sync::Arc;
use winit::window::Window;

const PALETTE_OFFSET: usize = 300;
const TILESHEET_OFFSET: usize = 1000;
const TILEMAP_OFFSET: usize = TILESHEET_OFFSET + 2560;

// these are just for convenience and mustn't be changed
const CANVAS_WIDTH: usize = 320;
const CAVAS_HEIGHT: usize = 200;

pub struct Renderer {
    pixel_buffer: Option<Pixels<'static>>,
    surface_visible: bool,
}
impl Renderer {
    pub fn new() -> Self {
        Self {
            pixel_buffer: None,
            surface_visible: true,
        }
    }
    pub fn resume(&mut self, surface_texture: pixels::SurfaceTexture<Arc<Window>>) {
        let mut pixel_buffer =
            Pixels::new(CANVAS_WIDTH as u32, CAVAS_HEIGHT as u32, surface_texture).unwrap();

        pixel_buffer.set_scaling_mode(pixels::ScalingMode::Fill);
        pixel_buffer.enable_vsync(true);

        self.pixel_buffer = Some(pixel_buffer);
    }
    pub fn resize_surface(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            println!("Surface not visible, dissabling rendering");
            self.surface_visible = false;
            return;
        } else if !self.surface_visible {
            println!("Surface visible, enabling rendering");
            self.surface_visible = true;
        }
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
        for (pixel_y, row) in ram_slice
            [TILESHEET_OFFSET + (id * 16)..TILESHEET_OFFSET + (id * 16) + 16]
            .chunks_exact(2)
            .enumerate()
        {
            for (pixel_chunk_x, half_row) in row.iter().enumerate() {
                let target_x = x + pixel_chunk_x * 4;
                let target_y = y + pixel_y;
                for x_offset in 0..4 {
                    let pixel = self
                        .pixel_buffer
                        .as_mut()
                        .expect("Tried to draw to uninitialized Pixels canvas")
                        .frame_mut()
                        .chunks_exact_mut(4)
                        .nth(target_x + x_offset + target_y * CANVAS_WIDTH)
                        .unwrap_or_else(|| panic!("Pixel coordinate: ({target_x}, {target_y}) out of range: ({CANVAS_WIDTH}, {CAVAS_HEIGHT})"));
                    let shift = (3 - x_offset) * 4;
                    let color_id = ((half_row >> shift) & 0x000F) as usize;
                    if color_id == 0 {
                        continue;
                    }
                    if color_id >= 16 {
                        panic!("Color Palette index: {color_id} is out of range (0 - 15)")
                    }
                    let color = ram_slice[PALETTE_OFFSET + color_id];
                    let (red, green, blue) = rgb_from_rgb565(color);
                    pixel[0] = red;
                    pixel[1] = green;
                    pixel[2] = blue;
                    pixel[3] = 0xff;
                }
            }
        }
    }
    /// reads the consoles memory to draw sprites to the Pixel buffer
    pub fn draw(&mut self, ram_slice: &[u16]) {
        let background_map = &ram_slice[TILEMAP_OFFSET..TILEMAP_OFFSET + 1000];
        let spritelayer = &ram_slice[TILEMAP_OFFSET..TILEMAP_OFFSET + 2000];
        let foreground_layer = &ram_slice[TILEMAP_OFFSET..TILEMAP_OFFSET + 3000];

        //drawing the clear color
        let clear_color = ram_slice[300];
        let (red, green, blue) = rgb_from_rgb565(clear_color);
        for pixel in self
            .pixel_buffer
            .as_mut()
            .expect("Tried to draw to uninitialized Pixels canvas")
            .frame_mut()
            .chunks_exact_mut(4)
        {
            pixel[0] = red;
            pixel[1] = green;
            pixel[2] = blue;
            pixel[3] = 0xff;
        }

        // drawing the background layer
        for (i, tile_word) in background_map.iter().enumerate() {
            self.draw_tile(
                ram_slice,
                (tile_word & 0x00FF) as usize,
                (tile_word & 0xFF00) as usize,
                (i * 8) % CANVAS_WIDTH,
                (i * 8) / CANVAS_WIDTH * 8,
            );
        }
    }
    pub fn render(&mut self) {
        if !self.surface_visible {
            return;
        }
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
fn rgb_from_rgb565(rgb565: u16) -> (u8, u8, u8) {
    let r5 = ((rgb565 >> 11) & 0b00011111) as u8;
    let g6 = ((rgb565 >> 5) & 0b00111111) as u8;
    let b5 = (rgb565 & 0b00011111) as u8;
    let red = ((r5 as u32 * 255) / 31) as u8;
    let green = ((g6 as u32 * 255) / 63) as u8;
    let blue = ((b5 as u32 * 255) / 31) as u8;
    (red, green, blue)
}
