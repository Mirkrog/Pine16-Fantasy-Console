use std::process;

use crate::interpreter::Interpreter;

mod interpreter;

fn main() -> anyhow::Result<()> {
    let mut interpreter = Interpreter::default();

    process::Command::new("cargo")
        .args(["run", "-p", "assembler"])
        .spawn()
        .expect("Command failed to start")
        .wait()
        .unwrap();

    interpreter.load_rom_from_path("test.v16.o").unwrap();

    interpreter.run();

    Ok(())
}
