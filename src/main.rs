#![cfg_attr(
    not(any(feature = "vulkan", feature = "dx12", feature = "metal")),
    allow(dead_code, unused_extern_crates, unused_imports)
)]

extern crate image;
extern crate linked_hash_set;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate env_logger;
extern crate gfx_hal as hal;
#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;

extern crate glsl_to_spirv;

pub mod color;
pub mod geometry;
pub mod filters;
pub mod folding;
pub mod imaging;

#[cfg(any(feature = "vulkan", feature = "dx12", feature = "metal"))]
fn main()
{
    env_logger::init();

    use folding::FoldingMachine;
    use std::fs::File;
    use std::env;

    // Get command line arguments (excluding the program name)
    for arg in env::args().skip(1)
    {
        println!("Running folding machine at {}", arg);
        if let Ok(mut input) = File::open(arg)
        {
            match serde_json::from_reader(input)
            {
                Ok(folder)  =>
                {
                    let mut folder : FoldingMachine<<back::Instance as hal::Instance>::Backend, hal::Compute> = folder;
                    match folder.compute_stage(0usize)
                    {
                        Ok(()) => { println!("Completed!"); },
                        Err(x) => { println!("Compute Err: {}", x); }
                    }
                },
                Err(x) => { println!("Error: Malformed input file! {}", x); }
            }
        }
        else
        {
            println!("Error: Could not open file!");
        }
    }
}


#[cfg(not(any(feature = "vulkan", feature = "dx12", feature = "metal")))]
fn main()
{

}
