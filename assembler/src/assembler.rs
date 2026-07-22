use std::collections::HashMap;

use miette::{NamedSource, SourceSpan};

use crate::assemblererror::AssemblerError;

#[repr(u8)]
#[non_exhaustive]
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
    fn from_str(string: &str, file_byte_index: usize) -> Result<Self, AssemblerError> {
        match string {
            "noop" => Ok(OpCode::NoOp),
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
                span: SourceSpan::new(file_byte_index.into(), other.len()),
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
        if value.is_empty() && !identifier.starts_with(|c: char| c.is_alphabetic()) {
            return Err(AssemblerError::ArgumentMissingValue {
                span: SourceSpan::new((file_byte_index + 1).into(), 1),
            });
        }

        let arg_type = match identifier {
            "#" => ArgumentType::Immediate(parse_int::parse::<u16>(value).map_err(|e| {
                AssemblerError::FailedToParseNumber {
                    number: value.to_string(),
                    parseerror: e,
                    span: SourceSpan::new((file_byte_index + 2).into(), value.len()),
                }
            })?),
            "$" => {
                let (address_type, value) = value.split_at(1);
                match address_type {
                    "#" => ArgumentType::Address(parse_int::parse::<u16>(value).map_err(|e| {
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
                            span: SourceSpan::new(file_byte_index.into(), other.len()),
                        });
                    }
                }
            }
            "*" => ArgumentType::Register(Register::parse_register(
                value,
                file_byte_index + identifier.len(),
            )?),
            other => {
                if identifier.starts_with(|c: char| c.is_alphabetic()) {
                    ArgumentType::Label(string.to_string())
                } else {
                    return Err(AssemblerError::UnknownArgumentPrefix {
                        argument_prefix: other.split_at(1).0.to_string(),
                        span: SourceSpan::new(file_byte_index.into(), other.len()),
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
                        span: SourceSpan::new(self.file_byte_index.into(), self.len_bytes),
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
    #[allow(unused)] // checking for immediates is rarely useful, but for completeness ;)
    pub fn is_immediate(&self) -> bool {
        matches!(self.arg_type, ArgumentType::Immediate(_))
    }
    pub fn is_immediate_address(&self) -> bool {
        matches!(self.arg_type, ArgumentType::Address(_))
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
    pub fn assemble(&mut self) -> Result<Vec<u16>, String> {
        let mut assembly: Vec<u16> = Vec::new();

        self.assemble_prepass();

        for instruction in self.source.lines() {
            let trimmed = match instruction.trim().split(';').next() {
                None | Some("") => continue,
                Some(trimmed) => trimmed,
            };

            match self.assemble_instruction(trimmed) {
                None => continue,
                Some(bytecode) => {
                    assembly.extend_from_slice(&bytecode);
                }
            }
        }

        if !self.errors.is_empty() {
            let error_count = self.errors.len();
            for error in self.errors.drain(..) {
                println!(
                    "{:?}",
                    miette::Report::new(error).with_source_code(NamedSource::new(
                        "src/test.v16.asm",
                        self.source.to_string()
                    ))
                );
            }

            return Err(format!(
                "Could not assemble _ due to {error_count} error(s)"
            ));
        }

        Ok(assembly)
    }
    /// This is where we search for all the labels and store their locations
    fn assemble_prepass(&mut self) {
        let mut label_strs: Vec<&str> = Vec::new();
        let source_start_ptr = self.source.as_ptr() as usize;
        let mut instruction_index = 0;

        for instruction in self.source.lines() {
            let instruction = match instruction.split(';').next() {
                None => {
                    continue;
                }
                Some(instruction) => {
                    let trimmed = instruction.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    trimmed
                }
            };
            if instruction.ends_with(':') {
                let trimmed_instruction = instruction.replace(':', "");
                match self.labels.insert(trimmed_instruction, instruction_index) {
                    None => {
                        label_strs.push(instruction);
                    }
                    Some(_) => {
                        println!("{}, {:?}, {:?}", instruction, label_strs, self.labels);
                        self.errors.push(AssemblerError::LabelDefinedMultipleTimes {
                            span: SourceSpan::new(
                                ((label_strs
                                    .iter()
                                    .find(|s| s.contains(instruction))
                                    .unwrap()
                                    .as_ptr() as usize)
                                    - source_start_ptr)
                                    .into(),
                                instruction.len(),
                            ),
                            span1: SourceSpan::new(
                                ((instruction.as_ptr() as usize) - source_start_ptr).into(),
                                instruction.len(),
                            ),
                        });
                    }
                }
            } else {
                instruction_index += 1;
            }
        }
    }
    /// Actually creating the output assembly
    fn assemble_instruction(&mut self, instruction: &str) -> Option<[u16; 3]> {
        let source_start_ptr = self.source.as_ptr() as usize;

        if instruction.contains(':') {
            if instruction.ends_with(':') {
                return None; // We early return when it is a flag
            } else {
                let label_str = instruction.split_whitespace().next().unwrap();
                self.errors.push(AssemblerError::ExpectedNewline {
                    span: SourceSpan::new(
                        (label_str.as_ptr() as usize - source_start_ptr + label_str.len() + 1)
                            .into(),
                        instruction.len() - label_str.len() - 1,
                    ),
                });
                return None;
            }
        }
        let mut tokens = instruction.split_whitespace();

        let opcode_token = tokens.next().unwrap();
        let opcode = OpCode::from_str(
            opcode_token.to_lowercase().as_str(),
            opcode_token.as_ptr() as usize - source_start_ptr,
        )
        .unwrap_or_else(|e| {
            self.errors.push(e);
            OpCode::NoOp
        });

        if instruction.split_whitespace().collect::<Vec<&str>>().len() > 3 {
            self.errors.push(AssemblerError::TooManyArguments {
                span: SourceSpan::new((opcode_token.as_ptr() as usize).into(), instruction.len()),
            });
        }

        // this looks very messy but its just turning faliure states into default values so that we can
        // have multiple errors in one line without it falling appart down the road
        let arg1 = match tokens.next() {
            Some(token) => {
                match Argument::parse_argument(token, token.as_ptr() as usize - source_start_ptr) {
                    Ok(arg) => arg,
                    Err(e) => {
                        self.errors.push(e);
                        return Some([0; 3]); // we return NoOp
                    }
                }
            }
            None => Argument::empty(
                instruction.as_ptr() as usize - source_start_ptr + instruction.len(),
            ),
        };
        let arg2 = match tokens.next() {
            Some(token) => {
                match Argument::parse_argument(token, token.as_ptr() as usize - source_start_ptr) {
                    Ok(arg) => arg,
                    Err(e) => {
                        self.errors.push(e);
                        return Some([0; 3]); // we return NoOp
                    }
                }
            }
            None => Argument::empty(
                instruction.as_ptr() as usize - source_start_ptr + instruction.len(),
            ),
        };

        Some(match opcode {
            OpCode::NoOp => {
                self.convert_to_bytecode(opcode, arg1, arg2, Some(|_| true), Some(|_| true))
            }
            OpCode::Add => self.convert_to_bytecode(
                opcode,
                arg1,
                arg2,
                Some(|arg| arg.is_writable()),
                Some(|arg| arg.is_readable()),
            ),
            OpCode::Sub => self.convert_to_bytecode(
                opcode,
                arg1,
                arg2,
                Some(|arg| arg.is_writable()),
                Some(|arg| arg.is_readable()),
            ),
            OpCode::Mul => self.convert_to_bytecode(
                opcode,
                arg1,
                arg2,
                Some(|arg| arg.is_writable()),
                Some(|arg| arg.is_readable()),
            ),
            OpCode::Div => self.convert_to_bytecode(
                opcode,
                arg1,
                arg2,
                Some(|arg| arg.is_writable()),
                Some(|arg| arg.is_readable()),
            ),
            OpCode::Exit => {
                self.convert_to_bytecode(opcode, arg1, arg2, Some(|arg| arg.is_readable()), None)
            }
            OpCode::Stall => {
                self.convert_to_bytecode(opcode, arg1, arg2, Some(|arg| arg.is_readable()), None)
            }
            OpCode::Mov => self.convert_to_bytecode(
                opcode,
                arg1,
                arg2,
                Some(|arg| arg.is_writable()),
                Some(|arg| arg.is_readable()),
            ),
            OpCode::Jmp => {
                self.convert_to_bytecode(opcode, arg1, arg2, Some(|arg| arg.is_label()), None)
            }
            OpCode::Jeq => self.convert_to_bytecode(
                opcode,
                arg1,
                arg2,
                Some(|arg| arg.is_label()),
                Some(|arg| arg.is_readable()),
            ),
            OpCode::Jne => self.convert_to_bytecode(
                opcode,
                arg1,
                arg2,
                Some(|arg| arg.is_label()),
                Some(|arg| arg.is_readable()),
            ),
        })
    }
    fn convert_to_bytecode(
        &mut self,
        opcode: OpCode,
        arg1: Argument,
        arg2: Argument,
        first_filter: Option<fn(&Argument) -> bool>,
        second_filter: Option<fn(&Argument) -> bool>,
    ) -> [u16; 3] {
        if let Some(is_valid) = first_filter
            && !is_valid(&arg1)
        {
            if !arg1.is_empty() {
                self.errors.push(AssemblerError::InvalidArgumentType {
                    arg: arg1.arg_type.clone(),
                    span: SourceSpan::new(arg1.file_byte_index.into(), arg1.len_bytes),
                });
            } else {
                self.errors.push(AssemblerError::MissingRequiredArgument {
                    argument_number: 1,
                    span: SourceSpan::new(arg1.file_byte_index.into(), arg1.len_bytes),
                });
            }
        }

        if let Some(is_valid) = second_filter
            && !is_valid(&arg2)
        {
            if !arg2.is_empty() {
                self.errors.push(AssemblerError::InvalidArgumentType {
                    arg: arg2.arg_type.clone(),
                    span: SourceSpan::new(arg2.file_byte_index.into(), arg2.len_bytes),
                });
            } else {
                self.errors.push(AssemblerError::MissingRequiredArgument {
                    argument_number: 2,
                    span: SourceSpan::new(arg2.file_byte_index.into(), arg2.len_bytes),
                });
            }
        }

        let arg1_type_bc = arg1.as_bytecode();
        let arg2_type_bc = arg2.as_bytecode();

        // very readable :D
        [
            ((opcode as u16) << 8) | (arg1_type_bc << 4) | arg2_type_bc,
            arg1.value_as_bytecode(&self.labels)
                .unwrap_or_else(|e| -> u16 {
                    self.errors.push(e);
                    0
                }),
            arg2.value_as_bytecode(&self.labels)
                .unwrap_or_else(|e| -> u16 {
                    self.errors.push(e);
                    0
                }),
        ]
    }
}

#[cfg(test)]
mod tests;
