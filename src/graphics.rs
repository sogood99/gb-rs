use sdl2::{pixels::Color, render::Canvas, video::Window, EventPump, Sdl};

pub struct Graphics {
    pub context: Sdl,
    pub canvas: Canvas<Window>,
    pub event_pump: EventPump,
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
            .window("GB-rs", 480, 432)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();

        let event_pump = context.event_pump().unwrap();

        Self {
            context,
            canvas,
            event_pump,
        }
    }

    pub fn render(&mut self) {
        self.canvas.present();
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        self.canvas.clear();

        self.canvas.present();
    }
}
