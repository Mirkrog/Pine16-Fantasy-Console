use std::process;

use winit::event_loop::{ControlFlow, EventLoop};

use crate::app::App;

mod app;
mod console;

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    process::Command::new("cargo")
        .args(["run", "-p", "assembler"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    //interpreter.load_rom_from_path("test.v16.o").unwrap();

    //interpreter.run();

    Ok(())
}
