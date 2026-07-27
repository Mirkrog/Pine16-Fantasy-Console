use std::process;

use anyhow::Ok;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::app::App;

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

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
