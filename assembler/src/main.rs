use crate::assembler::Assembler;

mod assembler;
mod tokenizer;

fn main() {
    let assembly = Assembler::new(include_str!("test.v16.asm")).assemble();

    println!("{:#?}", assembly)
}
