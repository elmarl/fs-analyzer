use fs_analyzer::file_analyzer;
use std::{path::Path, ffi::OsString};
use rustop::opts;

fn main() {
    let (args, _rest) = opts! {
        synopsis "This app will list 5 largest items in the given directory.";          // short info message for the help page
        param dir:String, desc:"Input file name."; 
        opt num:u8=5, desc:"Number of files to list."; 
    }.parse_or_exit();
    let fixed_path = Path::new(&args.dir);
    let num_of_files:u8 = args.num;
    file_analyzer::start(OsString::from(&fixed_path), num_of_files).unwrap();
}
