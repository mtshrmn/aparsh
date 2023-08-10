use crate::parser::FlagParser;
use clap::Parser;
use std::path::PathBuf;

mod parser;

#[derive(clap::Parser, Debug)]
struct Args {
    path: PathBuf,
}

fn main() {
    let args = Args::parse();
    let content = match std::fs::read_to_string(&args.path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error reading file {}: {}", args.path.display(), e);
            std::process::exit(e.raw_os_error().unwrap_or(1));
        }
    };

    let fp = FlagParser::new(&content).unwrap();
    println!("{}", fp.render());
}
