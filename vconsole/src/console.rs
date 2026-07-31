use crate::renderer::Renderer;
use std::{
    mem::take,
    ops::{BitAnd, BitOr, BitXor, Shl, Shr, Sub},
};
use winit::{dpi::PhysicalSize, keyboard};

#[repr(u8)]
#[derive(PartialEq, Eq, Debug)]
enum OpCode {
    NoOp,
    Add,
    Sub,
    Mul,
    Div,
    Stall,
    Await,
    Mov,
    Jmp,
    Jeq,
    Jne,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Push,
    Pop,
    Jsr,
    Rtr,
}
impl OpCode {
    fn from_u8(bytecode: u8) -> Result<OpCode, String> {
        match bytecode {
            0 => Ok(OpCode::NoOp),
            1 => Ok(OpCode::Add),
            2 => Ok(OpCode::Sub),
            3 => Ok(OpCode::Mul),
            4 => Ok(OpCode::Div),
            5 => Ok(OpCode::Stall),
            6 => Ok(OpCode::Await),
            7 => Ok(OpCode::Mov),
            8 => Ok(OpCode::Jmp),
            9 => Ok(OpCode::Jeq),
            10 => Ok(OpCode::Jne),
            11 => Ok(OpCode::And),
            12 => Ok(OpCode::Or),
            13 => Ok(OpCode::Xor),
            14 => Ok(OpCode::Shl),
            15 => Ok(OpCode::Shr),
            16 => Ok(OpCode::Jsr),
            17 => Ok(OpCode::Rtr),
            other => Err(format!("Unknown OpCode: {other}")),
        }
    }
}

