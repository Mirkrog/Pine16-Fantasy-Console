use std::{
    any,
    collections::HashMap,
    iter::{Enumerate, Peekable},
};

use anyhow::Context;

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
impl Register {
    fn parse_register(string: &str) -> anyhow::Result<Self> {
        match string {
            "A" => Ok(Self::A),
            "B" => Ok(Self::B),
            "C" => Ok(Self::C),
            "D" => Ok(Self::D),
            other => anyhow::bail!("Unknown Register: {}", other),
        }
    }
}
#[repr(u16)]
#[derive(PartialEq, Debug, Clone)]
#[non_exhaustive]
pub enum Argument {
    Empty,
    DirectValue(u16),   // Contains a Value
    Address(u16),       // Points to a value
    Register(Register), // Points to one of the registers
    Label(String),      // Used as the target for jump instructions
}
impl Argument {
    fn parse_argument(string: &str) -> anyhow::Result<Self> {
        let (identifier, value) = string
            .split_at(1);
        match identifier {
            "$" => Ok(Self::DirectValue(value.parse::<u16>().with_context(
                || format!("Failed parsing DirectValue from '{}'", value),
            )?)),
            "@" => Ok(Self::Address(value.parse::<u16>().with_context(|| {
                format!("Failed parsing Address from '{}'", value)
            })?)),
            "*" => Ok(Self::Register(Register::parse_register(value)?)),
            other => {
                if !string.is_empty() && string.chars().all(|c| c.is_alphabetic()) {
                    Ok(Self::Label(string.to_string()))
                } else {
                    anyhow::bail!("Unknown argument type: =>[{}]{}", other, value)
                }
            }
        }
    }
    fn as_bytecode(&self) -> anyhow::Result<u16> {
        match self {
            Self::Empty => Ok(0),
            Self::DirectValue(_) => Ok(1),
            Self::Address(_) => Ok(2),
            Self::Register(_) => Ok(3),
            Self::Label(_) => Ok(4),
        }
    }
    fn value_as_bytecode(&self, labels: &HashMap<String, u16>) -> anyhow::Result<u16> {
        match self {
            Self::Empty => Ok(0),
            Self::DirectValue(value) => Ok(*value),
            Self::Address(address) => Ok(*address),
            Self::Register(register) => Ok(*register as u16),
            Self::Label(label) => Ok(*labels
                .get(label)
                .ok_or(anyhow::anyhow!("Label not found: {}", label))?),
        }
    }
    fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
    fn is_address(&self) -> bool {
        matches!(self, Self::Address(_))
    }
    fn is_direct_value(&self) -> bool {
        matches!(self, Self::DirectValue(_))
    }
    fn is_register(&self) -> bool {
        matches!(self, Self::Register(_))
    }
    fn is_label(&self) -> bool {
        matches!(self, Self::Label(_))
    }
}

