use sdl2::{
    pixels::Color,
    rect::Rect,
    render::{Canvas, Texture, TextureCreator},
    surface::Surface,
    video::{Window, WindowContext},
    EventPump, Sdl,
};

use crate::{
    memory::Memory,
    utils::{address2string, byte2string, Address, Byte},
};

pub struct Graphics {
    pub context: Sdl,
    pub canvas: Canvas<Window>,
    pub event_pump: EventPump,
    pub texture_creator: TextureCreator<WindowContext>,
    pub i: usize,
}

impl Graphics {
    const LCDC_ADDRESS: Address = 0xFF40;
    const LCDC_ENABLE_FLAG: Byte = 0b1000_0000;
    const WINDOW_AREA_FLAG: Byte = 0b0100_0000;
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

    const BYTES_PER_TILE: Address = 16;

    pub fn new() -> Self {
        // Initialize SDL
        let context = sdl2::init().unwrap();

        // Set hint for vsync
        sdl2::hint::set("SDL_HINT_RENDER_VSYNC", "1");

        // Create window and renderer
        let video_subsystem = context.video().unwrap();
        let window = video_subsystem
            .window("GB-rs", 160, 144)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Self::BLACK);
        canvas.clear();

        let event_pump = context.event_pump().unwrap();

        let texture_creator = canvas.texture_creator();

        Self {
            context,
            canvas,
            event_pump,
            texture_creator,
            i: 0,
        }
    }

    pub fn render(&mut self, memory: &mut Memory) {
        self.i += 1;
        if self.i % 1000 != 0 {
            return;
        }
        self.i = 0;
        let lcdc = memory.read_byte_unsafe(Self::LCDC_ADDRESS);

        if !Memory::get_flag(lcdc, Self::LCDC_ENABLE_FLAG) {
            self.canvas.set_draw_color(Self::BLACK);
            self.canvas.clear();

            self.canvas.present();
            return;
        }

        let bcg_data_address = if Memory::get_flag(lcdc, Self::BGW_TILES_DATA_FLAG) {
            0x8000
        } else {
            0x8800
        };
        let bcg_map_address = if Memory::get_flag(lcdc, Self::BG_TILE_MAP_FLAG) {
            0x9C00
        } else {
            0x9800
        };

        // load textures
        let mut textures = Vec::with_capacity(128);
        for i in 0..128 {
            let mut data_vec = Vec::with_capacity(64 * 3);

            let texture_start_address = bcg_data_address + Self::BYTES_PER_TILE * i;

            // if i == 1 {
            //     println!("Texture start: {}", address2string(texture_start_address));
            // }

            for d in 0..8 {
                let lsb_address = texture_start_address + 2 * d;
                let msb_address = texture_start_address + 2 * d + 1;

                let lsb = memory.read_byte_unsafe(lsb_address);
                let msb = memory.read_byte_unsafe(msb_address);

                for j in (0..8).rev() {
                    let color_idx = ((msb >> j) & 1) * 2 + ((lsb >> j) & 1);
                    let mut color = match color_idx {
                        0 => Self::color2vec(Self::WHITE),
                        1 => Self::color2vec(Self::LIGHT_GREY),
                        2 => Self::color2vec(Self::DARK_GREY),
                        3 => Self::color2vec(Self::BLACK),
                        _ => panic!("Logical error"),
                    };
                    // if i == 1 {
                    //     print!("{}", color_idx);
                    // }
                    data_vec.append(&mut color);
                }
                // if i == 1 {
                //     println!();
                // }
            }

            let surface = Surface::from_data(
                data_vec.as_mut_slice(),
                8,
                8,
                24,
                sdl2::pixels::PixelFormatEnum::RGB24,
            )
            .unwrap();
            let texture = Texture::from_surface(&surface, &self.texture_creator).unwrap();
            textures.push(texture);
        }

        // 256x256 background
        for i in 0..32 {
            for j in 0..32 {
                let map_idx = 32 * i + j;
                let (x, y) = (8 * j, 8 * i);

                let texture_num = memory.read_byte_unsafe(bcg_map_address + map_idx);
                // let texture_num = if map_idx >= 128 { 127 } else { map_idx };
                self.canvas
                    .copy(
                        &textures[texture_num as usize],
                        None,
                        Rect::new(x.into(), y.into(), 8, 8),
                    )
                    .unwrap();
            }
        }
        // println!(
        //     "{}: {}, {}",
        //     byte2string(lcdc),
        //     address2string(bcg_data_address),
        //     address2string(bcg_data_address + Self::BYTES_PER_TILE * 128)
        // );
        //
        // println!(
        //     "{}: {}, {}",
        //     byte2string(lcdc),
        //     address2string(bcg_map_address),
        //     address2string(bcg_map_address + 32 * 32)
        // );
        self.canvas.present();
    }

    fn color2vec(color: Color) -> Vec<u8> {
        vec![color.r, color.g, color.b]
    }
}
