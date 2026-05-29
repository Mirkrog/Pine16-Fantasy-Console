use std::collections::HashMap;

use crate::tokenizer::{Token, Tokenizer, Value};

enum OpCode {
    Add,
    Sub,
}

pub struct Assembler<'a> {
    instructions: std::str::Lines<'a>,
    flags: HashMap<String, usize>,
}
impl<'a> Assembler<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            instructions: source.lines(),
            flags: HashMap::new(),
        }
    }
    pub fn assemble(&mut self) -> Vec<u16> {
        let mut assembly: Vec<u16> = Vec::new();

        self.assemble_prepass();

        loop {
            match self.assemble_next_instruction() {
                None => break,
                Some(Ok(mut bytecode)) => {
                    assembly.append(&mut bytecode);
                }
                Some(Err(e)) => panic!("{}", e)
            }
        }

        assembly
    }
    fn assemble_prepass(&mut self) {
        let mut instructions = self.instructions.clone().enumerate();
        loop {
            let (index, instruction) = match instructions.next() {
                None => {
                    return;
                }
                Some(instruction) => instruction,
            };

            if instruction.ends_with(':') {
                let sliced_instruction = instruction.replace(':', "");

                match self.flags.insert(sliced_instruction.clone(), index) {
                    None => {  },
                    Some(old_value) => println!(
                        "Warning Flag '{}' defined twice at lines: {}, {}",
                        sliced_instruction, old_value, index
                    ),
                }
            }
        }
    }
    fn assemble_next_instruction(&mut self) -> Option<anyhow::Result<Vec<u16>>> {
        let instruction = self.instructions.next()?;
        let mut tokens = Tokenizer::new(instruction);

        let mut bytecode: Vec<u16> = Vec::new();
        loop {
            let opcode = match tokens.parse_next_token() {
                Ok(None) => break,
                Ok(Some(Token::OpCode(token))) => token,
                Err(e) => return Some(Err(anyhow::anyhow!("{}", e))),
                Ok(Some(token)) => {
                    return Some(Err(anyhow::anyhow!(
                        "Token: '{:?}' is not an opcode",
                        token
                    )));
                }
            };
            let mut args = Vec::new();
            loop {
                args.push(match tokens.parse_next_token() {
                    Ok(None) => break,
                    Ok(Some(token)) => token,
                    Err(e) => return Some(Err(anyhow::anyhow!("{}", e))),
                });
            }

            match (opcode.to_lowercase().as_str(), args.as_slice()) {
                (
                    "add",
                    [
                        Token::Value(address),
                        Token::Value(address1),
                    ],
                ) => {
                    bytecode.push(OpCode::Add as u16);
                    match address {
                        Value::Address(address) => {
                            bytecode.push(0);
                            bytecode.push(*address);
                        },
                        Value::DirectValue(address) => {
                            bytecode.push(1);
                            bytecode.push(*address);
                        }
                    }
                    match address1 {
                        Value::Address(address) => {
                            bytecode.push(0);
                            bytecode.push(*address);
                        },
                        Value::DirectValue(address) => {
                            bytecode.push(1);
                            bytecode.push(*address);
                        }
                    }
                }
                (
                    "sub",
                    [
                        Token::Value(address),
                        Token::Value(address1),
                    ],
                ) => {
                    bytecode.push(OpCode::Sub as u16);
                    match address {
                        Value::Address(address) => {
                            bytecode.push(0);
                            bytecode.push(*address);
                        },
                        Value::DirectValue(address) => {
                            bytecode.push(1);
                            bytecode.push(*address);
                        }
                    }
                    match address1 {
                        Value::Address(address) => {
                            bytecode.push(0);
                            bytecode.push(*address);
                        },
                        Value::DirectValue(address) => {
                            bytecode.push(1);
                            bytecode.push(*address);
                        }
                    }
                }
                (opcode, args) => {
                    return Some(Err(anyhow::anyhow!(
                        "Argument could not be parsed: {:?} {:?}",
                        opcode,
                        args
                    )));
                }
            }
        }
        Some(Ok(bytecode))
    }
}
