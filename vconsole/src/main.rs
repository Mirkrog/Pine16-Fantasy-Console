use std::process;

use winit::event_loop::{ControlFlow, EventLoop};

use crate::app::App;

mod app;
mod console;
mod renderer;

fn main() -> anyhow::Result<()> {
    process::Command::new("cargo")
        .args(["run", "-p", "assembler"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    //interpreter.load_rom_from_path("test.v16.o").unwrap();

    //interpreter.run();

    Ok(())
}
