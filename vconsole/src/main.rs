use anyhow::Ok;

mod app;
mod console;
mod renderer;

use std::{path::PathBuf, process};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the cartridge to run
    #[arg()]
    cart_path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(extension) = args.cart_path.extension()
        && extension != "pinecart"
    {
        eprintln!(
            "Failed to open cartridge file {:?} Wrong file extension, expected \".pinecart\"",
            args.cart_path.as_path(),
        );
        process::exit(1);
    }

    app::run(args.cart_path)?;

    Ok(())
}