#[repr(u16)]
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ArgumentType {
    Empty,
    Immediate,              // Contains a Value
    Address,                // uses a direct value to point to the memory location
    RegisterPointedAddress, // uses the value of the register to point to the memory location
    Register,               // Points to one of the registers
}
impl ArgumentType {
    fn from_u8(bytecode: u8) -> Result<ArgumentType, String> {
        match bytecode {
            0 => Ok(ArgumentType::Empty),
            1 => Ok(ArgumentType::Immediate),
            2 => Ok(ArgumentType::Address),
            3 => Ok(ArgumentType::Register),
            4 => Ok(ArgumentType::RegisterPointedAddress),
            other => Err(format!("Unknown ArgumentType: {} (0 - 5)", other)),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Argument {
    arg_type: ArgumentType,
    value: u16,
}
impl Argument {
    pub fn new(arg_type: ArgumentType, value: u16) -> Self {
        Self { arg_type, value }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Instruction {
    opcode: OpCode,
    arg1: Argument,
    arg2: Argument,
}

impl Instruction {
    fn from_u16(source: &[u16; 3]) -> Result<Instruction, String> {
        // they are all bytes but arg0 and arg1 are only u4
        let (opcode_byte, args_byte) = ((source[0] >> 8) as u8, (source[0] & 0xff) as u8);
        let (arg1_type_byte, arg2_type_byte) = ((args_byte >> 4), args_byte & 0b00001111);

        Ok(Self {
            opcode: OpCode::from_u8(opcode_byte)?,
            arg1: Argument::new(ArgumentType::from_u8(arg1_type_byte)?, source[1]),
            arg2: Argument::new(ArgumentType::from_u8(arg2_type_byte)?, source[2]),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
enum MemoryPois {
    MajorConsoleVersion = 0x0000,
    MinorConsoleVersion = 0x0001,
    PatchConsoleVersion = 0x0002,
    MajorROMVersion = 0x0003,
    MinorROMVersion = 0x0004,
    PatchROMVersion = 0x0005,
    CPUCycleCounter = 0x0006,
    ProgramCounter = 0x0007,
    VblankFlag = 0x0008,
    CurrentPressedKeyASCII = 0x0009,
}

pub struct Console {
    renderer: Renderer,
    rom: Vec<u16>,
    sram: [u16; u16::MAX as usize],
    program_counter: u16,
    currently_awaiting: Option<u16>,
    stall_timer: u16,
    registers: [u16; 5],
    last_pressed_key: Option<keyboard::Key>,
}

impl Console {
    pub fn new() -> Self {
        Self {
            renderer: Renderer::new(),
            rom: Vec::new(),
            sram: [0; u16::MAX as usize],
            currently_awaiting: None,
            program_counter: 0,
            stall_timer: 0,
            registers: [0; 5],
            last_pressed_key: None,
        }
    }
    pub fn default() -> Self {
        Self::new()
    }
    pub fn is_rom_loaded(&self) -> bool {
        !self.rom.is_empty()
    }
    /// Creates everything to be able to render
    pub fn resume_renderer(&mut self, pixel_buffer: pixels::Pixels<'static>) {
        self.renderer.resume(pixel_buffer);
    }
    pub fn render(&mut self) {
        self.renderer.render();
    }
    pub fn resize_renderer(&mut self, size: PhysicalSize<u32>) {
        self.renderer.resize_surface(size.width, size.height);
    }
    pub fn key_pressed(&mut self, key: keyboard::Key) {
        if let Some(text) = key.to_text()
            && text.is_ascii()
        {
            let ascii = text
                .bytes()
                .next()
                .expect("Key cannot be converted to bytes");

            self.sram[MemoryPois::CurrentPressedKeyASCII as usize] = ascii as u16;

            self.last_pressed_key = Some(key);
        }
    }
    pub fn key_released(&mut self, key: keyboard::Key) {
        if self.last_pressed_key == Some(key) {
            self.sram[MemoryPois::CurrentPressedKeyASCII as usize] = 0;
        }
    }
    pub fn step(&mut self) {
        const STEP_RATE: f32 = 30.0; // steps per second
        const MHZ: f32 = 2.0;

        // setting the vblank flag
        self.sram[MemoryPois::VblankFlag as usize] = 0x0001;

        let mut step_counter: u32 = 0;
        while step_counter < ((MHZ * 1_000_000.0) / STEP_RATE) as u32 {
            // incrementing the cpu counter (it is just for user purposes, so it doesn't need to exist outside ram)
            self.sram[MemoryPois::CPUCycleCounter as usize] =
                self.sram[MemoryPois::CPUCycleCounter as usize].wrapping_add(1);

            if let Some(awaiting) = self.currently_awaiting {
                if self.sram[awaiting as usize] == 0 {
                    break;
                } else {
                    self.currently_awaiting = None
                }
            }

            // we just skip the current cpu cycle if we are still stalling
            if self.stall_timer > 0 {
                self.stall_timer -= 1;
                continue;
            }

            // incrementing program counter
            self.sram[MemoryPois::ProgramCounter as usize] = self.program_counter;

            let instruction = self.parse_next_instruction();

            let mut jumped = false;

            match instruction.opcode {
                OpCode::NoOp => {}
                OpCode::Add
                | OpCode::Sub
                | OpCode::Mul
                | OpCode::Div
                | OpCode::And
                | OpCode::Or
                | OpCode::Xor
                | OpCode::Shl
                | OpCode::Shr => {
                    let val1 = self.read_arg(&instruction.arg1);
                    let val2 = self.read_arg(&instruction.arg2);

                    let result = match instruction.opcode {
                        OpCode::Add => val1.wrapping_add(val2),
                        OpCode::Sub => val1.wrapping_sub(val2),
                        OpCode::Mul => val1.wrapping_mul(val2),
                        OpCode::Div => val1.wrapping_div(val2),
                        OpCode::And => val1.bitand(val2),
                        OpCode::Or => val1.bitor(val2),
                        OpCode::Xor => val1.bitxor(val2),
                        OpCode::Shl => val1.shl(val2),
                        OpCode::Shr => val1.shr(val2),
                        _ => unreachable!(),
                    };

                    self.write_arg(&instruction.arg1, result);
                }
                OpCode::Await => self.currently_awaiting = Some(self.read_arg(&instruction.arg1)),
                OpCode::Stall => self.stall_timer = self.read_arg(&instruction.arg1),
                // TODO: add longjumps
                OpCode::Jmp => {
                    self.program_counter = self.read_arg(&instruction.arg1);
                    jumped = true;
                }
                OpCode::Mov => {
                    let val2 = self.read_arg(&instruction.arg2);
                    self.write_arg(&instruction.arg1, val2);
                }
                OpCode::Jeq => {
                    if self.read_arg(&instruction.arg2) == 0 {
                        self.program_counter = self.read_arg(&instruction.arg1);
                        jumped = true;
                    }
                }
                OpCode::Jne => {
                    if self.read_arg(&instruction.arg2) != 0 {
                        self.program_counter = self.read_arg(&instruction.arg1);
                        jumped = true;
                    }
                }
                OpCode::Push => {
                    self.push_stack(self.read_arg(&instruction.arg1));
                }
                OpCode::Pop => {
                    let value = self.pop_stack();
                    self.write_arg(&instruction.arg1, value);
                }
                OpCode::Jsr => {
                    self.push_stack(self.program_counter);
                    self.program_counter = self.read_arg(&instruction.arg1);
                }
                OpCode::Rtr => self.program_counter = self.pop_stack(),
            }
            if !jumped {
                self.program_counter += 1;
            }
            // check if we reached the end of the program
            if self.program_counter as usize >= self.rom.len() / 3 {
                self.program_counter = 0;
            }

            if step_counter == 0 {
                // setting the vblank flag back to 0
                self.sram[MemoryPois::VblankFlag as usize] = 0x0000;
            }

            step_counter += 1;
        }

        self.renderer.draw(&self.sram);
    }
    fn read_arg(&self, arg: &Argument) -> u16 {
        match arg.arg_type {
            ArgumentType::Empty => panic!("Can't read from empty"),
            ArgumentType::Immediate => arg.value,
            ArgumentType::Address => self.read_ram(arg.value),
            ArgumentType::Register => {
                if (arg.value as usize) >= self.registers.len() {
                    panic!("Register address out of bounds: {}", arg.value)
                }
                self.registers[arg.value as usize]
            }
            ArgumentType::RegisterPointedAddress => {
                if (arg.value as usize) >= self.registers.len() {
                    panic!("Register address out of bounds: {}", arg.value)
                }
                let register = self.registers[arg.value as usize];
                self.read_ram(register)
            }
        }
    }
    fn write_arg(&mut self, arg: &Argument, value: u16) {
        match arg.arg_type {
            ArgumentType::Empty => panic!("Can't write to empty"),
            ArgumentType::Immediate => {
                panic!("Can't write to Immediate")
            }
            ArgumentType::Address => self.write_ram(arg.value, value),
            ArgumentType::Register => {
                if (arg.value as usize) >= self.registers.len() {
                    panic!("Register address out of bounds: {}", arg.value)
                }
                self.registers[arg.value as usize] = value
            }
            ArgumentType::RegisterPointedAddress => {
                if (arg.value as usize) >= self.registers.len() {
                    panic!("Register address out of bounds: {}", arg.value)
                }
                let register = self.registers[arg.value as usize];
                self.write_ram(register, value)
            }
        }
    }
    #[inline(always)]
    fn read_ram(&self, address: u16) -> u16 {
        self.sram[address as usize]
    }
    #[inline(always)]
    fn write_ram(&mut self, address: u16, value: u16) {
        if address >= 300 {
            self.sram[address as usize] = value;
        } else {
            panic!("Tried to write to read-only ram, address: {address}")
        }
    }
    #[inline(always)]
    fn push_stack(&mut self, value: u16) {
        self.registers[4] += 1;
        self.write_ram(u16::MAX - self.registers[4], value);
    }
    #[inline(always)]
    fn pop_stack(&mut self) -> u16 {
        self.registers[4] -= 1;
        self.read_ram((u16::MAX - 1) - self.registers[4])
    }
    #[inline(always)]
    fn parse_next_instruction(&mut self) -> Instruction {
        Instruction::from_u16(
            &self
                .rom
                .get((self.program_counter as usize * 3)..(self.program_counter as usize * 3) + 3)
                .unwrap_or_else(|| {
                    panic!(
                        "Program Pointer went out of bounds; ROM len: {}, Program Pointer: {}",
                        self.rom.len(),
                        self.program_counter,
                    );
                })
                .try_into()
                .unwrap(),
        )
        .unwrap()
    }
    pub fn load_rom_from_bytes(&mut self, bytes: Vec<u8>) -> anyhow::Result<()> {
        compare_version(&bytes[17..20]);

        self.rom = bytes
            .get(20..) // skip header (magic + version)
            .unwrap_or(&[])
            .chunks_exact(2)
            .map(|v| u16::from_le_bytes([v[0], v[1]]))
            .collect();
        // initializing read only flags
        self.sram[MemoryPois::MajorConsoleVersion as usize] =
            env!("CARGO_PKG_VERSION_MAJOR").parse::<u16>().unwrap();
        self.sram[MemoryPois::MinorConsoleVersion as usize] =
            env!("CARGO_PKG_VERSION_MINOR").parse::<u16>().unwrap();
        self.sram[MemoryPois::PatchConsoleVersion as usize] =
            env!("CARGO_PKG_VERSION_PATCH").parse::<u16>().unwrap();
        self.sram[MemoryPois::MajorROMVersion as usize] = bytes[17] as u16;
        self.sram[MemoryPois::MinorROMVersion as usize] = bytes[18] as u16;
        self.sram[MemoryPois::PatchROMVersion as usize] = bytes[19] as u16;

        // initializing palette
        self.sram[301] = 0x18E5; // Index 1:  Deep Night
        self.sram[302] = 0xF7BF; // Index 2:  Cloud White
        self.sram[303] = 0x424A; // Index 3:  Charcoal
        self.sram[304] = 0xA535; // Index 4:  Silver
        self.sram[305] = 0xFA50; // Index 5:  Cyber Pink
        self.sram[306] = 0x347F; // Index 6:  Ocean Blue
        self.sram[307] = 0x3EBE; // Index 7:  Sky Cyan
        self.sram[308] = 0x8787; // Index 8:  Slime Green
        self.sram[309] = 0xBAFD; // Index 9:  Magic Violet
        self.sram[310] = 0x6ADE; // Index 10: Electric Indigo
        self.sram[311] = 0xECAF; // Index 11: Toasted Peach
        self.sram[312] = 0xFB4B; // Index 12: Neon Coral
        self.sram[313] = 0xFDC6; // Index 13: Sunny Amber
        self.sram[314] = 0xF747; // Index 14: Electric Lemon
        self.sram[315] = 0x2EF1; // Index 15: Minty Green

        Ok(())
    }
}

/// Compares version bytes with the version of the console
fn compare_version(version_bytes: &[u8]) {
    assert_eq!(version_bytes.len(), 3);
    let major_version = env!("CARGO_PKG_VERSION_MAJOR").parse::<u8>().unwrap();
    let minor_version = env!("CARGO_PKG_VERSION_MINOR").parse::<u8>().unwrap();
    let patch_version = env!("CARGO_PKG_VERSION_PATCH").parse::<u8>().unwrap();

    if major_version != version_bytes[0] || minor_version != version_bytes[1] {
        panic!(
            "Rom is not compatible (console_ver: {}, bin_ver: {}.{}.{})",
            env!("CARGO_PKG_VERSION"),
            version_bytes[0],
            version_bytes[1],
            version_bytes[2]
        )
    }
    if patch_version != version_bytes[2] {
        println!(
            "Rom is only partially compatible (console_ver: {}, bin_ver: {:?})",
            env!("CARGO_PKG_VERSION"),
            version_bytes
        )
    }
}
