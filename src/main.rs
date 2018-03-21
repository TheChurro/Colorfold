
#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate image;

mod shaders;

use gfx::Device;

use gfx_window_glutin as gfx_glutin;

use glutin::{Event, WindowEvent, GlContext};

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

use std::{time, thread};

fn main()
{
    println!("Hello, world!");

    let mut event_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
                                    .with_title("Colorfold Test".to_string())
                                    .with_dimensions(500, 588);
    let context = glutin::ContextBuilder::new()
                            .with_vsync(true);
    let (window, mut device, mut factory, main_color, mut _main_depth) =
            gfx_glutin::init::<ColorFormat, DepthFormat>(window_builder, context, &event_loop);
    println!("Created Window");
    let texture = shaders::load_texture(&mut factory, "resources/Test.jpg")
                                .expect("Failed to load texture");

    println!("Texture size is {} {}", texture.width, texture.height);

    let pipeline = shaders::build_pipeline(&mut factory).expect("Could not create pipeline");
    let mut encoder : gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let (img_data, img_slice) = shaders::build_square(&mut factory, main_color.clone(),
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
            encoder.draw(&img_slice, &pipeline, &img_data);
            encoder.flush(&mut device);
            window.swap_buffers().unwrap();
            device.cleanup();
            thread::sleep(time::Duration::from_millis(10));
        }
    }
}
