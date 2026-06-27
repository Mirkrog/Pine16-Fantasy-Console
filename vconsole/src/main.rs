use std::process;

use crate::runtime::Runtime;

mod runtime;

fn main() -> anyhow::Result<()> {
    let mut interpreter = Runtime::default();

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
