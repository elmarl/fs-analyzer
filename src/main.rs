use fs_analyzer::file_analyzer;
use std::{path::Path, fs::DirEntry, ffi::OsString};
use rustop::opts;

fn print_name(entry: &DirEntry) {
    println!("{}", entry.file_name().to_str().unwrap())
}

fn main() {
    let (args, _rest) = opts! {
        synopsis "This is a simple test program.";          // short info message for the help page
        opt verbose:bool, desc:"Be verbose.";               // a flag -v or --verbose
        param dir:String, desc:"Input file name."; 
    }.parse_or_exit();
    // let s = ".";
    let fixed_path = Path::new(&args.dir);
    // file_analyzer::read_dir(&fixed_path).unwrap();    
    // file_analyzer::visit_dirs(&fixed_path, &print_name).unwrap();    

    // file_analyzer::start(OsString::from(&fixed_path)).unwrap();
    file_analyzer::start(OsString::from(&fixed_path)).unwrap();

    println!("Hello, world!");
}
