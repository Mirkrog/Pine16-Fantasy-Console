use std::process;

use crate::console::Console;

mod app;
mod console;

fn main() -> anyhow::Result<()> {
    let mut interpreter = Console::default();

    process::Command::new("cargo")
        .args(["run", "-p", "assembler"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    interpreter.load_rom_from_path("test.v16.o").unwrap();

    interpreter.run();

    Ok(())
}
