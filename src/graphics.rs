use std::{collections::VecDeque, ops::RangeFrom};

use sdl2::{
    pixels::{Color, PixelFormatEnum},
    rect::Point,
    render::{Canvas, TextureCreator},
    video::{Window, WindowContext},
    EventPump, Sdl, TimerSubsystem,
};
use std::fmt;

use crate::{
    memory::Memory,
    utils::{Address, Byte, Word},
};

const BYTES_PER_TILE: Word = 16;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const PIXEL_COUNT: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

const OBJ_TILE_ADDRESS: Address = 0x8000;
const SCY_ADDRESS: Address = 0xFF42;
const SCX_ADDRESS: Address = 0xFF43;
const WY_ADDRESS: Address = 0xFF4A;
const WX_ADDRESS: Address = 0xFF4B;
const LY_ADDRESS: Address = 0xFF44;
const LYC_ADDRESS: Address = 0xFF45;

const LCDC_ADDRESS: Address = 0xFF40;
const LCDC_ENABLE_FLAG: Byte = 0b1000_0000;
const WINDOW_TILE_MAP_FLAG: Byte = 0b0100_0000;
const WINDOW_ENABLE_FLAG: Byte = 0b0010_0000;
const BGW_TILES_DATA_FLAG: Byte = 0b0001_0000;
const BG_TILE_MAP_FLAG: Byte = 0b0000_1000;
const OBJ_SIZE_FLAG: Byte = 0b0000_0100;
const OBJ_ENABLE_FLAG: Byte = 0b0000_0010;
const BGW_ENABLE_FLAG: Byte = 0b0000_0001;

const BLACK: Color = Color::RGB(15, 56, 15);
const DARK_GREY: Color = Color::RGB(48, 98, 48);
const LIGHT_GREY: Color = Color::RGB(139, 172, 15);
const WHITE: Color = Color::RGB(155, 188, 15);

#[derive(Clone, Copy, Debug)]
enum PixelSource {
    Background,
    Object,
}

#[derive(Clone, Copy)]
struct Pixel {
    color_ref: u8, // should be u2
    pixel_source: PixelSource,
}

impl fmt::Debug for Pixel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.color_ref)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct PixelPos {
    x: usize,
    y: usize,
}

#[derive(Clone, Copy, Debug)]
struct TilePos {
    i: usize,
    j: usize,
}

impl PixelPos {
    fn new() -> PixelPos {
        PixelPos { x: 0, y: 0 }
    }
    fn to_tile(&self) -> TilePos {
        TilePos {
            i: self.x / 8,
            j: self.y / 8,
        }
    }
    fn next_line(&self) -> Self {
        Self {
            x: 0,
            y: self.y + 1,
        }
    }
    fn add(&self, dx: usize, dy: usize) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
    fn subtract(&self, dx: usize, dy: usize) -> Self {
        Self {
            x: self.x - dx,
            y: self.y - dy,
        }
    }
}

#[derive(Clone, Copy)]
struct Tile {
    tile: [[Pixel; 8]; 8],
}

