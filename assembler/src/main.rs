use std::{fs::File, io::Write};

use anyhow::Ok;
use byteorder::{LittleEndian, WriteBytesExt};
use simple_stopwatch::Stopwatch;

use crate::assembler::Assembler;

mod assembler;

fn main() -> anyhow::Result<()> {
    let stopwatch = Stopwatch::start_new();
    let assembly = Assembler::new(include_str!("test.v16.asm")).assemble()?;

    let mut out_file = File::create("test.v16.o")?;

    let mut buffer = [0u8; 20];
    buffer[..17].copy_from_slice("V16ASSEMBLYCODE:D".as_bytes());
    buffer[17] = env!("CARGO_PKG_VERSION_MAJOR").parse::<u8>().unwrap();
    buffer[18] = env!("CARGO_PKG_VERSION_MINOR").parse::<u8>().unwrap();
    buffer[19] = env!("CARGO_PKG_VERSION_PATCH").parse::<u8>().unwrap();

    out_file.write_all(&buffer).unwrap();

    for bytepair in assembly {
        out_file.write_u16::<LittleEndian>(bytepair)?
    }

    print!("Finished assembling in: {}s", stopwatch.s());
    Ok(())
}
