use byteorder::{LittleEndian, WriteBytesExt};
use colored::Colorize;
use miette::{NamedSource, SourceSpan};
use simple_stopwatch::Stopwatch;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::vec;
use std::{fs::File, io::Write};

use crate::assemblererror::AssemblerError;

#[repr(u8)]
#[non_exhaustive]
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
    FDump,
    RDump,
}
impl OpCode {
    fn from_str(string: &str, file_byte_index: usize) -> Result<Self, AssemblerError> {
        match string {
            "noop" => Ok(OpCode::NoOp),
            "add" => Ok(OpCode::Add),
            "sub" => Ok(OpCode::Sub),
            "mul" => Ok(OpCode::Mul),
            "div" => Ok(OpCode::Div),
            "stall" => Ok(OpCode::Stall),
            "await" => Ok(OpCode::Await),
            "mov" => Ok(OpCode::Mov),
            "jmp" => Ok(OpCode::Jmp),
            "jeq" => Ok(OpCode::Jeq),
            "jne" => Ok(OpCode::Jne),
            "and" => Ok(OpCode::And),
            "or" => Ok(OpCode::Or),
            "xor" => Ok(OpCode::Xor),
            "shl" => Ok(OpCode::Shl),
            "shr" => Ok(OpCode::Shr),
            "push" => Ok(OpCode::Push),
            "pop" => Ok(OpCode::Pop),
            "jsr" => Ok(OpCode::Jsr),
            "rtr" => Ok(OpCode::Rtr),
            "fdump" => Ok(OpCode::FDump),
            "rdump" => Ok(OpCode::RDump),
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
    SP,
}
impl Register {
    fn parse_register(string: &str, file_byte_index: usize) -> Result<Self, AssemblerError> {
        match string {
            "A" => Ok(Self::A),
            "B" => Ok(Self::B),
            "C" => Ok(Self::C),
            "D" => Ok(Self::D),
            "SP" => Ok(Self::SP),
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
    Broken, // Argument used only by the assembler, to signal a argument that failed to parse, prevents unintended errors later in the process
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
            ArgumentType::Broken => {
                unreachable!()
            }
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
            ArgumentType::Broken => {
                unreachable!()
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
    #[allow(unused)] // checking labels is useless because labels are just a type of immediate
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
    source_name: &'a str,
    labels: HashMap<String, u16>,
    data: BTreeMap<u16, u16>,
    errors: Vec<AssemblerError>,
}
impl<'a> Assembler<'a> {
    pub fn new(source: &'a str, source_name: &'a str) -> Self {
        Self {
            source,
            source_name,
            labels: HashMap::new(),
            data: BTreeMap::new(),
            errors: Vec::new(),
        }
    }
    pub fn assemble(&mut self) -> Result<Vec<u16>, String> {
        let mut assembly = self.assemble_prepass();

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
                "Could not assemble {} due to {error_count} error(s)",
                self.source_name
            ));
        }

        Ok(assembly)
    }
    /// This is where we search for all the labels and store their locations and parse data sections
    fn assemble_prepass(&mut self) -> Vec<u16> {
        let mut label_strs: Vec<&str> = Vec::new();
        let source_start_ptr = self.source.as_ptr() as usize;
        let mut instruction_index = 0;
        let mut data_index = 0;

        let mut data_buffer: BTreeMap<u16, u16> = BTreeMap::new(); // we will write the data to a map to duplicate data writes

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
            } else if instruction.starts_with('.') {
                let mut tokens = instruction
                    .split_whitespace()
                    .flat_map(|token| token.split(','))
                    .filter(|token| !token.is_empty())
                    .peekable();
                let type_token = tokens.next().unwrap_or_default();

                let mut lonely_number: Option<&str> = None;
                let mut data_repeats = 1;
                for i in 0..2 {
                    match tokens.peek().copied() {
                        None => {}
                        Some("at") => {
                            if let Some(number) = lonely_number {
                                self.errors.push(AssemblerError::NumberInfrontOperation {
                                    span: SourceSpan::new(
                                        (number.as_ptr() as usize - source_start_ptr).into(),
                                        number.len(),
                                    ),
                                });
                            }
                            tokens.next();
                            let token = tokens.next().unwrap_or_default();
                            data_index =
                                parse_int::parse::<u16>(token).unwrap_or_else(|e| -> u16 {
                                    self.errors.push(AssemblerError::FailedToParseNumber {
                                        number: token.to_string(),
                                        parseerror: e,
                                        span: SourceSpan::new(
                                            (token.as_ptr() as usize - source_start_ptr).into(),
                                            token.len(),
                                        ),
                                    });
                                    0
                                });
                        }
                        Some("repeat") => {
                            if let Some(number) = lonely_number {
                                self.errors.push(AssemblerError::NumberInfrontOperation {
                                    span: SourceSpan::new(
                                        (number.as_ptr() as usize - source_start_ptr).into(),
                                        number.len(),
                                    ),
                                });
                            }
                            tokens.next();
                            let token = tokens.next().unwrap_or_default();
                            data_repeats =
                                parse_int::parse::<u16>(token).unwrap_or_else(|e| -> u16 {
                                    self.errors.push(AssemblerError::FailedToParseNumber {
                                        number: token.to_string(),
                                        parseerror: e,
                                        span: SourceSpan::new(
                                            (token.as_ptr() as usize - source_start_ptr).into(),
                                            token.len(),
                                        ),
                                    });
                                    0
                                })
                        }
                        Some(other) => {
                            if parse_int::parse::<f64>(other).is_ok() {
                                lonely_number = Some(tokens.next().unwrap());
                            } else {
                                self.errors.push(AssemblerError::UnknownDataOperator {
                                    span: SourceSpan::new(
                                        (other.as_ptr() as usize - source_start_ptr).into(),
                                        other.len(),
                                    ),
                                });
                                tokens.next();
                            }
                        }
                    };
                }

                let data = match type_token {
                    ".dw" => {
                        let token = tokens.next().unwrap_or_else(|| {
                            self.errors.push(AssemblerError::WrongArgumentAmount {
                                expected_amount: 1,
                                amount: 0,
                                span: SourceSpan::new(
                                    ((type_token.as_ptr() as usize + type_token.len())
                                        - source_start_ptr)
                                        .into(),
                                    1,
                                ),
                            });
                            "1"
                        });
                        let value = parse_int::parse::<u16>(token).unwrap_or_else(|e| -> u16 {
                            self.errors.push(AssemblerError::FailedToParseNumber {
                                number: token.to_string(),
                                parseerror: e,
                                span: SourceSpan::new(
                                    (token.as_ptr() as usize - source_start_ptr).into(),
                                    token.len(),
                                ),
                            });
                            1
                        });
                        if value == 0 {
                            self.errors.push(AssemblerError::UselessDataDefinition {
                                span: SourceSpan::new(
                                    (token.as_ptr() as usize - source_start_ptr).into(),
                                    token.len(),
                                ),
                            });
                        }
                        vec![value]
                    }
                    ".dsprrow" => {
                        let amount = tokens.clone().count();
                        if amount != 8 {
                            let first_token = tokens.next();
                            self.errors.push(AssemblerError::WrongArgumentAmount {
                                expected_amount: 8,
                                amount,
                                span: SourceSpan::new(
                                    first_token
                                        .map_or_else(
                                            || {
                                                type_token.as_ptr() as usize + type_token.len() + 1
                                                    - source_start_ptr
                                            },
                                            |token| (token.as_ptr() as usize) - source_start_ptr,
                                        )
                                        .into(),
                                    tokens.next_back().map_or_else(
                                        || type_token.as_ptr() as usize + type_token.len() + 1,
                                        |token| {
                                            (token.as_ptr() as usize + token.len())
                                                - (first_token.unwrap().as_ptr() as usize)
                                        },
                                    ),
                                ),
                            });
                        }
                        let mut output = vec![0u16; 2];
                        for (i, token) in tokens.enumerate() {
                            let value = parse_int::parse::<u16>(token).unwrap_or_else(|e| -> u16 {
                                self.errors.push(AssemblerError::FailedToParseNumber {
                                    number: token.to_string(),
                                    parseerror: e,
                                    span: SourceSpan::new(
                                        (token.as_ptr() as usize - source_start_ptr).into(),
                                        token.len(),
                                    ),
                                });
                                1
                            });
                            if value > 15 {
                                self.errors.push(AssemblerError::PaletteIndexOutOfBounds {
                                    number: value,
                                    span: SourceSpan::new(
                                        (token.as_ptr() as usize - source_start_ptr).into(),
                                        token.len(),
                                    ),
                                });
                            }
                            output[i / 4] = value << ((3 - (i % 4)) * 4);
                        }
                        output
                    }
                    other => {
                        println!("{}", other);
                        todo!()
                    }
                };
                if (data_index as usize * data.len()) + data_repeats as usize > u16::MAX as usize {
                    self.errors
                        .push(AssemblerError::DataInitializedBeyondRAMBounds {
                            address: (data_index as usize * data.len()) + data_repeats as usize,
                            span: SourceSpan::new(
                                (instruction.as_ptr() as usize - source_start_ptr).into(),
                                instruction.len(),
                            ),
                        });
                } else {
                    for _ in 0..data_repeats {
                        for value in data.iter() {
                            data_buffer.insert(data_index, *value);
                            data_index += 1;
                        }
                    }
                }
            } else {
                instruction_index += 1;
            }
        }

        let mut assembly = vec![data_buffer.len() as u16];

        // converting the Map into a Vector and extending the assembly with it
        // btw I kinda like this code, it looks clean :]
        assembly.extend(data_buffer.iter().flat_map(|(k, v)| [*k, *v]));

        assembly
    }
    /// turns the instruction given into bytecode
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
        let mut opcode_obstructed = false;
        let trimmed_instruction = instruction.trim();
        if trimmed_instruction.starts_with(',') {
            self.errors.push(AssemblerError::MisplacedComma {
                span: SourceSpan::new(
                    (trimmed_instruction.as_ptr() as usize - source_start_ptr).into(),
                    1,
                ),
            });
            opcode_obstructed = true;
        }
        if trimmed_instruction.ends_with(',') {
            self.errors.push(AssemblerError::MisplacedComma {
                span: SourceSpan::new(
                    (trimmed_instruction.as_ptr() as usize - source_start_ptr
                        + trimmed_instruction.len()
                        - 1)
                    .into(),
                    1,
                ),
            });
        }

