use log::info;
use sdl2::{
    audio::{AudioQueue, AudioSpecDesired},
    Sdl, TimerSubsystem,
};

use crate::{
    clock::{Clock, CLOCK_FREQ},
    memory::Memory,
    utils::{bytes2word, get_flag, reset_flag, Address, Byte},
};

const DUTY_WAVES: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 12.5% duty wave
    [0, 0, 0, 0, 0, 0, 1, 1], // 25% duty wave
    [0, 0, 0, 0, 1, 1, 1, 1], // 50% duty wave
    [1, 1, 1, 1, 1, 1, 0, 0], // 75% duty wave
];

const WAVE_REGISTER_ADDRESS: Address = 0xFF30;
const MASTER_CONTROL_ADDRESS: Address = 0xFF26;
const AUDIO_ENABLE_FLAG: Byte = 0b1000_0000;
const PULSE_A_ENABLE_FLAG: Byte = 0b0000_0001;
const PULSE_B_ENABLE_FLAG: Byte = 0b0000_0010;
const WAVE_ENABLE_FLAG: Byte = 0b0000_0100;
const NOISE_ENABLE_FLAG: Byte = 0b0000_1000;

const C1_LENGTH_DUTY_CYCLE: Address = 0xFF11;
const C1_VOLUME_ENVOLOPE: Address = 0xFF12;
const C1_PERIOD_LOW: Address = 0xFF13;
const C1_PERIOD_HIGH_CONTROL: Address = 0xFF14;

const C1_LENGTH_ENABLE_FLAG: Byte = 0b0100_0000;
const TRIGGER_FLAG: Byte = 0b1000_0000;

const AUDIO_FREQ: u32 = 44100;
const CYCLE_PER_SAMPLE: u32 = CLOCK_FREQ / AUDIO_FREQ;
const MAX_LENGTH: u32 = 64;

/// Enum representing the 4 audio channels
pub enum Channel {
    PulseA,
    PulseB,
    Wave,
    Noise,
}

trait AudioChannel {
    fn step(&mut self, memory: &mut Memory);
    fn sample(&self) -> f32;
}

struct PulseA {
    /// how long one wave is (in terms of cpu cycles)
    period: u128,
    /// current tick inside each wave
    tick: u128,

    duty_wave: usize,
    duty_step: usize,

    volume: f32,

    /// Length register that auto ends
    length: u32,
    length_enable: bool,

    enabled: bool,
}

impl PulseA {
    fn new() -> Self {
        Self {
            period: 0,
            tick: 0,
            duty_wave: 0,
            duty_step: 0,
            volume: 0.2,
            length: 0,
            length_enable: false,
            enabled: true,
        }
    }

    fn initialize_volume(&mut self, memory: &mut Memory) {
        let flag = memory.read_byte(C1_VOLUME_ENVOLOPE);
        let volume = flag >> 4;
        self.volume = volume as f32 / 100.;
    }

    fn update_period(&mut self, memory: &mut Memory) {
        let msb = memory.read_byte(C1_PERIOD_HIGH_CONTROL) & 0b111;
        let lsb = memory.read_byte(C1_PERIOD_LOW);

        let freq = bytes2word(lsb, msb);
        self.period = (2048 - freq as u128) * 4;
    }

    fn update_duty_cycle(&mut self, memory: &mut Memory) {
        let value = memory.read_byte(C1_LENGTH_DUTY_CYCLE) >> 6;
        self.duty_wave = value as usize;
    }

    fn check_trigger(&self, memory: &mut Memory) -> bool {
        let mut flag_byte = memory.read_byte(C1_PERIOD_HIGH_CONTROL);

        if get_flag(flag_byte, TRIGGER_FLAG) {
            reset_flag(&mut flag_byte, TRIGGER_FLAG);
            memory.write_byte(C1_PERIOD_HIGH_CONTROL, flag_byte);
            true
        } else {
            false
        };

        false
    }

    fn update_length_enable(&mut self, memory: &mut Memory) {
        let flag_byte = memory.read_byte(C1_PERIOD_HIGH_CONTROL);
        let length_enable = get_flag(flag_byte, C1_LENGTH_ENABLE_FLAG);

        match (self.length_enable, length_enable) {
            (false, true) => {
                info!("LENGTH ENABLED {}", self.length);
            }
            _ => (),
        }

        self.length_enable = length_enable;
    }
}

impl AudioChannel for PulseA {
    fn step(&mut self, memory: &mut Memory) {
        self.update_period(memory);
        if self.check_trigger(memory) {
            self.enabled = !self.enabled;
        }
        self.initialize_volume(memory);
        self.update_duty_cycle(memory);

        if self.tick >= self.period {
            self.duty_step = (self.duty_step + 1) % 8;
            self.tick = 0;
            info!(
                "{:?} {:?} {:?} {:?} {:?}",
                self.period, self.enabled, self.volume, self.duty_wave, self.duty_step
            );
        }

        self.tick += 1;
    }

    fn sample(&self) -> f32 {
        if self.enabled {
            (DUTY_WAVES[self.duty_wave][self.duty_step] as f32 - 0.5) * 2. * self.volume
        } else {
            0.0
        }
    }
}

pub struct Audio {
    pub timer: TimerSubsystem,
    device: AudioQueue<f32>,
    pulse_a: PulseA,
    last_timestamp: u128,
    buffer: Vec<f32>,
}

impl Audio {
    pub fn new(context: &Sdl) -> Self {
        let desired_spec = AudioSpecDesired {
            freq: Some(AUDIO_FREQ as i32),
            channels: Some(2),
            samples: None,
        };
        let audio = context.audio().unwrap();
        let timer = context.timer().unwrap();
        let device: AudioQueue<f32> = audio.open_queue(None, &desired_spec).unwrap();
        let pulse_a = PulseA::new();
        device.resume();

        Audio {
            timer,
            device,
            pulse_a,
            last_timestamp: 0,
            buffer: Vec::new(),
        }
    }

    pub fn handle_audio(&mut self, memory: &mut Memory, clock: &Clock) {
        let clock_ticks = clock.get_timestamp() - self.last_timestamp;

        for _i in 0..clock_ticks {
            self.last_timestamp += 1;

            self.pulse_a.step(memory);

            if self.last_timestamp % CYCLE_PER_SAMPLE as u128 == 0 {
                let data = if self.audio_enabled(memory) {
                    let a_sample = self.pulse_a.sample();
                    a_sample
                } else {
                    0.0
                };
                self.buffer.push(data);
            }
        }

        if self.buffer.len() > 1000 {
            self.device.queue_audio(&self.buffer).unwrap();

            self.buffer.clear();
        }
    }

    fn audio_enabled(&self, memory: &mut Memory) -> bool {
        memory.read_byte(MASTER_CONTROL_ADDRESS) & AUDIO_ENABLE_FLAG > 0
    }
}
