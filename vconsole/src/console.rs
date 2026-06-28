use byteorder::{LittleEndian, ReadBytesExt};
use simple_stopwatch::Stopwatch;

use std::{
    fs::File,
    io::{BufReader, Read},
    io::{Seek, SeekFrom},
};

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
    fn from_u8(bytecode: u8) -> anyhow::Result<Self> {
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
            other => {
                anyhow::bail!("Unknown OpCode: {other}")
            }
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
    fn from_u8(bytecode: u8) -> anyhow::Result<Self> {
        match bytecode {
            0 => Ok(ArgumentType::Empty),
            1 => Ok(ArgumentType::Immediate),
            2 => Ok(ArgumentType::Address),
            3 => Ok(ArgumentType::Register),
            4 => Ok(ArgumentType::RegisterPointedAddress),
            other => {
                anyhow::bail!("Unknown ArgumentType: {} (0 - 5)", other)
            }
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
    fn from_u16(source: &[u16; 3]) -> anyhow::Result<Self> {
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

enum VersionMatchType {
    Full,         // The Version Number matches down to the patch
    Partial,      // The Version Number matches but not the patch
    Incompatible, // The Mayor or Minor Version doesn't match
}

pub struct Console {
    rom: Vec<u16>,
    sram: Vec<u16>,
    program_ptr: u32,
    stall_timer: u16,
    exit_code: u16,
    registers: [u16; 4],
}

impl Console {
    pub fn new(sram_size: usize) -> Self {
        Self {
            rom: Vec::new(),
            sram: vec![0; sram_size],
            program_ptr: 0,
            stall_timer: 0,
            exit_code: 0,
            registers: [0; 4],
        }
    }
    pub fn default() -> Self {
        Self::new(64 * 1000)
    }
    pub fn run(&mut self) {
        loop {
            // we just skip the current cpu cycle, not clean but works :D
            if self.stall_timer > 0 {
                self.stall_timer -= 1;
                continue;
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
                    break;
                }
                // TODO: add longjumps
                // we have to jump to the address - 1 because the program_pointer is incremented after this
                OpCode::Jmp => {
                    self.program_ptr = self.read_arg(&instruction.arg1) as u32;
                    jumped = true;
                }

                OpCode::Mov => {
                    let val2 = self.read_arg(&instruction.arg2);
                    self.write_arg(&instruction.arg1, val2);
                }
                OpCode::Jeq => {
                    if self.read_arg(&instruction.arg2) == 0 {
                        self.program_ptr = self.read_arg(&instruction.arg1) as u32;
                        jumped = true;
                    }
                }
                OpCode::Jne => {
                    if self.read_arg(&instruction.arg2) != 0 {
                        self.program_ptr = self.read_arg(&instruction.arg1) as u32;
                        jumped = true;
                    }
                }
            }
            if !jumped {
                self.program_ptr += 1;
            }
            // check if we reached the end of the program
            if self.program_ptr as usize >= self.rom.len() / 3 {
                self.exit_code = 0;
                break;
            }
        }
        println!("Program quit with exit code: {}", self.exit_code)
    }
    fn read_arg(&mut self, arg: &Argument) -> u16 {
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
    pub fn read_ram(&mut self, address: u16) -> u16 {
        self.sram.get(address as usize).cloned().unwrap_or(0)
    }
    pub fn write_ram(&mut self, address: u16, value: u16) {
        self.sram[address as usize] = value
    }
    pub fn parse_next_instruction(&mut self) -> Instruction {
        Instruction::from_u16(
            &self
                .rom
                .get((self.program_ptr as usize * 3)..(self.program_ptr as usize * 3) + 3)
                .expect("Out of bounds Instruction read")
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
        match compare_version(version_bytes) {
            VersionMatchType::Full => {}
            VersionMatchType::Partial => {
                println!(
                    "Rom is only partially compatible (console_ver: {}, bin_ver: {:?})",
                    env!("CARGO_PKG_VERSION"),
                    version_bytes
                )
            }
            VersionMatchType::Incompatible => {
                panic!(
                    "Rom is not compatible (console_ver: {}, bin_ver: {:?})",
                    env!("CARGO_PKG_VERSION"),
                    version_bytes
                )
            }
        }

        for _ in 0..u16_capacity {
            self.rom.push(match reader.read_u16::<LittleEndian>() {
                Ok(value) => value,
                Err(e) => {
                    panic!("{}", e)
                }
            });
        }

        println!("Loaded ROM in: {}s", watch.s());
        Ok(())
    }
}

/// Compares version bytes with the version of the console
fn compare_version(version_bytes: [u8; 3]) -> VersionMatchType {
    let mayor_version = env!("CARGO_PKG_VERSION_MAJOR").parse::<u8>().unwrap();
    let minor_version = env!("CARGO_PKG_VERSION_MINOR").parse::<u8>().unwrap();
    let patch_version = env!("CARGO_PKG_VERSION_PATCH").parse::<u8>().unwrap();

    if mayor_version != version_bytes[0] || minor_version != version_bytes[1] {
        return VersionMatchType::Incompatible;
    }
    if patch_version != version_bytes[2] {
        return VersionMatchType::Partial;
    }
    VersionMatchType::Full
}