impl fmt::Debug for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for i in 0..8 {
            for j in 0..8 {
                write!(f, "{}", self.tile[i][j].color_ref)?;
            }
            if i != 7 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl Tile {
    fn all_zero(&self) -> bool {
        for i in 0..8 {
            for j in 0..8 {
                if self.tile[i][j].color_ref != 0 {
                    return false;
                }
            }
        }
        true
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Pixel {
        self.tile[y][x]
    }

    pub fn fetch_tile(memory: &Memory, pixel_source: PixelSource, address: Address) -> Self {
        // println!("{}", address2string(address));

        let default_tile = Pixel {
            color_ref: 0,
            pixel_source,
        };
        let mut tile = [[default_tile; 8]; 8];

        for x in 0..8 {
            let lsb_address = address + 2 * (x as Address);
            let msb_address = address + 2 * (x as Address) + 1;

            let lsb = memory.read_byte_unsafe(lsb_address);
            let msb = memory.read_byte_unsafe(msb_address);

            for y in 0..8 {
                let b = 7 - y;
                let color_ref = ((msb >> b) & 1) * 2 + ((lsb >> b) & 1);
                tile[x][y] = Pixel {
                    color_ref,
                    pixel_source,
                };
            }
        }

        Self { tile }
    }

    pub fn get_range(&self, x: RangeFrom<usize>, y: usize) -> &[Pixel] {
        &self.tile[y][x]
    }
}

struct BgFIFO {
    fifo: VecDeque<Pixel>,
    initialized: bool,

    screen_pos: PixelPos,
    in_window: bool,
}

impl BgFIFO {
    fn new() -> Self {
        let screen_pos = PixelPos::new();
        Self {
            fifo: VecDeque::new(),
            screen_pos,
            initialized: false,
            in_window: false,
        }
    }
    fn get_scroll(memory: &Memory) -> (usize, usize) {
        let scy = memory.read_byte_unsafe(SCY_ADDRESS) as usize;
        let scx = memory.read_byte_unsafe(SCX_ADDRESS) as usize;
        (scx, scy)
    }
    fn get_viewport(memory: &Memory) -> (usize, usize) {
        let wy = memory.read_byte_unsafe(WY_ADDRESS) as usize;
        let wx = memory.read_byte_unsafe(WX_ADDRESS) as usize;
        (wx, wy)
    }
    fn in_window(p: PixelPos, memory: &Memory) -> bool {
        let (wx, wy) = Self::get_viewport(memory);
        let lcdc = memory.read_byte_unsafe(LCDC_ADDRESS);
        let window_enable = Memory::get_flag(lcdc, WINDOW_ENABLE_FLAG);
        window_enable && p.x + 7 >= wx && p.y >= wy
    }

    // must call before using
    fn next_line(&mut self, memory: &Memory) {
        self.screen_pos = if self.initialized {
            self.screen_pos.next_line()
        } else {
            self.initialized = true;
            self.screen_pos
        };
        self.in_window = Self::in_window(self.screen_pos, memory);
        self.fifo.clear();

        self.fetch(memory);
    }
    fn pop(&mut self, memory: &Memory) -> Pixel {
        if !self.in_window && Self::in_window(self.screen_pos, memory) {
            self.in_window = true;
            self.fifo.clear();
            self.fetch(memory);
        }
        let p = self.fifo.pop_front().unwrap();
        self.screen_pos.x += 1;
        self.fetch(memory);
        p
    }
    fn fetch(&mut self, memory: &Memory) {
        let lcdc = memory.read_byte_unsafe(LCDC_ADDRESS);

        while self.fifo.len() < 8 {
            let (fx, fy, map_address) = if !self.in_window {
                let bcg_map_address = if Memory::get_flag(lcdc, BG_TILE_MAP_FLAG) {
                    0x9C00
                } else {
                    0x9800
                };
                let (dx, dy) = Self::get_scroll(memory);
                (
                    (self.screen_pos.x + self.fifo.len() + dx) % 255,
                    (self.screen_pos.y + dy) % 255,
                    bcg_map_address,
                )
            } else {
                let window_map_address = if Memory::get_flag(lcdc, WINDOW_TILE_MAP_FLAG) {
                    0x9C00
                } else {
                    0x9800
                };
                let (wx, wy) = Self::get_viewport(memory);
                (
                    (self.screen_pos.x + self.fifo.len() + 7 - wx) % 255,
                    (self.screen_pos.y - wy) % 255,
                    window_map_address,
                )
            };
            let fp = PixelPos { x: fx, y: fy };
            let tile_pos = fp.to_tile();
            let tile_idx = tile_pos.i + tile_pos.j * 32;
            let bcw_tile_address = if Memory::get_flag(lcdc, BGW_TILES_DATA_FLAG) {
                0x8000
            } else {
                0x8800
            };
            let tile_num_address = map_address + (tile_idx as Address);
            let tile_num = memory.read_byte_unsafe(tile_num_address);
            let start_address = bcw_tile_address + BYTES_PER_TILE * (tile_num as Address);

            let tile = Tile::fetch_tile(memory, PixelSource::Background, start_address);
            let (tx, ty) = (fp.x % 8, fp.y % 8);
            let tile_line = tile.get_range(tx.., ty);
            self.fifo.extend(tile_line);
        }
    }
}

pub struct ObjFIFO {}

pub struct Graphics {
    pub context: Sdl,
    pub canvas: Canvas<Window>,
    pub event_pump: EventPump,
    pub texture_creator: TextureCreator<WindowContext>,
    pub timer: TimerSubsystem,

    // gb related
    line_y: usize,
    screen_buffer: [Byte; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
    line_drawn: bool,
    rendered: bool,
    last_timestamp: u128,
    bg_fifo: BgFIFO,
}

impl Graphics {
    pub fn new() -> Self {
        // Initialize SDL
        let context = sdl2::init().unwrap();

        // Set hint for vsync
        // sdl2::hint::set("SDL_HINT_RENDER_VSYNC", "1");

        // Create window and renderer
        let video_subsystem = context.video().unwrap();
        let window = video_subsystem
            .window("GB-rs", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(BLACK);
        canvas.clear();

        let event_pump = context.event_pump().unwrap();

        let texture_creator = canvas.texture_creator();

        let timer = context.timer().unwrap();

        Self {
            context,
            canvas,
            event_pump,
            texture_creator,
            timer,
            screen_buffer: [0; PIXEL_COUNT * 3],
            line_y: 0,
            line_drawn: false,
            rendered: false,
            last_timestamp: 0,
            bg_fifo: BgFIFO::new(),
        }
    }

    /// Render according to gb specifications [pandocs](https://gbdev.io/pandocs/Rendering.html)
    /// Each line requires 456 dots = 114 machine cycles,
    /// First 20 mcycles are OAM scan,
    /// Between 20-72/92 mcycles are pixel rendering
    /// Between 72/92-114 mcycles is HBlank (do nothing)
    pub fn render(&mut self, memory: &mut Memory, timestamp: u128) {
        let clock_diff = timestamp - self.last_timestamp;

        if clock_diff >= 114 {
            // to next line
            self.last_timestamp = self.last_timestamp + 114;
            self.line_y += 1;
            self.line_drawn = false;
            memory.write_byte(LY_ADDRESS, self.line_y as Byte);
        }

        if self.line_y >= 144 {
            // render to screen
            if !self.rendered {
                let mut texture = self
                    .texture_creator
                    .create_texture_target(
                        PixelFormatEnum::RGB24,
                        SCREEN_WIDTH as u32,
                        SCREEN_HEIGHT as u32,
                    )
                    .unwrap();
                texture
                    .update(None, &self.screen_buffer, SCREEN_WIDTH * 3)
                    .unwrap();
                self.canvas.copy(&texture, None, None).unwrap();
                self.canvas.present();
                self.rendered = true;
            }
        } else if !self.line_drawn && clock_diff > 20 && clock_diff <= 92 {
            // draw line to screen

            self.bg_fifo.next_line(memory);
            for x in 0..SCREEN_WIDTH {
                let val = self.bg_fifo.pop(memory);
                let color = match val.color_ref {
                    0 => BLACK,
                    1 => DARK_GREY,
                    2 => LIGHT_GREY,
                    3 => WHITE,
                    _ => panic!("{:?} unknown pixel value", val),
                };

                let offset = self.line_y * SCREEN_WIDTH * 3 + x * 3;
                self.screen_buffer[offset] = color.r;
                self.screen_buffer[offset + 1] = color.g;
                self.screen_buffer[offset + 2] = color.b;
            }
            self.line_drawn = true;
        }

        if self.line_y > 153 {
            // next cycle
            self.line_y = 0;
            self.line_drawn = false;
            self.rendered = false;
            self.bg_fifo = BgFIFO::new();
            memory.write_byte(LY_ADDRESS, self.line_y as Byte);
        }
    }
}
