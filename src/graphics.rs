use std::{
    collections::{hash_map::Entry, HashMap, VecDeque},
    ops::Range,
};

use sdl2::{
    pixels::{Color, PixelFormatEnum},
    render::{Canvas, TextureCreator},
    video::{Window, WindowContext},
    EventPump, Sdl, TimerSubsystem,
};
use std::fmt;

use crate::{
    cpu::{INTERRUPT_FLAG_ADDRESS, LCD_FLAG, VBLANK_FLAG},
    memory::Memory,
    utils::{get_flag, set_flag, set_flag_ref, Address, Byte, Word},
};

const BYTES_PER_TILE: Word = 16;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const PIXEL_COUNT: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

pub const OAM_ADDRESS: Address = 0xFE00;
const SCY_ADDRESS: Address = 0xFF42;
const SCX_ADDRESS: Address = 0xFF43;
const WY_ADDRESS: Address = 0xFF4A;
const WX_ADDRESS: Address = 0xFF4B;
const LY_ADDRESS: Address = 0xFF44;
const LYC_ADDRESS: Address = 0xFF45;

// LCDC flags
const LCDC_ADDRESS: Address = 0xFF40;
const LCDC_ENABLE_FLAG: Byte = 0b1000_0000;
const WINDOW_TILE_MAP_FLAG: Byte = 0b0100_0000;
const WINDOW_ENABLE_FLAG: Byte = 0b0010_0000;
const BGW_TILES_DATA_FLAG: Byte = 0b0001_0000;
const BG_TILE_MAP_FLAG: Byte = 0b0000_1000;
const OBJ_SIZE_FLAG: Byte = 0b0000_0100;
const OBJ_ENABLE_FLAG: Byte = 0b0000_0010;
const BGW_ENABLE_FLAG: Byte = 0b0000_0001;

const BG_PALETTE_ADDRESS: Address = 0xFF47;
const OBP0_ADDRESS: Address = 0xFF48;
const OBP1_ADDRESS: Address = 0xFF49;

// Object Attribute/Flags
const OBJ_TILE_ADDRESS: Address = 0x8000;
const OBJ_COUNT: usize = 40;
const OBJ_PRIORITY_FLAG: Byte = 0b1000_0000;
const OBJ_YFLIP_FLAG: Byte = 0b0100_0000;
const OBJ_XFLIP_FLAG: Byte = 0b0010_0000;
const OBJ_PALETTE_FLAG: Byte = 0b0001_0000;

const LCD_STATUS_ADDRESS: Address = 0xFF41;
const LCY_INT_FLAG: Byte = 0b0100_0000;
const MODE2_INT_FLAG: Byte = 0b0010_0000;
const MODE1_INT_FLAG: Byte = 0b0001_0000;
const MODE0_INT_FLAG: Byte = 0b0000_1000;
const LYC_EQ_LY_FLAG: Byte = 0b0000_0100;

const SCANLINE_CYCLES: u128 = 114;

const BLACK: Color = Color::RGB(0, 0, 0);
const DARK_GREY: Color = Color::RGB(48, 48, 48);
const LIGHT_GREY: Color = Color::RGB(139, 139, 139);
const WHITE: Color = Color::RGB(255, 255, 255);

#[derive(Clone, Copy, Debug, PartialEq)]
enum PixelSource {
    /// When background is disabled
    Background {
        enabled: bool,
    },
    Object {
        number: usize,
    }, // object number
}

#[derive(Clone, Copy)]
pub struct Pixel {
    color_ref: u8, // should be u2
    pixel_source: PixelSource,
}

impl Pixel {
    fn new(color_ref: u8, pixel_source: PixelSource) -> Self {
        Self {
            color_ref,
            pixel_source,
        }
    }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

    pub fn get_range(&self, x: Range<usize>, y: usize) -> &[Pixel] {
        &self.tile[y][x]
    }

    pub fn flip_x(&mut self) {
        for row in self.tile.iter_mut() {
            row.reverse();
        }
    }

    pub fn flip_y(&mut self) {
        self.tile.reverse();
    }
}

pub trait FIFO {
    fn next_line(&mut self, memory: &Memory);
    fn pop(&mut self, memory: &Memory) -> Pixel;
}

struct BgFIFO {
    fifo: VecDeque<Pixel>,
    initialized: bool,
    lcdc: Byte,

