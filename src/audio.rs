use sdl2::{AudioSubsystem, Sdl};

use crate::memory::Memory;

pub struct Audio {
    audio: AudioSubsystem,
}

impl Audio {
    pub fn new(context: &Sdl) -> Self {
        Audio {
            audio: context.audio().unwrap(),
        }
    }

    pub fn handle_audio(memory: &mut Memory) {}
}