        if instruction.contains('.') {
            return None; // We early return when it is a definition
        }

        let mut tokens = instruction
            .split_whitespace()
            .flat_map(|token| token.split(','))
            .filter(|token| !token.is_empty());

        let opcode_token = tokens.next().unwrap();
        let opcode = if !opcode_obstructed {
            OpCode::from_str(
                opcode_token.to_lowercase().as_str(),
                opcode_token.as_ptr() as usize - source_start_ptr,
            )
            .unwrap_or_else(|e| {
                self.errors.push(e);
                OpCode::NoOp
            })
        } else {
            OpCode::NoOp
        };

        if tokens.clone().count() > 3 {
            self.errors.push(AssemblerError::TooManyArguments {
                span: SourceSpan::new((opcode_token.as_ptr() as usize).into(), instruction.len()),
            });
        }

        // this looks very messy but its just turning failure states into default values so that we can
        // have multiple errors in one line without it falling appart down the road
        let arg1 = match tokens.next() {
            Some(token) => {
                match Argument::parse_argument(token, token.as_ptr() as usize - source_start_ptr) {
                    Ok(arg) => arg,
                    Err(e) => {
                        self.errors.push(e);
                        Argument {
                            arg_type: ArgumentType::Broken,
                            file_byte_index: 0,
                            len_bytes: 0,
                        }
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
                        Argument {
                            arg_type: ArgumentType::Broken,
                            file_byte_index: 0,
                            len_bytes: 0,
                        }
                    }
                }
            }
            None => Argument::empty(
                instruction.as_ptr() as usize - source_start_ptr + instruction.len(),
            ),
        };
        // we allow both arguments to be parsed when one fails, but abort before they are used to prevent misleading errors
        if matches!(arg1.arg_type, ArgumentType::Broken)
            | matches!(arg2.arg_type, ArgumentType::Broken)
        {
            return Some([0; 3]);
        }

