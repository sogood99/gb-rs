use log::info;

use crate::utils::{address2string, bytes2word, Address, Byte, Word};

const BOOTROM_SIZE: usize = 0x100;
const MEMORY_SIZE: usize = 0x10000;
const ROM_SIZE: usize = 0x4000;

const DMA_ADDRESS: Address = 0xFF46;
const MBC_TYPE_ADDRESS: Address = 0x0147;
const ROM_SIZE_ADDRESS: Address = 0x0148;
const RAM_SIZE_ADDRESS: Address = 0x0149;

const ROM_ONLY_SIZE: usize = 0x8000;

const UNLOAD_BOOT_ADDRESS: usize = 0xFF50;

#[derive(Debug, PartialEq, Eq)]
pub enum CartridgeType {
    RomOnly,
    MBC1,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CartridgeState {
    None,
    RomOnly(RomState),
    MBC1(MBC1State),
}

#[derive(Debug, PartialEq, Eq)]
pub struct RomState {}

#[derive(Debug, PartialEq, Eq)]
pub struct MBC1State {
    ram_enabled: bool,
    rom_number: usize,
    ram_number: usize,
}

impl MBC1State {
    fn new() -> Self {
        Self {
            rom_number: 1,
            ram_enabled: false,
            ram_number: 0,
        }
    }
}

pub struct Memory {
    memory: [Byte; MEMORY_SIZE],
    boot_rom: [Byte; BOOTROM_SIZE],
    rom: Vec<Vec<Byte>>,
    ram: Vec<Vec<Byte>>,
    cartridge: CartridgeState,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            memory: [0; MEMORY_SIZE],
            boot_rom: [0; BOOTROM_SIZE],
            rom: Vec::new(),
            ram: Vec::new(),
            cartridge: CartridgeState::None,
        }
    }

    pub fn load_cartidge(&mut self, rom_data: Vec<u8>) {
        let ctype = self.get_cartridge_type_rom(&rom_data);
        let rom_size = self.get_rom_size_rom(&rom_data);
        let ram_size = self.get_ram_size_rom(&rom_data);
        info!("Load Rom Size {:#04X?}", rom_data.len(),);
        info!("Rom Type {:?}", ctype);
        info!("Rom Size {:?}", rom_size);
        info!("Ram Size {:?}", ram_size);

        self.cartridge = match ctype {
            CartridgeType::RomOnly => CartridgeState::RomOnly(RomState {}),
            CartridgeType::MBC1 => CartridgeState::MBC1(MBC1State::new()),
        };

        // copy rom_data to self.rom
        let rom_data = rom_data.as_slice();

        let rom_bank_num = 1 << (rom_size + 1);
        for i in 0..rom_bank_num {
            let mut rom_bank = Vec::with_capacity(ROM_SIZE);
            rom_bank.extend_from_slice(&rom_data[ROM_SIZE * i..ROM_SIZE * (i + 1)]);
            self.rom.push(rom_bank);
        }
        self.memory[BOOTROM_SIZE..ROM_SIZE].copy_from_slice(&self.rom[0][BOOTROM_SIZE..ROM_SIZE]);
        self.memory[ROM_SIZE..ROM_SIZE * 2].copy_from_slice(&self.rom[1]);
    }

    pub fn load_boot(&mut self, boot_data: Vec<u8>) {
        info!("Boot Size {:#04X?}", boot_data.len());
        self.boot_rom.copy_from_slice(&boot_data);
        self.memory[..BOOTROM_SIZE].copy_from_slice(&self.boot_rom);
    }

    fn unload_boot(&mut self) {
        info!("Unloading boot rom");
        self.memory[..BOOTROM_SIZE].copy_from_slice(&self.rom[0][..BOOTROM_SIZE]);
    }

    pub fn read_byte(&self, address: Address) -> Byte {
        let address = address as usize;
        self.memory[address]
    }

    pub fn read_word(&self, address: Address) -> Word {
        let address = address as usize;
        bytes2word(self.memory[address], self.memory[address + 1])
    }

    /// Write byte to address according to MMU
    pub fn write_byte(&mut self, address: Address, byte: Byte) {
        let address = address as usize;

        if address == UNLOAD_BOOT_ADDRESS {
            self.unload_boot();
        }

        let ctype = self.get_cartridge_type();
        match ctype {
            CartridgeType::RomOnly => {
                if address >= 0x8000 {
                    self.memory[address] = byte;
                }
            }
            CartridgeType::MBC1 => {
                if address >= 0x8000 {
                    self.memory[address] = byte;
                } else if address < 0x8000 {
                    unimplemented!("{}", address2string(address as Address));
                }
            }
        }
    }

    /// Get cartridge type from memory
    pub fn get_cartridge_type(&self) -> CartridgeType {
        match self.cartridge {
            CartridgeState::None => panic!("Cartridge not loaded"),
            CartridgeState::RomOnly(_) => CartridgeType::RomOnly,
            CartridgeState::MBC1(_) => CartridgeType::MBC1,
        }
    }

    /// Get cartridge type given rom (in vec)
    pub fn get_cartridge_type_rom(&self, rom: &Vec<Byte>) -> CartridgeType {
        let rom_type = rom[MBC_TYPE_ADDRESS as usize];
        match rom_type {
            0x00 => CartridgeType::RomOnly,
            0x01 => CartridgeType::MBC1,
            _ => unimplemented!(),
        }
    }

    /// Get rom size
    pub fn get_rom_size_rom(&self, rom: &Vec<Byte>) -> usize {
        let rom_size = rom[ROM_SIZE_ADDRESS as usize].into();
        return rom_size;
    }

    /// Get ram size
    pub fn get_ram_size_rom(&self, rom: &Vec<Byte>) -> usize {
        let ram_size = rom[RAM_SIZE_ADDRESS as usize].into();
        return ram_size;
    }

    /// Wrapping add value to address
    pub fn wrapping_add(&mut self, address: Address, value: Byte) {
        assert!((address as usize) < MEMORY_SIZE);
        let mut mem_val = self.read_byte(address);
        mem_val = mem_val.wrapping_add(value);
        self.write_byte(address, mem_val);
    }
}
