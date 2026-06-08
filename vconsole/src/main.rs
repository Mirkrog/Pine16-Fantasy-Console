use crate::interpreter::Interpreter;

mod interpreter;

fn main() -> anyhow::Result<()> {
    let mut interpreter = Interpreter::default();

    interpreter.load_rom_from_path("test.v16.o").unwrap();

    interpreter.run();

    Ok(())
}
