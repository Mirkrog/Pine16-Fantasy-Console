mod assembler;
mod assemblererror;

use std::{fs, path::PathBuf, process};

use clap::Parser;
use colored::Colorize;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Source file that should be assembled
    #[arg()]
    source: PathBuf,

    /// Output file to write to (if left out, the input filename is used for lookup)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    if args.source.extension().and_then(|s| s.to_str()) != Some("pineasm") {
        eprintln!(
            "{} to open source file {:?} Wrong file extension, expected \".pineasm\"",
            "Failed".red().bold(),
            args.source.as_path(),
        );
        process::exit(1);
    }

    let source_string = fs::read_to_string(&args.source).unwrap_or_else(|e| {
        eprintln!(
            "{} to open source file {:?} {}",
            "Failed".red().bold(),
            args.source.as_path(),
            e
        );
        process::exit(1);
    });

    let output_path = match args.output {
        Some(path) => path,
        None => {
            let mut path = PathBuf::new();
            path.set_file_name(args.source.file_stem().expect("input file has no prefix"));
            path.set_extension("pinecart");
            path
        }
    };

    assembler::run_assembler(
        source_string,
        args.source.file_name().unwrap().to_owned(),
        output_path,
    );
}