    screen_pos: PixelPos,
    in_window: bool,
    tile_cache: HashMap<TilePos, Tile>,
}

impl BgFIFO {
    fn new() -> Self {
        let screen_pos = PixelPos::new();
        Self {
            fifo: VecDeque::new(),
            screen_pos,
            lcdc: 0,
            initialized: false,
            in_window: false,
            tile_cache: HashMap::new(),
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

    fn fetch(&mut self, memory: &Memory) {
        let lcdc = memory.read_byte(LCDC_ADDRESS);
        let window_enabled = get_flag(lcdc, BGW_ENABLE_FLAG);

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

            let tile = match self.tile_cache.entry(tile_pos) {
                Entry::Occupied(occ) => occ.into_mut(),
                Entry::Vacant(vacant) => {
                    let tile_idx = tile_pos.i + tile_pos.j * 32;
                    let tile_num_address = map_address + (tile_idx as Address);
                    let tile_num = memory.read_byte(tile_num_address);
                    let tile_start_address = if get_flag(lcdc, BGW_TILES_DATA_FLAG) {
                        0x8000 + BYTES_PER_TILE * (tile_num as Address)
                    } else {
                        let res = 0x9000 + (BYTES_PER_TILE as i32) * ((tile_num as i8) as i32);
                        res as Address
                    };

                    let tile = Tile::fetch_tile(
                        memory,
                        PixelSource::Background {
                            enabled: window_enabled,
                        },
                        tile_start_address,
                    );
                    vacant.insert(tile)
                }
            };

            let (tx, ty) = (fp.x % 8, fp.y % 8);
            let tile_line = tile.get_range(tx..8, ty);
            self.fifo.extend(tile_line);

            // if last line, clear cache
            if ty == 7 {
                self.tile_cache.remove(&tile_pos);
            }
        }
    }
}

impl FIFO for BgFIFO {
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
        self.lcdc = Graphics::get_lcdc(memory);

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Object {
    index: usize,
    x_pos: usize,
    y_pos: usize,
    tile_num: Address,
    flag: Byte,
}

impl Object {
    fn new(index: usize, x_pos: usize, y_pos: usize, tile_num: Address, flag: Byte) -> Self {
        Self {
            index,
            x_pos,
            y_pos,
            tile_num,
            flag,
        }
    }
}

pub struct ObjFIFO {
    fifo: VecDeque<Pixel>,
    lcdc: Byte,
    initialized: bool,
    screen_y: usize,
    obj_attr: HashMap<usize, Object>,
}

impl ObjFIFO {
    fn new() -> Self {
        Self {
            fifo: VecDeque::new(),
            lcdc: 0,
            screen_y: 0,
            initialized: false,
            obj_attr: HashMap::new(),
        }
    }
    fn merge(p1: Pixel, p2: Pixel) -> Pixel {
        if p1.color_ref == 0 {
            p2
        } else {
            p1
        }
    }
    fn get_obj_attr(&self, obj_index: usize) -> Object {
        self.obj_attr.get(&obj_index).unwrap().clone()
    }
}

impl FIFO for ObjFIFO {
    // must call before using, finds all objects that intersect
    fn next_line(&mut self, memory: &Memory) {
        self.screen_y = if self.initialized {
            self.screen_y + 1
        } else {
            self.initialized = true;
            self.screen_y
        };
        self.fifo.clear();
        self.obj_attr.clear();
        self.lcdc = Graphics::get_lcdc(memory);

        let mut line_pixels = [Pixel::new(0, PixelSource::Object { number: 0 }); SCREEN_WIDTH];

        if get_flag(self.lcdc, OBJ_ENABLE_FLAG) {
            // find all intersections
            for obj_idx in 0..OBJ_COUNT {
                let obj_address = OAM_ADDRESS + 4 * (obj_idx as Address);

                let y_pos = memory.read_byte(obj_address) as usize;
                let x_pos = memory.read_byte(obj_address + 1) as usize;
                let tile_number = memory.read_byte(obj_address + 2) as Address;
                let flag = memory.read_byte(obj_address + 3);

                // TODO: modify for 16x8 objects
                if y_pos <= self.screen_y + 16
                    && self.screen_y + 8 < y_pos
                    && !(x_pos == 0 || x_pos >= 168)
                {
                    let tile_start_address = OBJ_TILE_ADDRESS + BYTES_PER_TILE * tile_number;
                    let mut tile = Tile::fetch_tile(
                        memory,
                        PixelSource::Object { number: obj_idx },
                        tile_start_address,
                    );

                    if get_flag(flag, OBJ_XFLIP_FLAG) {
                        tile.flip_x();
                    }
                    if get_flag(flag, OBJ_YFLIP_FLAG) {
                        tile.flip_y();
                    }

                    let y = self.screen_y + 16 - y_pos;
                    let xrange = if x_pos < 8 {
                        8 - x_pos..8
                    } else if x_pos > SCREEN_WIDTH {
                        0..(8 + SCREEN_WIDTH) - x_pos
                    } else {
                        0..8
                    };

                    let tile_line = tile.get_range(0..8, y);
                    for d in xrange {
                        line_pixels[x_pos + d - 8] =
                            Self::merge(line_pixels[x_pos + d - 8], tile_line[d]);
                    }

                    self.obj_attr.insert(
                        obj_idx,
                        Object::new(obj_idx, x_pos, y_pos, tile_number, flag),
                    );
                }

                if self.obj_attr.len() >= 10 {
                    break;
                }
            }
        }

        self.fifo.extend(line_pixels);
    }