        Some(match opcode {
            OpCode::NoOp => {
                self.convert_to_bytecode(opcode, arg1, arg2, Some(|_| true), Some(|_| true))
            }
            OpCode::Add
            | OpCode::Sub
            | OpCode::Mul
            | OpCode::Div
            | OpCode::Mov
            | OpCode::And
            | OpCode::Or
            | OpCode::Xor
            | OpCode::Shl
            | OpCode::Shr => self.convert_to_bytecode(
                opcode,
                arg1,
                arg2,
                Some(|arg| arg.is_writable()),
                Some(|arg| arg.is_readable()),
            ),
            OpCode::Stall | OpCode::Jmp | OpCode::Await | OpCode::Push | OpCode::Jsr => {
                self.convert_to_bytecode(opcode, arg1, arg2, Some(|arg| arg.is_readable()), None)
            }
            OpCode::Rtr | OpCode::FDump | OpCode::RDump => {
                self.convert_to_bytecode(opcode, arg1, arg2, None, None)
            }
            OpCode::Pop => {
                self.convert_to_bytecode(opcode, arg1, arg2, Some(|arg| arg.is_writable()), None)
            }
            OpCode::Jeq | OpCode::Jne => self.convert_to_bytecode(
                opcode,
                arg1,
                arg2,
                Some(|arg| arg.is_readable()),
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

pub fn run_assembler(source: String, source_name: &str, output: PathBuf) {
    let stopwatch = Stopwatch::start_new();

    let bytecode = match Assembler::new(&source, source_name).assemble() {
        Ok(data) => data,
        Err(err) => {
            eprintln!("{}: {err}", "error".red().bold());
            std::process::exit(1);
        }
    };

    let mut out_file = match File::create(output) {
        Ok(file) => file,
        Err(err) => {
            eprintln!(
                "{}: Failed to create output file: {:?}",
                "error".red().bold(),
                err
            );
            std::process::exit(1);
        }
    };

    let mut buffer = [0u8; 20];
    buffer[..17].copy_from_slice("PINE16ASSEMBLY :D".as_bytes());
    buffer[17] = env!("CARGO_PKG_VERSION_MAJOR").parse::<u8>().unwrap();
    buffer[18] = env!("CARGO_PKG_VERSION_MINOR").parse::<u8>().unwrap();
    buffer[19] = env!("CARGO_PKG_VERSION_PATCH").parse::<u8>().unwrap();

    out_file.write_all(&buffer).unwrap();

    for word in bytecode {
        out_file.write_u16::<LittleEndian>(word).unwrap();
    }

    println!(
        "{} assembling in: {}s",
        "Finished".green().bold(),
        stopwatch.s()
    );
}
