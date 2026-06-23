use byteorder::{LittleEndian, WriteBytesExt};
use colored::Colorize;
use simple_stopwatch::Stopwatch;
use std::{fs::File, io::Write};

mod assembler;
mod assemblererror;

use assembler::Assembler;

fn main() {
    let stopwatch = Stopwatch::start_new();

    // 1. Catch the result of assemble() manually
    let assembly = match Assembler::new(include_str!("test.v16.asm")).assemble() {
        Ok(data) => data,
        Err(err) => {
            eprintln!("{} {}", "Error:".red().bold(), err);
            std::process::exit(1);
        }
    };

    let mut out_file = match File::create("test.v16.o") {
        Ok(file) => file,
        Err(err) => {
            eprintln!(
                "{} Failed to create output file: {:?}",
                "Error:".red().bold(),
                err
            );
            std::process::exit(1);
        }
    };

    let mut buffer = [0u8; 20];
    buffer[..17].copy_from_slice("V16ASSEMBLYCODE:D".as_bytes());
    buffer[17] = env!("CARGO_PKG_VERSION_MAJOR").parse::<u8>().unwrap();
    buffer[18] = env!("CARGO_PKG_VERSION_MINOR").parse::<u8>().unwrap();
    buffer[19] = env!("CARGO_PKG_VERSION_PATCH").parse::<u8>().unwrap();

    out_file.write_all(&buffer).unwrap();

    for bytepair in assembly {
        out_file.write_u16::<LittleEndian>(bytepair).unwrap();
    }

    println!("Finished assembling in: {}s", stopwatch.s());
}
