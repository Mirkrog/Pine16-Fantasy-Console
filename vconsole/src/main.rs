use clap::{Parser, builder::Str};
use std::{path::PathBuf, process};

mod app;
mod audiomodule;
mod console;
mod renderer;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the cartridge to run
    cart_path: PathBuf,
    /// Enables guardrails which prevent undefined behavior
    #[arg(short, long, default_value_t = false)]
    guardrails: bool,
    /// Sets the log-level for the console, possible values: 'Off', 'Error', 'Warn', 'Info', 'Debug', 'Trace'
    #[arg(short, long = "log-level", default_value_t = log::LevelFilter::Info)]
    log_level: log::LevelFilter,
    /// Sets the log-scope for the console, use 'all' to enable all logs
    #[arg(short = 's', long = "log-scope", default_value_t = "vconsole".to_string())]
    log_scope: String,
}

fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    if let Some(extension) = args.cart_path.extension()
        && extension != "pinecart"
    {
        eprintln!(
            "Failed to open cartridge file {:?} Wrong file extension, expected \".pinecart\"",
            args.cart_path.as_path(),
        );
        process::exit(1);
    }

    if args.log_scope == "all" {
        env_logger::builder().filter_level(args.log_level).init();
    } else {
        env_logger::builder()
            .filter_module(args.log_scope.as_str(), args.log_level)
            .init();
    }
    app::run(args.cart_path, args.guardrails)?;

    Ok(())
}
