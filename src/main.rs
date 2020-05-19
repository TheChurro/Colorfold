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
    /// List of Colorfold Descriptors to turn into a fragment shader.
    #[structopt(parse(from_os_str))]
    descriptors: Vec<PathBuf>,
    /// List of Colorfold Descriptors to convert from RON form to JSON form.
    #[structopt(short="j", long="to-json", parse(from_os_str))]
    convert_to_json: Vec<PathBuf>,
    /// List of Colorfold Descriptors to convert from JSON form to RON form.
    #[structopt(short="r", long="to-ron", parse(from_os_str))]
    convert_to_ron:  Vec<PathBuf>,
}

fn main() {
    env_logger::init();
    let args = Args::from_args();

    use folding::FoldingMachine;
    use std::ffi::OsStr;
    use std::fs::File;
    let json_ext = Some(OsStr::new("json"));

    for file in args.convert_to_ron {
        if let Ok(input) = File::open(&file) {
            if let Ok(folder) = serde_json::from_reader::<_, FoldingMachine>(input) {
                if let Ok(ron_form) = ron::ser::to_string_pretty(&folder, Default::default()) {
                    if let Err(e) = std::fs::write(file.with_extension("ron"), &ron_form) {
                        println!("Failed to write {:#?} to ron!\nError: {}", &file, e);
                    }
                }
            }
        }
    }

    for file in args.convert_to_json {
        if let Ok(input) = File::open(&file) {
            if let Ok(folder) = ron::de::from_reader::<_, FoldingMachine>(input) {
                if let Ok(output) = File::create(file.with_extension("json")) {
                    if let Err(e) = serde_json::to_writer_pretty(output, &folder) {
                        println!("Failed to write {:#?} to json!\nError: {}", &file, e);
                    }
                }
            }
        }
    }

    // Get command line arguments (excluding the program name)
    for file in args.descriptors {
        println!("Running folding machine at {:#?}", &file);
        if let Ok(input) = File::open(&file) {
            let folder: FoldingMachine = if file.extension() == json_ext {
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
