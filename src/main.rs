use fs_analyzer::file_analyzer;
use std::{path::Path, ffi::OsString};
use rustop::opts;

fn main() {
    let (args, _rest) = opts! {
        synopsis "This is a simple test program.";          // short info message for the help page
        opt verbose:bool, desc:"Be verbose.";               // a flag -v or --verbose
        param dir:String, desc:"Input file name."; 
    }.parse_or_exit();
    let fixed_path = Path::new(&args.dir);
    file_analyzer::start(OsString::from(&fixed_path)).unwrap();
}
