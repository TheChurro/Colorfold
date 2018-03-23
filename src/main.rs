
#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate image;

// extern crate gfx_hal as hal;
// #[cfg(feature = "dx12")]
// extern crate gfx_backend_dx12 as back;
// #[cfg(feature = "vulkan")]
// extern crate gfx_backend_vulkan as back;
// #[cfg(feature = "metal")]
// extern crate gfx_backend_metal as back;

pub mod shaders;
pub mod color;
pub mod filters;
pub mod parser;

use filters::Palette;
use filters::Rotation::*;
use filters::Control::*;
use filters::Scale::*;
use color::Color;

use gfx::Device;

use gfx_window_glutin as gfx_glutin;

use glutin::{Event, WindowEvent, GlContext};

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

use std::{time, thread};

fn main()
{
    let mut event_loop = glutin::EventsLoop::new();

    let window_builder = glutin::WindowBuilder::new()
                                    .with_title("Colorfold Test".to_string())
                                    .with_dimensions(500, 588);

    let context = glutin::ContextBuilder::new()
                            .with_vsync(true);

    let (window, mut device, mut factory, main_color, mut _main_depth) =
            gfx_glutin::init::<ColorFormat, DepthFormat>(window_builder, context, &event_loop);

    let texture = shaders::load_texture(&mut factory, "resources/Test.jpg")
                                .expect("Failed to load texture");

    let mut palette = Palette::new();
    let rot_1 = SingleSingle(Point(Color(255, 0, 0)), Point(Color(0, 255, 255)));
    palette.filters.push((rot_1, RatioClamp));
    let rot_2 = SingleSingle(Point(Color(0, 255, 255)), Point(Color(255, 0, 0)));
    palette.filters.push((rot_2, RatioClamp));

    let (pipeline, window_pipe) = shaders::build_pipeline(&mut factory, &palette)
                                                .expect("Could not create pipeline");
    let mut encoder : gfx::Encoder<_, _> = factory.create_command_buffer().into();

    use gfx::Factory;
    {
        let texture_color = factory.view_texture_as_render_target(&texture.tex_target.clone(),
                                                                  0, None)
                                        .expect("Could not get texture as render target");

        let (img_data, img_slice) = shaders::build_square(&mut factory, texture_color.clone(),
                                                          &mut encoder, &texture);

        encoder.clear(&texture_color.clone(), [0.0, 0.0, 0.0, 1.0]);
        encoder.draw(&img_slice, &pipeline, &img_data);
        encoder.flush(&mut device);
        // window.swap_buffers().unwrap();
        device.cleanup();

        // TODO: Changle libraries so we can save images!!
    }

    use shaders::Texture;
    let Texture { width, height, tex_view:_, tex_target } = texture;

    use gfx::format::Swizzle;
    let tex_view = factory.view_texture_as_shader_resource::<ColorFormat>(&tex_target.clone(),
                                                                          (0, 1), Swizzle::new())
                                .expect("Could not convert texture into shader resource");
    let texture = Texture{ width, height, tex_view, tex_target };

    let (img_data, img_slice) = shaders::build_img(&mut factory, main_color.clone(),
                                                   &mut encoder, texture);

    let mut running = true;
    while running
    {
        event_loop.poll_events(|event| {
            match event
            {
                Event::WindowEvent{event, ..} => match event
                    {
                        WindowEvent::Closed        => running = false,
                        WindowEvent::Resized(w, h) => window.resize(w, h),
                        _ => {}
                    },
                _ => {}
            }
        });

        if running
        {
            encoder.clear(&main_color, [0.0, 0.0, 0.0, 1.0]);
            encoder.draw(&img_slice, &window_pipe, &img_data);
            encoder.flush(&mut device);
            window.swap_buffers().unwrap();
            device.cleanup();
            thread::sleep(time::Duration::from_millis(10));
        }
    }
}