pub struct Assembler<'a> {
    source: &'a str,
    labels: HashMap<String, u16>,
}
impl<'a> Assembler<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            labels: HashMap::new(),
        }
    }
    pub fn assemble(&mut self) -> anyhow::Result<Vec<u16>> {
        let mut assembly: Vec<u16> = Vec::new();

        match self.assemble_prepass() {
            Ok(_) => {}
            Err(e) => {
                anyhow::bail!("{}", e)
            }
        }

        for (index, instruction) in self.source.lines().enumerate() {
            let instruction = match instruction.trim().split(';').next() {
                None | Some("") => {
                    continue;
                }
                Some(instruction) => instruction,
            };
            match self.assemble_instruction(instruction) {
                Ok(None) => continue,
                Ok(Some(mut bytecode)) => {
                    assembly.append(&mut bytecode);
                }
                Err(e) => anyhow::bail!("Error occured on line {}: {}", index + 1, e),
            }
        }

        Ok(assembly)
    }
    /// This is where we search for all the labels and store their locations
    fn assemble_prepass(&mut self) -> anyhow::Result<()> {
        let mut bytecode_index = 0;
        for (index, instruction) in self.source.lines().enumerate() {
            let instruction = match instruction.trim().split(';').next() {
                None => {
                    continue;
                }
                Some(instruction) => {
                    if instruction.is_empty() {
                        continue;
                    }
                    instruction
                }
            };
            if instruction.ends_with(':') {
                match self
                    .labels
                    .insert(instruction.replace(':', ""), bytecode_index * 3)
                {
                    None => continue,
                    Some(old) => {
                        anyhow::bail!(
                            "Label '{}' defined twice at lines {}, {}",
                            instruction.replace(':', ""),
                            old / 3 + 1,
                            index + 1
                        )
                    }
                }
            } else {
                bytecode_index += 1;
            }
        }
        Ok(())
    }
    fn assemble_instruction(&mut self, instruction: &str) -> anyhow::Result<Option<Vec<u16>>> {
        if instruction.ends_with(':') {
            return Ok(None); // We early return when it is a flag
        }
        let mut tokens = instruction.split_whitespace();
        let opcode = tokens
            .next()
            .ok_or(anyhow::anyhow!("Operation not specified"))?;

        let mut args: Vec<Argument> = Vec::new();
        for token in tokens {
            args.push(Argument::parse_argument(token)?);
        }

        let mut bytecode: Vec<u16> = Vec::new();

        match opcode.to_lowercase().as_str() {
            "add" => {
                bytecode.extend_from_slice(&self.convert_to_bytecode(
                    OpCode::Add,
                    args,
                    Some(|arg| arg.is_address() | arg.is_register()),
                    Some(|arg| arg.is_address() | arg.is_direct_value() | arg.is_register()),
                )?);
            }
            "sub" => {
                bytecode.extend_from_slice(&self.convert_to_bytecode(
                    OpCode::Sub,
                    args,
                    Some(|arg| arg.is_address() | arg.is_register()),
                    Some(|arg| arg.is_address() | arg.is_direct_value() | arg.is_register()),
                )?);
            }
            "mul" => {
                bytecode.extend_from_slice(&self.convert_to_bytecode(
                    OpCode::Mul,
                    args,
                    Some(|arg| arg.is_address() | arg.is_register()),
                    Some(|arg| arg.is_address() | arg.is_direct_value() | arg.is_register()),
                )?);
            }
            "div" => {
                bytecode.extend_from_slice(&self.convert_to_bytecode(
                    OpCode::Div,
                    args,
                    Some(|arg| arg.is_address() | arg.is_register()),
                    Some(|arg| arg.is_address() | arg.is_direct_value() | arg.is_register()),
                )?);
            }
            "exit" => {
                bytecode.extend_from_slice(&self.convert_to_bytecode(
                    OpCode::Exit,
                    args,
                    Some(|arg| {
                        arg.is_address()
                            | arg.is_empty()
                            | arg.is_direct_value()
                            | arg.is_register()
                    }),
                    None,
                )?);
            }
            "stall" => {
                bytecode.extend_from_slice(&self.convert_to_bytecode(
                    OpCode::Stall,
                    args,
                    Some(|arg| arg.is_address() | arg.is_empty() | arg.is_direct_value()),
                    None,
                )?);
            }
            "mov" => {
                bytecode.extend_from_slice(&self.convert_to_bytecode(
                    OpCode::Mov,
                    args,
                    Some(|arg| arg.is_address() | arg.is_register()),
                    Some(|arg| arg.is_address() | arg.is_direct_value() | arg.is_register()),
                )?);
            }
            "jmp" => {
                let arg0 = args
                    .first()
                    .ok_or(anyhow::anyhow!("Missing Argument for Jmp Operation"))?;

                bytecode.extend_from_slice(&self.convert_to_bytecode(
                    OpCode::Jmp,
                    args,
                    Some(|arg| arg.is_label()),
                    Some(|arg| arg.is_direct_value() | arg.is_address() | arg.is_empty()),
                )?);
            }
            other => {
                anyhow::bail!("Unknown operation: {}", other)
            }
        }
        Ok(Some(bytecode))
    }
    fn convert_to_bytecode(
        &mut self,
        opcode: OpCode,
        args: Vec<Argument>,
        first_filter: Option<fn(&Argument) -> bool>,
        second_filter: Option<fn(&Argument) -> bool>,
    ) -> anyhow::Result<[u16; 3]> {
        let arg0 = args.first().unwrap_or(&Argument::Empty);
        let arg1 = args.get(1).unwrap_or(&Argument::Empty);

        if let Some(is_valid) = first_filter
            && !is_valid(arg0)
        {
            anyhow::bail!("First argument is of invalid type: {:?}", arg0);
        }

        if let Some(is_valid) = second_filter
            && !is_valid(arg1)
        {
            anyhow::bail!("Second argument is of invalid type: {:?}", arg1);
        }

        let arg0_type_bc = arg0.as_bytecode()?;
        let arg1_type_bc = arg1.as_bytecode()?;

        Ok([
            ((opcode as u16) << 8) | (arg0_type_bc << 4) | arg1_type_bc,
            arg0.value_as_bytecode(&self.labels)?,
            arg1.value_as_bytecode(&self.labels)?,
        ])
    }
}

#[cfg(test)]
mod tests;
