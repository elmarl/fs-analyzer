use clap::Parser;
use fs_analyzer::file_analyzer;
use std::{path::PathBuf, time::Instant};

#[derive(Parser, Debug)]
#[command(name = "File Lister")]
#[command(author = "Your Name")]
#[command(version = "1.0")]
#[command(about = "Lists the largest files in a directory", long_about = None)]
struct Args {
    /// Input directory path
    #[arg(short, long, value_name = "DIR")]
    dir: PathBuf,

    /// Number of files to list
    #[arg(short, long, default_value_t = 5)]
    num: usize,
}

fn main() {
    let args = Args::parse();

    let start_time = Instant::now();

    if !args.dir.is_dir() {
        eprintln!("Error: The path provided is not a directory.");
        std::process::exit(1);
    }

    if let Err(e) = file_analyzer::start(args.dir, args.num) {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }

    let duration = start_time.elapsed();
    println!("Listed files in: {:.2?}", duration);
}
