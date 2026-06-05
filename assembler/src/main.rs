use std::fs::File;

use anyhow::Ok;
use byteorder::{BigEndian, WriteBytesExt};
use simple_stopwatch::Stopwatch;

use crate::assembler::Assembler;

mod assembler;

fn main() -> anyhow::Result<()> {
    let stopwatch = Stopwatch::start_new();
    let assembly = Assembler::new(include_str!("test.v16.asm")).assemble()?;

    let mut out_file = File::create("test.v16.o")?;

    for bytepair in assembly {
        out_file.write_u16::<BigEndian>(bytepair)?
    }

    print!("Finished assembling in: {}s", stopwatch.s());

    Ok(())
}
