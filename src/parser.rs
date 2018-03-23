#![cfg(any(feature = "vulkan", feature = "dx12", feature = "metal"))]
use hal;
use back;

use color::Color;
use folding;
use folding::{FoldingMachine, StageDesc};
use filters::Palette;
use filters::Control::*;
use filters::Rotation::*;
use filters::Scale::*;

use std::io::{BufReader, BufRead, self};
use std::prelude::*;
use std::fs::File;

#[derive(Debug)]
pub enum ParsingError
{
    FileError(io::Error),
    InvalidToken(String, usize)
}

pub fn parse_filter(file_name : &str) -> Result<FoldingMachine<<back::Instance as hal::Instance>::Backend, hal::Compute>, ParsingError>
{

    use parser::ParsingError::*;
    // Open a buffered reader for the given file
    match File::open(file_name)
    {
    Ok(f) =>
    {
        let reader = BufReader::new(f);

        let mut images = Vec::new();
        let mut stages = Vec::new();

        for (i, line) in reader.lines().enumerate()
        {
            match line
            {
            Ok(line) =>
            {
                let split = line.split(':');
            },
            Err(x) => {return Err(FileError(x));}
            }
        }

        // TODO: Parse file into palette and stage

        Ok(<folding::FoldingMachine<<back::Instance as hal::Instance>::Backend, hal::Compute>>::new(images, stages))
    }
    Err(x) => Err(FileError(x))
    }

}
