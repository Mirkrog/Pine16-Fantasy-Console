use byteorder::{BigEndian, ReadBytesExt};

use std::{fs::File, io::BufReader};

#[repr(u8)]
#[non_exhaustive]
enum OpCode {
    Add,
    Sub,
    Mul,
    Div,
    Exit,
    Stall,
    Mov,
    Jmp,
}
#[derive(PartialEq, Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Register {
    A,
    B,
    C,
    D,
}
#[repr(u16)]
#[derive(PartialEq, Debug, Clone)]
#[non_exhaustive]
pub enum Argument {
    Empty,
    DirectValue(u16),                 // Contains a Value
    Address(u16),                     // uses a direct value to point to the memory location
    RegisterPointedAddress(Register), // uses the value of the register to point to the memory location
    Register(Register),               // Points to one of the registers
    Label(String), // Used as the target for jump instructions, can also be used as a way of handling rom addresses
}

struct Instruction {
    opcode: OpCode,
    arg0: Argument,
    arg1: Argument,
}
impl Instruction {
    fn from_u16(source: &[u16; 3]) -> Self {
        let (opcode_byte, args_byte) = ((source[0] >> 8) as u8, (source[0] & 0xff) as u8);
        let (arg0_byte, arg1_byte) = ((args_byte >> 4) as u8, (args_byte & 0b00001111) as u8);

        Self {
            opcode: opcode_byte,
        }
    }
}

pub struct Interpreter {
    rom: Vec<u16>,
    sram: Vec<u16>,
    sram_size: usize,
    stack_size: usize,
    stack_ptr: usize,
    program_ptr: usize,
}

impl Interpreter {
    pub fn new(sram_size: usize, stack_size: usize) -> Self {
        Self {
            rom: Vec::new(),
            sram: Vec::with_capacity(sram_size),
            sram_size,
            stack_size,
            stack_ptr: 0,
            program_ptr: 0,
        }
    }
    pub fn default() -> Self {
        Self::new(128 * 1000, 100)
    }
    pub fn run(&mut self) {
        loop {
            if self.interpret_next_instruction() {
                break;
            }
        }
    }
    fn interpret_next_instruction(&mut self) -> bool {}
    pub fn load_rom_from_path(&mut self, path: &str) -> anyhow::Result<()> {
        self.rom = read_file_as_u16_vec(path)?;

        Ok(())
    }
}

fn read_file_as_u16_vec(file_path: &str) -> anyhow::Result<Vec<u16>> {
    let file = File::open(file_path)?;
    let metadata = file.metadata()?;
    let file_size_bytes = metadata.len();
    let u16_capacity = (file_size_bytes / 2) as usize;
    let mut u16_buffer = Vec::with_capacity(u16_capacity);
    let mut reader = BufReader::new(file);
    loop {
        match reader.read_u16::<BigEndian>() {
            Ok(value) => u16_buffer.push(value),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    break;
                }
                return Err(e.into());
            }
        }
    }
    Ok(u16_buffer)
}
