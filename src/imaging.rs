
use image;
use std::io;

use image::{ImageBuffer, Rgba, ImageError};

use serde::{Serialize, Serializer, Deserialize, Deserializer};

pub struct Image
{
    pub data      : Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    pub location  : String,
    pub id        : String,
    pub is_loaded : bool,
}

impl Image
{
    pub fn new(id : String, location : String) -> Image
    {
        Image
        {
            data : None,
            location,
            id,
            is_loaded : false,
        }
    }

    pub fn load_image(&mut self) -> Result<(), ImageError>
    {
        match image::open(self.location.clone())
        {
            Err(x)  =>
            {
                self.data = None;
                self.is_loaded = false;
                Err(x)
            },
            Ok(img) =>
            {
                self.data = Some(img.to_rgba());
                self.is_loaded = true;
                Ok(())
            }
        }
    }

    pub fn load_u32_vec(&mut self) -> Result<Vec<u32>, ImageError>
    {
        if !self.is_loaded
        {
            if let Err(x) = self.load_image()
            {
                return Err(x);
            }
        }

        let mut outs = Vec::new();
        let raw_u8 = self.data.clone().unwrap().into_raw();

        for i in 0..(raw_u8.len()/4)
        {
            let mut value = 0u32;
            value |= raw_u8[i * 4] as u32;
            value |= (raw_u8[i * 4 + 1] as u32) << 8;
            value |= (raw_u8[i * 4 + 2] as u32) << 16;
            value |= (raw_u8[i * 4 + 3] as u32) << 24;
            outs.push(value);
        }

        Ok(outs)
    }

    pub fn save_image(&mut self, data: Vec<u8>, width : u32, height : u32) -> Result<(), io::Error>
    {
        image::save_buffer(self.location.clone(), data.as_slice(), width, height, image::ColorType::RGBA(8))
    }

    pub fn save_u32_vec(&mut self, data : Vec<u32>, width : u32, height : u32) -> Result<(), io::Error>
    {
        let mut img_data = Vec::new();

        for v in data
        {
            img_data.push((v & 255) as u8);
            img_data.push(((v >> 8) & 255) as u8);
            img_data.push(((v >> 16) & 255) as u8);
            img_data.push(((v >> 24) & 255) as u8);
        }

        self.data = Some(ImageBuffer::from_raw(width, height, img_data.clone()).expect("Not valid data"));
        self.save_image(img_data, width, height)
    }
}

impl Clone for Image
{
    fn clone(&self) -> Image
    {
        Image::new(self.id.clone(), self.location.clone())
    }
}

// ================================================================================================
// == Serde Serialization for parsing input files.                                               ==
// ================================================================================================
#[derive(Serialize, Deserialize)]
struct SerializableImage {
    name : String,
    location : String,
}

impl Serialize for Image {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        SerializableImage { name: self.id.clone(), location: self.location.clone() }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Image {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        Deserialize::deserialize(deserializer)
            .map(|SerializableImage { name, location }| Image::new(name, location))
    }
}
