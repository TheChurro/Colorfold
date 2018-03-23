
use gfx;
use image;

use gfx::memory::Usage::*;
use gfx::Resources;
use gfx::Factory;
use gfx::Slice;
use gfx::CommandBuffer;
use gfx::Encoder;
use gfx::handle::{ShaderResourceView, RenderTargetView};

use gfx::traits::FactoryExt;
use gfx::{PipelineState, PipelineStateError};

pub type ColorFormat = gfx::format::Srgba8;
pub type SurfaceType = gfx::format::R8_G8_B8_A8;

use filters::Palette;

gfx_defines! {

    vertex Vertex {
        pos:    [f32; 2] = "pos",
        uv_pos: [f32; 2] = "uv_pos",
    }

    constant ImageSize {
        size : [i32; 2] = "size",
    }

    pipeline pipe {
        vbuf : gfx::VertexBuffer<Vertex> = (),
        img  : gfx::TextureSampler<[f32; 4]> = "img",
        img_dims : gfx::ConstantBuffer<ImageSize> = "img_dims",
        out  : gfx::RenderTarget<ColorFormat> = "Target0",
    }

    pipeline window_pipe {
        vbuf : gfx::VertexBuffer<Vertex> = (),
        img  : gfx::TextureSampler<[f32; 4]> = "img",
        out  : gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

#[derive(Debug)]
pub enum   TextureError { ImageError(image::ImageError), GfxError(gfx::CombinedError) }
pub struct Texture<R : Resources>
{
    pub width      : u32,
    pub height     : u32,
    pub tex_view   : ShaderResourceView<R, [f32; 4]>,
    pub tex_target : gfx::handle::Texture<R, SurfaceType>
}

pub fn load_texture<F, R>(factory : &mut F, path : &str)
                            -> Result<Texture<R>, TextureError>
    where F: Factory<R>, R: Resources
{
    match image::open(path) {
        Err(x)  => Err(TextureError::ImageError(x)),
        Ok(img) =>
        {
            let img = img.to_rgba();
            let (width, height) = img.dimensions();
            let kind = gfx::texture::Kind::D2(width as u16, height as u16,
                                              gfx::texture::AaMode::Single);
            match factory.create_texture_immutable_u8::<ColorFormat>(kind, &[&img])
            {
                Err(x) => Err(TextureError::GfxError(x)),
                Ok((_, texture_view)) =>
                {
                    let out_texture = factory.create_texture::<SurfaceType>(kind, 1,
                                                             gfx::SHADER_RESOURCE | gfx::RENDER_TARGET,
                                                             Data, None)
                                                     .expect("Could not create texture");

                    Ok(Texture
                      {
                            width      : width,
                            height     : height,
                            tex_view   : texture_view,
                            tex_target : out_texture,
                      })
                }
            }
        }
    }
}

const LIB        : &'static [u8; 5546] = include_bytes!("../shaders/lib.fsh");
const MAIN_BEGIN : &'static [u8; 422]  = include_bytes!("../shaders/main_beginning.fsh");
const MAIN_END   : &'static [u8; 182]  = include_bytes!("../shaders/main_end.fsh");
use std;

pub fn save_shader(palette : &Palette, out_file : String) -> Result<(), std::io::Error>
{
    let shading_body = palette.shader("color_vec".to_owned(), "out_vec".to_owned(),
                                      "total".to_owned(), "num_zeros".to_owned());
    let shading_body = shading_body.as_bytes();
    let mut fragment = LIB.to_vec();
    fragment.extend_from_slice(MAIN_BEGIN);
    fragment.extend_from_slice(shading_body);
    fragment.extend_from_slice(MAIN_END);

    use std::fs::File;
    use std::io::prelude::*;

    let mut file = File::create(out_file)?;
    file.write_all(fragment.as_slice())?;
    Ok(())
}

pub fn build_pipeline<R: Resources, F : Factory<R>>(f : &mut F, palette : &Palette) ->
                    Result<(PipelineState<R, pipe::Meta>, PipelineState<R, window_pipe::Meta>),
                            PipelineStateError<String>>
{
    let shading_body = palette.shader("color_vec".to_owned(), "out_vec".to_owned(),
                                      "total".to_owned(), "num_zeros".to_owned());
    let shading_body = shading_body.as_bytes();
    let mut fragment = LIB.to_vec();
    fragment.extend_from_slice(MAIN_BEGIN);
    fragment.extend_from_slice(shading_body);
    fragment.extend_from_slice(MAIN_END);

    let filter_pso = f.create_pipeline_simple(
        include_bytes!("../shaders/vertex.vsh"),
        fragment.as_slice(),
        pipe::new());
    let render_pso = f.create_pipeline_simple(
         include_bytes!("../shaders/vertex.vsh"),
         include_bytes!("../shaders/default_img.fsh"),
         window_pipe::new());
    match filter_pso
    {
        Ok(filter_pso) =>
        {
            match render_pso
            {
                Ok(render_pso) =>
                {
                    Ok((filter_pso, render_pso))
                },
                Err(x) => Err(x)
            }
        },
        Err(x) => Err(x)
    }
}

const SQUARE : [Vertex; 4] = [
    Vertex { pos : [1.0, -1.0], uv_pos : [1.0, 1.0] },
    Vertex { pos : [-1.0, -1.0], uv_pos : [0.0, 1.0] },
    Vertex { pos : [-1.0, 1.0], uv_pos : [0.0, 0.0] },
    Vertex { pos : [1.0, 1.0], uv_pos : [1.0, 0.0] }
];

const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];


pub fn build_square<R: Resources, F : Factory<R>, B : CommandBuffer<R>>
                                                 (factory : &mut F,
                                                  color   : RenderTargetView<R, ColorFormat>,
                                                  encoder : &mut Encoder<R, B>,
                                                  tex     : &Texture<R>)
                                                  -> (pipe::Data<R>, Slice<R>)
{
    let (buffer, slice) = factory.create_vertex_buffer_with_slice(&SQUARE, INDICES);
    let sampler = factory.create_sampler_linear();
    let dim_buffer = factory.create_constant_buffer(1);
    encoder.update_constant_buffer(&dim_buffer, &ImageSize{size:[tex.width as i32, tex.height as i32]});
    let data = pipe::Data
    {
        vbuf : buffer,
        img : (tex.tex_view.clone(), sampler),
        img_dims : dim_buffer,
        out : color
    };
    (data, slice)
}

pub fn build_img<R: Resources, F : Factory<R>, B : CommandBuffer<R>>
                                                 (factory : &mut F,
                                                  color   : RenderTargetView<R, ColorFormat>,
                                                  encoder : &mut Encoder<R, B>,
                                                  tex     : Texture<R>)
                                                  -> (window_pipe::Data<R>, Slice<R>)
{
    let (buffer, slice) = factory.create_vertex_buffer_with_slice(&SQUARE, INDICES);
    let sampler = factory.create_sampler_linear();
    let dim_buffer = factory.create_constant_buffer(1);
    encoder.update_constant_buffer(&dim_buffer, &ImageSize{size:[tex.width as i32, tex.height as i32]});
    let data = window_pipe::Data
    {
        vbuf : buffer,
        img : (tex.tex_view, sampler),
        out : color
    };
    (data, slice)
}
