#![cfg_attr(
    not(any(feature = "vulkan", feature = "dx12", feature = "metal")),
    allow(dead_code, unused_extern_crates, unused_imports)
)]

extern crate image;

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

    use folding::StageDesc;
    use folding::FoldingMachine;
    use filters::Palette;
    use filters::Control::*;
    use filters::Rotation::*;
    use filters::Scale::RatioClamp;
    use color::Color;

    let images = vec![("TestIn".to_owned(), "resources/Test.jpg".to_owned()),
                      ("TestOut".to_owned(), "resources/TestOut.png".to_owned())];
    let rot0 = SingleSingle(Point(Color(255, 0, 0)), Point(Color(128, 255, 0)));
    let rot1 = SingleSingle(Point(Color(255, 0, 0)), Point(Color(255, 0, 0)));
    let palette = Palette { filters : vec![(rot0, RatioClamp), (rot1, RatioClamp)] };
    let stage_descs = vec![StageDesc{
                                        input_ids : vec!["TestIn".to_owned()],
                                        output_id : "TestOut".to_owned(),
                                        palette : palette
                                    }];
    let mut folder = <folding::FoldingMachine<<back::Instance as hal::Instance>::Backend, hal::Compute>>::new(images, stage_descs);
    folder.compute_stage(0usize).expect("Could not compute");

    use std::fs::File;
    use std::io::Write;
    let mut machine_out = File::create("resources/test_machine_file.json").unwrap();
    serde_json::to_writer_pretty(machine_out, &folder);
}

#[cfg(not(any(feature = "vulkan", feature = "dx12", feature = "metal")))]
fn main()
{

}