    fn pop(&mut self, _memory: &Memory) -> Pixel {
        let p = self.fifo.pop_front().unwrap();
        p
    }
}

/// PPU Mode with corresponding line number
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PPUMode {
    /// Horizontal BLANK
    Mode0 { line: usize },
    /// Vertical BLANK
    Mode1 { line: usize },
    /// OAM Scan
    Mode2 { line: usize },
    /// Drawing Pixels
    Mode3 { line: usize },
}

impl PPUMode {
    fn to_num(&self) -> Byte {
        match self {
            Self::Mode0 { .. } => 0,
            Self::Mode1 { .. } => 1,
            Self::Mode2 { .. } => 2,
            Self::Mode3 { .. } => 3,
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
    obj_fifo: ObjFIFO,
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
            obj_fifo: ObjFIFO::new(),
            last_ppu_mode: PPUMode::Mode1 { line: 153 },
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
            self.obj_fifo = ObjFIFO::new();
        }

        let clock_diff = timestamp - self.last_timestamp;
        let current_ppu_mode = self.get_mode(clock_diff);

        if self.last_ppu_mode != current_ppu_mode {
            // PPU Mode transitions
            match (self.last_ppu_mode, current_ppu_mode) {
                (PPUMode::Mode1 { line: l1 }, PPUMode::Mode2 { line: l2 })
                    if l1 == 153 && l2 == 0 =>
                {
                    // new frame
                    self.set_lyc(memory);
                }
                (PPUMode::Mode2 { line: l1 }, PPUMode::Mode3 { line: l2 }) if l1 == l2 => {
                    // draw scanline
                    self.draw_scanline(memory);
                }
                (PPUMode::Mode3 { line: l1 }, PPUMode::Mode0 { line: l2 }) if l1 == l2 => {
                    // finish draw pixel to hblank
                }
                (PPUMode::Mode0 { line: l1 }, PPUMode::Mode2 { line: l2 }) if l1 + 1 == l2 => {
                    // newline
                    self.set_lyc(memory);
                }
                (PPUMode::Mode0 { line: l1 }, PPUMode::Mode1 { line: l2 }) if l1 + 1 == l2 => {
                    // render to screen if vblank
                    self.set_lyc(memory);
                    self.set_vblank_int(memory);
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
                (PPUMode::Mode1 { line: l1 }, PPUMode::Mode1 { line: l2 }) if l1 + 1 == l2 => {
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
            PPUMode::Mode1 { line: self.line_y }
        } else if clock_diff <= 20 {
            PPUMode::Mode2 { line: self.line_y }
        } else if clock_diff < 77 {
            PPUMode::Mode3 { line: self.line_y }
        } else {
            PPUMode::Mode0 { line: self.line_y }
        }
    }

    fn draw_scanline(&mut self, memory: &mut Memory) {
        // draw line to screen_buffer
        self.bg_fifo.next_line(memory);
        self.obj_fifo.next_line(memory);
        for x in 0..SCREEN_WIDTH {
            let bg_pixel = self.bg_fifo.pop(memory);
            let obj_pixel = self.obj_fifo.pop(memory);
            let pixel = self.mix(bg_pixel, obj_pixel);
            let color = self.pixel_to_color(pixel, memory);

            let lcdc = Self::get_lcdc(memory);

            let color = if get_flag(lcdc, LCDC_ENABLE_FLAG) {
                color
            } else {
                BLACK
            };

            let offset = self.line_y * SCREEN_WIDTH * 3 + x * 3;
            self.screen_buffer[offset] = color.r;
            self.screen_buffer[offset + 1] = color.g;
            self.screen_buffer[offset + 2] = color.b;
        }
    }

    fn pixel_to_color(&self, pixel: Pixel, memory: &mut Memory) -> Color {
        let palette = match pixel.pixel_source {
            PixelSource::Background { enabled } => {
                let palette = memory.read_byte(BG_PALETTE_ADDRESS);
                if enabled {
                    palette
                } else {
                    // background is diabled, just use black
                    0xFF
                }
            }
            PixelSource::Object { number } => {
                let obj_flag = self.obj_fifo.get_obj_attr(number).flag;
                let palette = if get_flag(obj_flag, OBJ_PALETTE_FLAG) {
                    memory.read_byte(OBP1_ADDRESS)
                } else {
                    memory.read_byte(OBP0_ADDRESS)
                };
                // last one always 3 = black
                palette | 0b11
            }
        };

        let color_idx = match pixel.color_ref {
            0 => palette & 0b11,
            1 => (palette >> 2) & 0b11,
            2 => (palette >> 4) & 0b11,
            3 => (palette >> 6) & 0b11,
            _ => panic!(),
        };
        match color_idx {
            0 => WHITE,
            1 => LIGHT_GREY,
            2 => DARK_GREY,
            3 => BLACK,
            _ => panic!(),
        }
    }

    /// Set ppu stat flag and LCD interrupt flag
    fn set_ppu(&self, ppu_mode: PPUMode, memory: &mut Memory) {
        let stat_flag = memory.read_byte(LCD_STATUS_ADDRESS) & !0b11;
        let new_stat_flag = stat_flag | ppu_mode.to_num();

        // interrupt
        let mut int_flag = memory.read_byte(INTERRUPT_FLAG_ADDRESS);
        match ppu_mode {
            PPUMode::Mode0 { .. } if get_flag(stat_flag, MODE0_INT_FLAG) => {
                set_flag(&mut int_flag, LCD_FLAG);
            }
            PPUMode::Mode1 { .. } if get_flag(stat_flag, MODE1_INT_FLAG) => {
                set_flag(&mut int_flag, LCD_FLAG);
            }
            PPUMode::Mode2 { .. } if get_flag(stat_flag, MODE2_INT_FLAG) => {
                set_flag(&mut int_flag, LCD_FLAG);
            }
            _ => (),
        }
        memory.write_byte(INTERRUPT_FLAG_ADDRESS, int_flag);
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
                let mut int_flag = memory.read_byte(INTERRUPT_FLAG_ADDRESS);
                set_flag(&mut int_flag, LCD_FLAG);
                memory.write_byte(INTERRUPT_FLAG_ADDRESS, int_flag);
            }
        }
    }

    /// Set the vblank interrupt
    fn set_vblank_int(&self, memory: &mut Memory) {
        let mut int_flag = memory.read_byte(INTERRUPT_FLAG_ADDRESS);
        set_flag(&mut int_flag, VBLANK_FLAG);
        memory.write_byte(INTERRUPT_FLAG_ADDRESS, int_flag);
    }

    fn get_lcdc(memory: &Memory) -> Byte {
        memory.read_byte(LCDC_ADDRESS)
    }

    // Mixes Background pixel with Object Pixel
    fn mix(&self, bgp: Pixel, obp: Pixel) -> Pixel {
        match (bgp.pixel_source, obp.pixel_source) {
            (PixelSource::Background { enabled: b }, PixelSource::Object { number: o }) => {
                if obp.color_ref == 0 {
                    // transparent
                    bgp
                } else if !b {
                    obp
                } else {
                    let obj_attr = self.obj_fifo.get_obj_attr(o);
                    if get_flag(obj_attr.flag, OBJ_PRIORITY_FLAG) && bgp.color_ref >= 1 {
                        bgp
                    } else {
                        obp
                    }
                }
            }
            _ => panic!("Mix usage: (Background pixel, object pixel)"),
        }
    }
}
