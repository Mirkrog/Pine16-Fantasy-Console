use std::process;

use anyhow::Ok;

mod app;
mod console;
mod renderer;

fn main() -> anyhow::Result<()> {
    if !process::Command::new("cargo")
        .args(["run", "-p", "assembler", "--release"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success()
    {
        process::exit(0) // preventing "double exit"
    }

    app::run()?;

    Ok(())
}
