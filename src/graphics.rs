use std::{collections::VecDeque, ops::RangeFrom};

use sdl2::{
    pixels::{Color, PixelFormatEnum},
    render::{Canvas, TextureCreator},
    video::{Window, WindowContext},
    EventPump, Sdl, TimerSubsystem,
};
use std::fmt;

use crate::{
    cpu::CPU,
    memory::Memory,
    utils::{get_flag, set_flag, set_flag_ref, Address, Byte, Word},
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

const LCD_STATUS_ADDRESS: Address = 0xFF41;
const LCY_INT_FLAG: Byte = 0b0100_0000;
const MODE2_INT_FLAG: Byte = 0b0010_0000;
const MODE1_INT_FLAG: Byte = 0b0001_0000;
const MODE0_INT_FLAG: Byte = 0b0000_1000;
const LYC_EQ_LY_FLAG: Byte = 0b0000_0100;

const SCANLINE_CYCLES: u128 = 114;

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
        let default_tile = Pixel {
            color_ref: 0,
            pixel_source,
        };
        let mut tile = [[default_tile; 8]; 8];

        for x in 0..8 {
            let lsb_address = address + 2 * (x as Address);
            let msb_address = address + 2 * (x as Address) + 1;

            let lsb = memory.read_byte(lsb_address);
            let msb = memory.read_byte(msb_address);

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
        let scy = memory.read_byte(SCY_ADDRESS) as usize;
        let scx = memory.read_byte(SCX_ADDRESS) as usize;
        (scx, scy)
    }
    fn get_viewport(memory: &Memory) -> (usize, usize) {
        let wy = memory.read_byte(WY_ADDRESS) as usize;
        let wx = memory.read_byte(WX_ADDRESS) as usize;
        (wx, wy)
    }
    fn in_window(p: PixelPos, memory: &Memory) -> bool {
        let (wx, wy) = Self::get_viewport(memory);
        let lcdc = memory.read_byte(LCDC_ADDRESS);
        let window_enable = get_flag(lcdc, WINDOW_ENABLE_FLAG);
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
        let lcdc = memory.read_byte(LCDC_ADDRESS);

        while self.fifo.len() < 8 {
            let (fx, fy, map_address) = if !self.in_window {
                let bcg_map_address = if get_flag(lcdc, BG_TILE_MAP_FLAG) {
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
                let window_map_address = if get_flag(lcdc, WINDOW_TILE_MAP_FLAG) {
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
            let bcw_tile_address = if get_flag(lcdc, BGW_TILES_DATA_FLAG) {
                0x8000
            } else {
                0x8800
            };
            let tile_num_address = map_address + (tile_idx as Address);
            let tile_num = memory.read_byte(tile_num_address);
            let start_address = bcw_tile_address + BYTES_PER_TILE * (tile_num as Address);

            let tile = Tile::fetch_tile(memory, PixelSource::Background, start_address);
            let (tx, ty) = (fp.x % 8, fp.y % 8);
            let tile_line = tile.get_range(tx.., ty);
            self.fifo.extend(tile_line);
        }
    }
}

pub struct ObjFIFO {}

/// PPU Mode with corresponding line number
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PPUMode {
    /// Horizontal BLANK
    Mode0(usize),
    /// Vertical BLANK
    Mode1(usize),
    /// OAM Scan
    Mode2(usize),
    /// Drawing Pixels
    Mode3(usize),
}

impl PPUMode {
    fn to_num(&self) -> Byte {
        match self {
            Self::Mode0(_) => 0,
            Self::Mode1(_) => 1,
            Self::Mode2(_) => 2,
            Self::Mode3(_) => 3,
        }
    }
}

pub struct Graphics {
    pub context: Sdl,
    pub canvas: Canvas<Window>,
    pub event_pump: EventPump,
    pub texture_creator: TextureCreator<WindowContext>,
    pub timer: TimerSubsystem,

    // gb related
    line_y: usize,
    screen_buffer: [Byte; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
    last_timestamp: u128,
    bg_fifo: BgFIFO,
    last_ppu_mode: PPUMode,
}

impl Graphics {
    pub fn new() -> Self {
        // Initialize SDL
        let context = sdl2::init().unwrap();

        // Set hint for vsync
        sdl2::hint::set("SDL_HINT_RENDER_VSYNC", "1");

        // Create window and renderer
        let video_subsystem = context.video().unwrap();
        let window = video_subsystem
            .window("GB-rs", SCREEN_WIDTH as u32 * 2, SCREEN_HEIGHT as u32 * 2)
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
            last_timestamp: 0,
            bg_fifo: BgFIFO::new(),
            last_ppu_mode: PPUMode::Mode1(153),
        }
    }

    /// Render according to gb specifications [pandocs](https://gbdev.io/pandocs/Rendering.html)
    /// Each line requires 456 dots = 114 machine cycles,
    /// First 20 mcycles are OAM scan,
    /// Between 20-72/92 mcycles are pixel rendering
    /// Between 72/92-114 mcycles is HBlank (do nothing)
    pub fn render(&mut self, memory: &mut Memory, timestamp: u128) {
        let clock_diff = timestamp - self.last_timestamp;

        if clock_diff >= SCANLINE_CYCLES {
            // to next line
            self.last_timestamp = self.last_timestamp + SCANLINE_CYCLES;
            self.line_y += 1;
        }

        if self.line_y > 153 {
            // next cycle
            self.line_y = 0;
            self.bg_fifo = BgFIFO::new();
        }

        let clock_diff = timestamp - self.last_timestamp;
        let current_ppu_mode = self.get_mode(clock_diff);

        if self.last_ppu_mode != current_ppu_mode {
            // PPU Mode transitions
            match (self.last_ppu_mode, current_ppu_mode) {
                (PPUMode::Mode1(l1), PPUMode::Mode2(l2)) if l1 == 153 && l2 == 0 => {
                    // new frame
                    self.set_lyc(memory);
                }
                (PPUMode::Mode2(l1), PPUMode::Mode3(l2)) if l1 == l2 => {
                    // draw line to screen_buffer
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
                }
                (PPUMode::Mode3(l1), PPUMode::Mode0(l2)) if l1 == l2 => {
                    // finish draw pixel to hblank
                }
                (PPUMode::Mode0(l1), PPUMode::Mode2(l2)) if l1 + 1 == l2 => {
                    // newline
                    self.set_lyc(memory);
                }
                (PPUMode::Mode0(l1), PPUMode::Mode1(l2)) if l1 + 1 == l2 => {
                    // render to screen if vblank
                    self.set_lyc(memory);
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
                }
                (PPUMode::Mode1(l1), PPUMode::Mode1(l2)) if l1 + 1 == l2 => {
                    // newline in vblank mode
                    self.set_lyc(memory);
                }
                _ => panic!(
                    "PPU Transition Error {:?} {:?}, Clock Diff {:?} at line {:?}",
                    self.last_ppu_mode, current_ppu_mode, clock_diff, self.line_y
                ),
            }
            self.last_ppu_mode = current_ppu_mode;
            self.set_ppu(current_ppu_mode, memory);
        }
    }

    fn get_mode(&self, clock_diff: u128) -> PPUMode {
        assert!(clock_diff <= SCANLINE_CYCLES);
        if self.line_y >= 144 {
            PPUMode::Mode1(self.line_y)
        } else if clock_diff <= 20 {
            PPUMode::Mode2(self.line_y)
        } else if clock_diff < 77 {
            PPUMode::Mode3(self.line_y)
        } else {
            PPUMode::Mode0(self.line_y)
        }
    }

    /// Set ppu stat flag and LCD interrupt flag
    fn set_ppu(&self, ppu_mode: PPUMode, memory: &mut Memory) {
        let stat_flag = memory.read_byte(LCD_STATUS_ADDRESS) & !0b11;
        let new_stat_flag = stat_flag | ppu_mode.to_num();

        // interrupt
        let mut int_flag = memory.read_byte(CPU::INTERRUPT_FLAG_ADDRESS);
        match ppu_mode {
            PPUMode::Mode0(_) if get_flag(stat_flag, MODE0_INT_FLAG) => {
                set_flag(&mut int_flag, CPU::LCD_FLAG);
            }
            PPUMode::Mode1(_) if get_flag(stat_flag, MODE1_INT_FLAG) => {
                set_flag(&mut int_flag, CPU::LCD_FLAG);
            }
            PPUMode::Mode2(_) if get_flag(stat_flag, MODE2_INT_FLAG) => {
                set_flag(&mut int_flag, CPU::LCD_FLAG);
            }
            _ => (),
        }
        memory.write_byte(CPU::INTERRUPT_FLAG_ADDRESS, int_flag);
        memory.write_byte(LCD_STATUS_ADDRESS, new_stat_flag);
    }

    /// Set ly and lyc int/flags
    fn set_lyc(&self, memory: &mut Memory) {
        memory.write_byte(LY_ADDRESS, self.line_y as Byte);
        let lyc = memory.read_byte(LYC_ADDRESS) as usize;
        if lyc == self.line_y {
            // set the lyc == ly flag in stat
            let stat_flag = memory.read_byte(LCD_STATUS_ADDRESS);
            let new_stat_flag = set_flag_ref(stat_flag, LYC_EQ_LY_FLAG);
            memory.write_byte(LCD_STATUS_ADDRESS, new_stat_flag);

            if get_flag(stat_flag, LCY_INT_FLAG) {
                let mut int_flag = memory.read_byte(CPU::INTERRUPT_FLAG_ADDRESS);
                set_flag(&mut int_flag, CPU::LCD_FLAG);
                memory.write_byte(CPU::INTERRUPT_FLAG_ADDRESS, int_flag);
            }
        }
    }
}
