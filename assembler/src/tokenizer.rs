#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Value {
    DirectValue(u16), // In asm: $
    Address(u16), // In asm: @
}
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Token {
    OpCode(String),
    Value(Value),
    Flag(String),
}

pub struct Tokenizer<'a> {
    instruction_slices: std::iter::Enumerate<std::str::SplitWhitespace<'a>>,
}
impl<'a> Tokenizer<'a> {
    pub fn new(instruction: &'a str) -> Self {
        Self {
            instruction_slices: instruction.split(&[';', '#'][..]).next().expect("Empty instruction").split_whitespace().enumerate(),
        }
    }
    pub fn parse_next_token(&mut self) -> anyhow::Result<Option<Token>> {
        let (index, text) = match self.instruction_slices.next() {
            Some(slice) => slice,
            None => return Ok(None),
        };

        let (first_char, rest) = text
            .split_at_checked(1)
            .ok_or_else(|| anyhow::anyhow!("Malformed Argument {}: '{}'", index, text))?;

        let token = match first_char {
            "$" => Token::Value(Value::DirectValue(rest.parse::<u16>().map_err(|e| {
                anyhow::anyhow!(
                    "Error encountered while parsing Argument {} '{}':  {}",
                    index,
                    text,
                    e
                )
            })?)),
            "@" => Token::Value(Value::Address(rest.parse::<u16>().map_err(|e| {
                anyhow::anyhow!(
                    "Error encountered while parsing Argument {} '{}':  {}",
                    index,
                    text,
                    e
                )
            })?)),
            other => {
                if other.is_ascii() {
                    if index == 0 {
                        Token::OpCode(text.to_string())
                    } else {
                        Token::Flag(text.to_string())
                    }
                } else {
                    anyhow::bail!(
                        "Argument {} is not a valid OpCode or Descriptor: '{}'",
                        index,
                        text
                    )
                }
            }
        };

        Ok(Some(token))
    }
}
