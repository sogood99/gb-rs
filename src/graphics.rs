use sdl2::{pixels::Color, render::Canvas, video::Window};

pub struct Graphics {
    pub canvas: Canvas<Window>,
}

impl Graphics {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().expect("could not initialize sdl2");
        let video_subsystem = sdl_context
            .video()
            .expect("could not initialize video subsystem");

        let window = video_subsystem
            .window("GameBoy Emulator", 800, 600)
            .position_centered()
            .build()
            .expect("could not initialize video subsystem");

        let mut canvas = window
            .into_canvas()
            .build()
            .expect("could not build canvas");
        canvas.set_draw_color(Color::RGB(255, 0, 0));
        canvas.clear();
        canvas.present();
        Self { canvas }
    }
}
