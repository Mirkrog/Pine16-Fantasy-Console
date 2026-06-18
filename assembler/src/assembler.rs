use std::collections::HashMap;

use miette::SourceSpan;

use crate::assemblererror::{AssemblerError, AssemblerErrors};

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
    Jeq,
    Jne,
}
impl OpCode {
    fn from_str(string: &str, file_byte_index: usize) -> Result<Self, AssemblerError> {
        match string {
            "add" => Ok(OpCode::Add),
            "sub" => Ok(OpCode::Sub),
            "mul" => Ok(OpCode::Mul),
            "div" => Ok(OpCode::Div),
            "exit" => Ok(OpCode::Exit),
            "stall" => Ok(OpCode::Stall),
            "mov" => Ok(OpCode::Mov),
            "jmp" => Ok(OpCode::Jmp),
            "jeq" => Ok(OpCode::Jeq),
            "jne" => Ok(OpCode::Jne),
            other => Err(AssemblerError::UnknownOpCode {
                opcode: other.to_string(),
                span: SourceSpan::new(file_byte_index.into(), string.len()),
            }),
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
impl Register {
    fn parse_register(string: &str, file_byte_index: usize) -> Result<Self, AssemblerError> {
        match string {
            "A" => Ok(Self::A),
            "B" => Ok(Self::B),
            "C" => Ok(Self::C),
            "D" => Ok(Self::D),
            other => Err(AssemblerError::UnknownRegister {
                register: other.to_string(),
                span: SourceSpan::new(file_byte_index.into(), 1),
            }),
        }
    }
}
#[derive(PartialEq, Debug, Clone)]
pub struct Argument {
    pub arg_type: ArgumentType,
    pub file_byte_index: usize,
    pub len_bytes: usize,
}
#[repr(u16)]
#[derive(PartialEq, Debug, Clone, Default)]
#[non_exhaustive]
pub enum ArgumentType {
    #[default]
    Empty,
    Immediate(u16),                   // # Value contained in bytecode
    Address(u16),                     // $# Direct value points to memory
    RegisterPointedAddress(Register), // $* Register value points to memory
    Register(Register),               // * Points to a register
    Label(String),                    // Target for jumps / rom addresses
}

impl Argument {
    pub fn empty(file_byte_index: usize) -> Argument {
        Self {
            arg_type: ArgumentType::default(),
            file_byte_index,
            len_bytes: 1,
        }
    }

    pub fn parse_argument(string: &str, file_byte_index: usize) -> Result<Self, AssemblerError> {
        let (identifier, value) = string.split_at(1);

        let arg_type = match identifier {
            "#" => ArgumentType::Immediate(value.parse::<u16>().map_err(|e| {
                AssemblerError::FailedToParseNumber {
                    number: value.to_string(),
                    parseerror: e,
                    span: SourceSpan::new((file_byte_index + 2).into(), value.len()),
                }
            })?),
            "$" => {
                let (address_type, value) = value.split_at(1);
                match address_type {
                    "#" => ArgumentType::Address(value.parse::<u16>().map_err(|e| {
                        AssemblerError::FailedToParseNumber {
                            number: value.to_string(),
                            parseerror: e,
                            span: SourceSpan::new((file_byte_index + 2).into(), value.len()),
                        }
                    })?),
                    "*" => ArgumentType::RegisterPointedAddress(Register::parse_register(
                        value,
                        file_byte_index + 2,
                    )?),
                    other => {
                        return Err(AssemblerError::UnknownAddressType {
                            address: other.to_string(),
                            span: SourceSpan::new((file_byte_index + 1).into(), 1),
                        });
                    }
                }
            }
            "*" => ArgumentType::Register(Register::parse_register(value, file_byte_index + 1)?),
            other => {
                if !other.is_empty() && identifier.starts_with(|c: char| c.is_alphabetic()) {
                    ArgumentType::Label(string.to_string())
                } else {
                    return Err(AssemblerError::UnknownArgumentPrefix {
                        argument_prefix: other.split_at(1).0.to_string(),
                        span: SourceSpan::new(file_byte_index.into(), 1),
                    });
                }
            }
        };

        Ok(Self {
            arg_type,
            file_byte_index,
            len_bytes: string.len(),
        })
    }

    pub fn as_bytecode(&self) -> u16 {
        match &self.arg_type {
            ArgumentType::Empty => 0,
            ArgumentType::Immediate(_) | ArgumentType::Label(_) => 1,
            ArgumentType::Address(_) => 2,
            ArgumentType::Register(_) => 3,
            ArgumentType::RegisterPointedAddress(_) => 4,
        }
    }

    pub fn value_as_bytecode(&self, labels: &HashMap<String, u16>) -> Result<u16, AssemblerError> {
        match &self.arg_type {
            ArgumentType::Empty => Ok(0),
            ArgumentType::Immediate(value) => Ok(*value),
            ArgumentType::Address(value) => Ok(*value),
            ArgumentType::Register(value) => Ok(*value as u16),
            ArgumentType::RegisterPointedAddress(value) => Ok(*value as u16),
            ArgumentType::Label(value) => {
                labels
                    .get(value)
                    .copied()
                    .ok_or(AssemblerError::UnknownLabel {
                        label: value.to_string(),
                        span: SourceSpan::new((self.file_byte_index + 1).into(), self.len_bytes),
                    })
            }
        }
    }
    pub fn is_empty(&self) -> bool {
        matches!(self.arg_type, ArgumentType::Empty)
    }
    pub fn is_readable(&self) -> bool {
        !self.is_empty()
    }
    pub fn is_writable(&self) -> bool {
        self.is_address() || self.is_register()
    }
    pub fn is_immediate_address(&self) -> bool {
        matches!(self.arg_type, ArgumentType::Address(_))
    }
    pub fn is_immediate(&self) -> bool {
        matches!(self.arg_type, ArgumentType::Immediate(_))
    }
    pub fn is_register(&self) -> bool {
        matches!(self.arg_type, ArgumentType::Register(_))
    }
    pub fn is_label(&self) -> bool {
        matches!(self.arg_type, ArgumentType::Label(_))
    }
    pub fn is_register_pointed(&self) -> bool {
        matches!(self.arg_type, ArgumentType::RegisterPointedAddress(_))
    }
    pub fn is_address(&self) -> bool {
        self.is_immediate_address() || self.is_register_pointed()
    }
}

pub struct Assembler<'a> {
    source: &'a str,
    labels: HashMap<String, u16>,
    errors: Vec<AssemblerError>,
}
impl<'a> Assembler<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            labels: HashMap::new(),
            errors: Vec::new(),
        }
    }
    pub fn assemble(&mut self) -> anyhow::Result<Vec<u16>> {
        let mut assembly: Vec<u16> = Vec::new();

        match self.assemble_prepass() {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e)
            }
        }

        for instruction in self.source.lines() {
            let trimmed = match instruction.trim().split(';').next() {
                None | Some("") => continue,
                Some(trimmed) => trimmed,
            };

            match self.assemble_instruction(trimmed) {
                Ok(None) => continue,
                Ok(Some(bytecode)) => {
                    assembly.extend_from_slice(&bytecode);
                }
                Err(e) => {
                    self.errors.push(e);
                }
            }
        }

        if !self.errors.is_empty() {
            let combined_error = AssemblerErrors {
                related: self.errors.drain(..).collect(),
            };

            let report =
                miette::Report::new(combined_error).with_source_code(self.source.to_string());

            anyhow::bail!("{:?}", report);
        }

        Ok(assembly)
    }
    /// This is where we search for all the labels and store their locations
    fn assemble_prepass(&mut self) -> anyhow::Result<()> {
        let mut instruction_index = 0;
        for (index, instruction) in self.source.lines().enumerate() {
            let instruction = match instruction.split(';').next() {
                None => {
                    continue;
                }
                Some(instruction) => {
                    let trimmed = instruction.trim();
                    if trimmed.trim().is_empty() {
                        continue;
                    }
                    trimmed
                }
            };
            if instruction.ends_with(':') {
                match self
                    .labels
                    .insert(instruction.replace(':', ""), instruction_index)
                {
                    None => continue,
                    Some(old) => {
                        anyhow::bail!(
                            "Label '{}' defined twice at lines ({}, {})",
                            instruction.replace(':', ""),
                            old + 1,
                            index + 1
                        )
                    }
                }
            } else {
                instruction_index += 1;
            }
        }
        Ok(())
    }
    /// Actually creating the output assembly
    fn assemble_instruction(
        &mut self,
        instruction: &str,
    ) -> Result<Option<[u16; 3]>, AssemblerError> {
        let source_start_ptr = self.source.as_ptr() as usize;

        if instruction.contains(':') {
            if instruction.ends_with(':') {
                return Ok(None); // We early return when it is a flag
            } else {
                let label_str = instruction.split_whitespace().next().unwrap();
                return Err(AssemblerError::ExpectedNewline {
                    span: SourceSpan::new(
                        (label_str.as_ptr() as usize - source_start_ptr + label_str.len() + 1)
                            .into(),
                        instruction.len() - label_str.len() - 1,
                    ),
                });
            }
        }
        let mut tokens = instruction.split_whitespace();

        let opcode_token = tokens.next().unwrap();
        let opcode = OpCode::from_str(
            opcode_token.to_lowercase().as_str(),
            opcode_token.as_ptr() as usize - source_start_ptr,
        )?;

        if instruction.split_whitespace().collect::<Vec<&str>>().len() > 3 {
            return Err(AssemblerError::TooManyArguments {
                span: SourceSpan::new(
                    (opcode_token.as_ptr() as usize).into(),
                    instruction.len() - opcode_token.as_ptr() as usize - source_start_ptr,
                ),
            });
        }

        let arg1 = if let arg1_token = tokens.next()
            && arg1_token.is_some()
        {
            Argument::parse_argument(
                arg1_token.unwrap(),
                arg1_token.unwrap().as_ptr() as usize - source_start_ptr - 1,
            )?
        } else {
            Argument::empty(instruction.as_ptr() as usize - source_start_ptr + instruction.len())
        };
        let arg2 = if let arg2_token = tokens.next()
            && arg2_token.is_some()
        {
            Argument::parse_argument(
                arg2_token.unwrap(),
                arg2_token.unwrap().as_ptr() as usize - source_start_ptr - 1,
            )?
        } else {
            Argument::empty(instruction.as_ptr() as usize - source_start_ptr + instruction.len())
        };

        Ok(Some(match opcode {
            OpCode::Add => self.convert_to_bytecode(
                opcode,
                arg1,
                arg2,
                Some(|arg| arg.is_writable()),
                Some(|arg| arg.is_readable()),
            )?,
            OpCode::Sub => self.convert_to_bytecode(
                opcode,
                arg1,
                arg2,
                Some(|arg| arg.is_writable()),
                Some(|arg| arg.is_readable()),
            )?,
            OpCode::Mul => self.convert_to_bytecode(
                opcode,
                arg1,
                arg2,
                Some(|arg| arg.is_writable()),
                Some(|arg| arg.is_readable()),
            )?,
            OpCode::Div => self.convert_to_bytecode(
                opcode,
                arg1,
                arg2,
                Some(|arg| arg.is_writable()),
                Some(|arg| arg.is_readable()),
            )?,
            OpCode::Exit => {
                self.convert_to_bytecode(opcode, arg1, arg2, Some(|arg| arg.is_readable()), None)?
            }
            OpCode::Stall => {
                self.convert_to_bytecode(opcode, arg1, arg2, Some(|arg| arg.is_readable()), None)?
            }
            OpCode::Mov => self.convert_to_bytecode(
                opcode,
                arg1,
                arg2,
                Some(|arg| arg.is_writable()),
                Some(|arg| arg.is_readable()),
            )?,
            OpCode::Jmp => {
                self.convert_to_bytecode(opcode, arg1, arg2, Some(|arg| arg.is_label()), None)?
            }
            OpCode::Jeq => self.convert_to_bytecode(
                opcode,
                arg1,
                arg2,
                Some(|arg| arg.is_label()),
                Some(|arg| arg.is_readable()),
            )?,
            OpCode::Jne => self.convert_to_bytecode(
                opcode,
                arg1,
                arg2,
                Some(|arg| arg.is_label()),
                Some(|arg| arg.is_readable()),
            )?,
        }))
    }
    fn convert_to_bytecode(
        &mut self,
        opcode: OpCode,
        arg1: Argument,
        arg2: Argument,
        first_filter: Option<fn(&Argument) -> bool>,
        second_filter: Option<fn(&Argument) -> bool>,
    ) -> Result<[u16; 3], AssemblerError> {
        if let Some(is_valid) = first_filter
            && !is_valid(&arg1)
        {
            if !arg1.is_empty() {
                return Err(AssemblerError::InvalidArgumentType {
                    arg: arg1.arg_type.clone(),
                    span: SourceSpan::new(arg1.file_byte_index.into(), arg1.len_bytes),
                });
            } else {
                return Err(AssemblerError::MissingRequiredArgument {
                    argument_number: 1,
                    span: SourceSpan::new(arg1.file_byte_index.into(), arg1.len_bytes),
                });
            }
        }

        if let Some(is_valid) = second_filter
            && !is_valid(&arg2)
        {
            if !arg2.is_empty() {
                return Err(AssemblerError::InvalidArgumentType {
                    arg: arg2.arg_type.clone(),
                    span: SourceSpan::new(arg2.file_byte_index.into(), arg2.len_bytes),
                });
            } else {
                return Err(AssemblerError::MissingRequiredArgument {
                    argument_number: 2,
                    span: SourceSpan::new(arg2.file_byte_index.into(), arg2.len_bytes),
                });
            }
        }

        let arg1_type_bc = arg1.as_bytecode();
        let arg2_type_bc = arg2.as_bytecode();

        Ok([
            ((opcode as u16) << 8) | (arg1_type_bc << 4) | arg2_type_bc,
            arg1.value_as_bytecode(&self.labels)?,
            arg2.value_as_bytecode(&self.labels)?,
        ])
    }
}

#[cfg(test)]
mod tests;
