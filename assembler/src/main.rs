use anyhow::Ok;

use crate::assembler::Assembler;

mod assembler;

fn main() -> anyhow::Result<()> {
    let assembly = Assembler::new(include_str!("test.v16.asm")).assemble()?;
    
    for value in assembly {
        println!("{:016b},", value)
    }
    Ok(())
}
