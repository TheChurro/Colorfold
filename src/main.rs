extern crate linked_hash_set;

#[macro_use]
extern crate serde;
extern crate ron;
extern crate serde_json;

extern crate env_logger;

extern crate structopt;

pub mod color;
pub mod data;
pub mod dependency;
pub mod filters;
pub mod folding;
pub mod geometry;
pub mod imaging;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "colorfold",
    about = "Convert colorfold specifications into vdmx-compliant shaders."
)]
struct Args {
    #[structopt(parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() {
    env_logger::init();
    let args = Args::from_args();

    use folding::FoldingMachine;
    use std::ffi::OsStr;
    use std::fs::File;
    let json_ext = Some(OsStr::new("json"));

    // Get command line arguments (excluding the program name)
    for file in args.files {
        println!("Running folding machine at {:#?}", &file);
        if let Ok(input) = File::open(&file) {
            let mut folder: FoldingMachine = if file.extension() == json_ext {
                match serde_json::from_reader(input) {
                    Ok(x) => x,
                    Err(e) => {
                        println!("Error: Malformed json input {:#?}.\nError: {}", &file, e);
                        continue;
                    }
                }
            } else {
                match ron::de::from_reader(input) {
                    Ok(x) => x,
                    Err(e) => {
                        println!("Error: Malformed ron input {:#?}.\nError: {}", &file, e);
                        continue;
                    }
                }
            };
            if let Err(e) = folder.with_location(file.clone()).vdmx_shader(0usize, true) {
                println!("Failed to save shader {:#?} due to error {}", &file, e);
            }
        } else {
            println!("Error: Could not open file {:#?} for read!", &file);
        }
    }
}
