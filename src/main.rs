extern crate linked_hash_set;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate env_logger;

pub mod color;
pub mod filters;
pub mod folding;
pub mod geometry;
pub mod imaging;

fn main() {
    env_logger::init();

    use folding::FoldingMachine;
    use std::env;
    use std::fs::File;

    // Get command line arguments (excluding the program name)
    for arg in env::args().skip(1) {
        println!("Running folding machine at {}", arg);
        if let Ok(input) = File::open(&arg) {
            match serde_json::from_reader(input) {
                Ok(folder) => {
                    let mut folder: FoldingMachine = folder;
                    if let Err(e) = folder.vdmx_shader(0usize, true) {
                        println!("Failed to save shader {} due to error {}", &arg, e);
                    }
                }
                Err(x) => {
                    println!("Error: Malformed input file! {}", x);
                }
            }
        } else {
            println!("Error: Could not open file {}!", &arg);
        }
    }
}
