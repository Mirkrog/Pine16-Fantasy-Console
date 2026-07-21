use byteorder::{LittleEndian, ReadBytesExt};
use simple_stopwatch::Stopwatch;
use std::sync::Arc;
use winit::{dpi::PhysicalSize, window::Window};

use std::{
    fs::File,
    io::{BufReader, Read},
    io::{Seek, SeekFrom},
};

use crate::renderer::Renderer;

#[repr(u8)]
#[derive(PartialEq, Eq, Debug)]
enum OpCode {
    NoOp,
    Add,
    Sub,
    Mul,
    Div,
    Exit,
    Stall,
    Mov,
    Jmp,
    Jeq,
    Jne,
}
impl OpCode {
    fn from_u8(bytecode: u8) -> Result<OpCode, String> {
        match bytecode {
            0 => Ok(OpCode::NoOp),
            1 => Ok(OpCode::Add),
            2 => Ok(OpCode::Sub),
            3 => Ok(OpCode::Mul),
            4 => Ok(OpCode::Div),
            5 => Ok(OpCode::Exit),
            6 => Ok(OpCode::Stall),
            7 => Ok(OpCode::Mov),
            8 => Ok(OpCode::Jmp),
            9 => Ok(OpCode::Jeq),
            10 => Ok(OpCode::Jne),
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
}

pub struct Console {
    renderer: Renderer,
    rom: Vec<u16>,
    sram: Vec<u16>,
    program_counter: u16,
    stall_timer: u16,
    exit_code: u16,
    registers: [u16; 4],
}

impl Console {
    pub fn new(sram_size: usize) -> Self {
        Self {
            renderer: Renderer::new(),
            rom: Vec::new(),
            sram: vec![0x0000a; sram_size],
            program_counter: 0,
            stall_timer: 0,
            exit_code: 0,
            registers: [0; 4],
        }
    }
    pub fn default() -> Self {
        Self::new(64 * 1000)
    }
    pub fn is_rom_loaded(&self) -> bool {
        !self.rom.is_empty()
    }
    /// Creates everything to be able to render
    pub fn resume_renderer(&mut self, surface_texture: pixels::SurfaceTexture<Arc<Window>>) {
        self.renderer.resume(surface_texture);
    }
    pub fn render(&mut self) {
        self.renderer.render();
    }
    pub fn resize_renderer(&mut self, size: PhysicalSize<u32>) {
        self.renderer.resize_surface(size.width, size.height);
    }
    pub fn step(&mut self) {
        // incrementing the cpu counter (it is just for user purposes, so it doesn't need to exist outside ram)
        self.sram[MemoryPois::CPUCycleCounter as usize] =
            self.sram[MemoryPois::CPUCycleCounter as usize].wrapping_add(1);

        self.sram[MemoryPois::ProgramCounter as usize] = self.program_counter;

        // we just skip the current cpu cycle if we are still stalling
        if self.stall_timer > 0 {
            self.stall_timer -= 1;
            return;
        }

        let instruction = self.parse_next_instruction();

        let mut jumped = false;

        match instruction.opcode {
            OpCode::NoOp => {}
            OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div => {
                let val1 = self.read_arg(&instruction.arg1);
                let val2 = self.read_arg(&instruction.arg2);

                let result = match instruction.opcode {
                    OpCode::Add => val1.wrapping_add(val2),
                    OpCode::Sub => val1.wrapping_sub(val2),
                    OpCode::Mul => val1.wrapping_mul(val2),
                    OpCode::Div => val1.wrapping_div(val2),
                    _ => unreachable!(),
                };

                self.write_arg(&instruction.arg1, result);
            }
            OpCode::Stall => self.stall_timer = self.read_arg(&instruction.arg1),
            OpCode::Exit => {
                self.exit_code = self.read_arg(&instruction.arg1);
                return; // TODO: implement exit
            }
            // TODO: add longjumps
            // we have to jump to the address - 1 because the program_counter is incremented after this
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
        }
        if !jumped {
            self.program_counter += 1;
        }
        // check if we reached the end of the program
        if self.program_counter as usize >= self.rom.len() / 3 {
            self.program_counter = 0;
            // TODO: implement exit
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
    pub fn read_ram(&self, address: u16) -> u16 {
        self.sram[address as usize]
    }
    #[inline(always)]
    pub fn write_ram(&mut self, address: u16, value: u16) {
        if address >= 300 {
            self.sram[address as usize] = value;
        }
    }
    fn parse_next_instruction(&mut self) -> Instruction {
        Instruction::from_u16(
            &self
                .rom
                .get((self.program_counter as usize * 3)..(self.program_counter as usize * 3) + 3)
                .unwrap_or_else(|| {
                    panic!(
                        "Program Pointer went out of bounds; ROM len: {}, pointer: {}",
                        self.rom.len(),
                        self.program_counter,
                    );
                })
                .try_into()
                .unwrap(),
        )
        .unwrap()
    }
    pub fn load_rom_from_path(&mut self, path: &str) -> anyhow::Result<()> {
        let watch = Stopwatch::start_new();

        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let file_size_bytes = metadata.len();
        let u16_capacity = (file_size_bytes / 2 - 10) as usize; // Ignoring the Magic and Version
        let mut reader = BufReader::new(file);
        self.rom = Vec::with_capacity(u16_capacity);

        // The secret is ignored when assembling so it is also ignored when interpreting
        reader.seek(SeekFrom::Start(17))?;

        let mut version_bytes: [u8; 3] = [0; 3];
        reader.read_exact(&mut version_bytes)?;
        compare_version(version_bytes);

        for _ in 0..u16_capacity {
            self.rom.push(match reader.read_u16::<LittleEndian>() {
                Ok(value) => value,
                Err(e) => {
                    panic!("{}", e)
                }
            });
        }
        // initializing read only flags
        self.sram[MemoryPois::MajorConsoleVersion as usize] =
            env!("CARGO_PKG_VERSION_MAJOR").parse::<u16>().unwrap();
        self.sram[MemoryPois::MinorConsoleVersion as usize] =
            env!("CARGO_PKG_VERSION_MINOR").parse::<u16>().unwrap();
        self.sram[MemoryPois::PatchConsoleVersion as usize] =
            env!("CARGO_PKG_VERSION_PATCH").parse::<u16>().unwrap();
        self.sram[MemoryPois::MajorROMVersion as usize] = version_bytes[0] as u16;
        self.sram[MemoryPois::MinorROMVersion as usize] = version_bytes[1] as u16;
        self.sram[MemoryPois::PatchROMVersion as usize] = version_bytes[2] as u16;

        println!("Loaded ROM in: {}s", watch.s());
        Ok(())
    }
}

/// Compares version bytes with the version of the console
fn compare_version(version_bytes: [u8; 3]) {
    let major_version = env!("CARGO_PKG_VERSION_MAJOR").parse::<u8>().unwrap();
    let minor_version = env!("CARGO_PKG_VERSION_MINOR").parse::<u8>().unwrap();
    let patch_version = env!("CARGO_PKG_VERSION_PATCH").parse::<u8>().unwrap();

    if major_version != version_bytes[0] || minor_version != version_bytes[1] {
        panic!(
            "Rom is not compatible (console_ver: {}, bin_ver: {:?})",
            env!("CARGO_PKG_VERSION"),
            version_bytes
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
