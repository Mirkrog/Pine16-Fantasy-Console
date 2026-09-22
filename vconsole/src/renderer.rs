use pixels::{Pixels, wgpu::Color};

const PALETTE_OFFSET: usize = 300;
const TILESHEET_OFFSET: usize = 1000;
const TILEMAP_OFFSET: usize = TILESHEET_OFFSET + 2544;

// these are just for convenience and mustn't be changed
pub const CANVAS_WIDTH: usize = 320;
pub const CANVAS_HEIGHT: usize = 200;

pub struct Renderer {
    pixel_buffer: Option<Pixels<'static>>,
    surface_visible: bool,
    guardrails: bool,
}
impl Renderer {
    pub fn new(guardrails: bool) -> Self {
        Self {
            pixel_buffer: None,
            surface_visible: true,
            guardrails,
        }
    }
    pub fn resume(&mut self, mut pixel_buffer: Pixels<'static>) {
        pixel_buffer.clear_color(Color {
            r: 0.2,
            g: 0.2,
            b: 0.2,
            a: 1.0,
        });
        pixel_buffer.set_scaling_mode(pixels::ScalingMode::Fill);
        pixel_buffer.enable_vsync(false);

        self.pixel_buffer = Some(pixel_buffer);
    }
    pub fn resize_surface(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            log::debug!("Surface not visible, skipping rendering");
            self.surface_visible = false;
            return;
        } else if !self.surface_visible {
            log::debug!("Surface visible, skipping rendering");
            self.surface_visible = true;
        }
        match self.pixel_buffer.as_mut() {
            Some(buffer) => buffer.resize_surface(width, height).unwrap(),
            None => {
                log::warn!("Tried to resize uninitialized buffer")
            }
        }
    }
    pub fn draw_tile(
        &mut self,
        ram_slice: &[u16; 65535],
        mut id: usize,
        flags: u8,
        x: usize,
        y: usize,
    ) {
        let x_flipped = (flags >> 1) & 1 == 1;
        let y_flipped = flags & 1 == 1;
        if id >= 255 {
            if self.guardrails {
                panic!("Tile ID: {id} is out of bounds")
            } else {
                id = 0
            }
        }
        if id == 0 {
            return;
        }

        let frame = self
            .pixel_buffer
            .as_mut()
            .expect("Tried to draw to uninitialized Pixels canvas")
            .frame_mut();
        for (pixel_y, row) in ram_slice
            [TILESHEET_OFFSET + ((id - 1) * 16)..TILESHEET_OFFSET + ((id - 1) * 16) + 16]
            .chunks_exact(2)
            .enumerate()
        {
            for (pixel_chunk_x, half_row) in row.iter().enumerate() {
                let target_y = if y_flipped { 7 - pixel_y } else { pixel_y } + y;
                if target_y >= CANVAS_HEIGHT {
                    continue;
                }
                for x_offset in 0..4 {
                    let local_x = pixel_chunk_x * 4 + x_offset;
                    let target_x = if x_flipped { 7 - local_x } else { local_x } + x;
                    if target_x >= CANVAS_WIDTH {
                        continue;
                    }
                    let shift = (3 - x_offset) * 4;
                    let color_id = ((half_row >> shift) & 0x000F) as usize;
                    if color_id == 0 {
                        continue;
                    }
                    let color = ram_slice[PALETTE_OFFSET + color_id];
                    let (red, green, blue) = rgb_from_rgb565(color);
                    if let Some(pixel) = frame.get_mut(
                        (target_x + target_y * CANVAS_WIDTH) * 4
                            ..(target_x + target_y * CANVAS_WIDTH) * 4 + 4,
                    ) {
                        pixel.copy_from_slice(&[red, green, blue, 0xFF]);
                    }
                }
            }
        }
    }
    /// reads the consoles memory to draw sprites to the Pixel buffer
    pub fn draw(&mut self, ram_slice: &[u16; 65535], dump_frame: bool) {
        let background_map = &ram_slice[TILEMAP_OFFSET..TILEMAP_OFFSET + 1000];
        let sprite_layer = &ram_slice[TILEMAP_OFFSET + 1000..TILEMAP_OFFSET + 1999];
        let foreground_layer = &ram_slice[TILEMAP_OFFSET + 1999..TILEMAP_OFFSET + 2999];

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
                (tile_word & 0xFF00) as u8,
                (i * 8) % CANVAS_WIDTH,
                (i / (CANVAS_WIDTH / 8)) * 8,
            );
        }
        // drawing the sprite layer
        for tile_words in sprite_layer.chunks_exact(3) {
            if tile_words[1] as usize > CANVAS_WIDTH || tile_words[2] as usize > CANVAS_HEIGHT {
                continue;
            }
            self.draw_tile(
                ram_slice,
                (tile_words[0] & 0x00FF) as usize,
                (tile_words[0] & 0xFF00) as u8,
                tile_words[1] as usize,
                tile_words[2] as usize,
            );
        }
        // drawing the foreground layer
        for (i, tile_word) in foreground_layer.iter().enumerate() {
            self.draw_tile(
                ram_slice,
                (tile_word & 0x00FF) as usize,
                (tile_word & 0xFF00) as u8,
                (i * 8) % CANVAS_WIDTH,
                (i / (CANVAS_WIDTH / 8)) * 8,
            );
        }
        if dump_frame {
            log::info!(
                "{:?}",
                self.pixel_buffer
                    .as_mut()
                    .expect("Tried to dump uninitialized Pixels canvas")
                    .frame_mut()
                    .chunks_exact(4)
                    .collect::<Vec<&[u8]>>()
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
                log::warn!("Tried to render without pixel_buffer!");
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
