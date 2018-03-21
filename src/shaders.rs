
use gfx;
use image;

use gfx::Resources;
use gfx::Factory;
use gfx::Slice;
use gfx::CommandBuffer;
use gfx::Encoder;
use gfx::handle::{ShaderResourceView, RenderTargetView};

use gfx::traits::FactoryExt;
use gfx::{PipelineState, PipelineStateError};

pub type ColorFormat = gfx::format::Srgba8;

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
}

#[derive(Debug)]
pub enum   TextureError { ImageError(image::ImageError), GfxError(gfx::CombinedError) }
pub struct Texture<R : Resources>
{
    pub width    : u32,
    pub height   : u32,
    pub tex_view : ShaderResourceView<R, [f32; 4]>
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
                Ok((_, texture_view)) => Ok(Texture
                                           {
                                                width    : width,
                                                height   : height,
                                                tex_view : texture_view
                                            })
            }
        }
    }
}

pub fn build_pipeline<R: Resources, F : Factory<R>>(f : &mut F) ->
                    Result<PipelineState<R, pipe::Meta>, PipelineStateError<String>>
{
    f.create_pipeline_simple(
        include_bytes!("../shaders/test.vsh"),
        include_bytes!("../shaders/test.fsh"),
        pipe::new()
    )
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
                                                  tex     : Texture<R>)
                                                  -> (pipe::Data<R>, Slice<R>)
{
    let (buffer, slice) = factory.create_vertex_buffer_with_slice(&SQUARE, INDICES);
    let sampler = factory.create_sampler_linear();
    let dim_buffer = factory.create_constant_buffer(1);
    encoder.update_constant_buffer(&dim_buffer, &ImageSize{size:[tex.width as i32, tex.height as i32]});
    let data = pipe::Data
    {
        vbuf : buffer,
        img : (tex.tex_view, sampler),
        img_dims : dim_buffer,
        out : color
    };
    (data, slice)
}
