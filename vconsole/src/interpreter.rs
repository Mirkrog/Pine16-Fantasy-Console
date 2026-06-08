use byteorder::{BigEndian, ReadBytesExt};

use std::{any, fs::File, io::BufReader, sync::Arc};

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
impl OpCode {
    fn from_u8(bytecode: u8) -> anyhow::Result<Self> {
        match bytecode {
            0 => Ok(OpCode::Add),
            1 => Ok(OpCode::Sub),
            2 => Ok(OpCode::Mul),
            3 => Ok(OpCode::Div),
            4 => Ok(OpCode::Exit),
            5 => Ok(OpCode::Stall),
            6 => Ok(OpCode::Mov),
            7 => Ok(OpCode::Jmp),
            other => {
                anyhow::bail!("Unknown OpCode: {}  (0 - 7)", other)
            }
        }
    }
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
pub enum ArgumentType {
    Empty,
    DirectValue,            // Contains a Value
    Address,                // uses a direct value to point to the memory location
    RegisterPointedAddress, // uses the value of the register to point to the memory location
    Register,               // Points to one of the registers
    Label, // Used as the target for jump instructions, can also be used as a way of handling rom addresses
}
impl ArgumentType {
    fn from_u8(bytecode: u8) -> anyhow::Result<Self> {
        match bytecode {
            0 => Ok(ArgumentType::Empty),
            1 => Ok(ArgumentType::DirectValue),
            2 => Ok(ArgumentType::Address),
            3 => Ok(ArgumentType::RegisterPointedAddress),
            4 => Ok(ArgumentType::Register),
            5 => Ok(ArgumentType::Label),
            other => {
                anyhow::bail!("Unknown ArgumentType: {} (0 - 5)", other)
            }
        }
    }
}

struct Instruction {
    opcode: OpCode,
    arg1_type: ArgumentType,
    arg2_type: ArgumentType,
    arg1: u16,
    arg2: u16,
}
impl Instruction {
    fn from_u16(source: &[u16; 3]) -> anyhow::Result<Self> {
        // they are all bytes but arg0 and arg1 are only u4
        let (opcode_byte, args_byte) = ((source[0] >> 8) as u8, (source[0] & 0xff) as u8);
        let (arg1_type_byte, arg2_type_byte) = ((args_byte >> 4), args_byte & 0b00001111);

        Ok(Self {
            opcode: OpCode::from_u8(opcode_byte)?,
            arg1_type: ArgumentType::from_u8(arg1_type_byte)?,
            arg2_type: ArgumentType::from_u8(arg2_type_byte)?,
            arg1: source[1],
            arg2: source[2],
        })
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
    fn interpret_next_instruction(&mut self) -> bool {
        let instruction = Instruction::from_u16(
            &self.rom[self.program_ptr..self.program_ptr + 3]
                .try_into()
                .unwrap(),
        )
        .unwrap();

        false
    }
    pub fn load_rom_from_path(&mut self, path: &str) -> anyhow::Result<()> {
        self.rom = read_file_as_u16_vec(path)?;

        Ok(())
    }
    fn read_argument(arg_type: ArgumentType, pointer: u16) {
        unimplemented!()
    }
    fn write_argument(arg_type: ArgumentType, pointer: u16) {
        unimplemented!()
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
