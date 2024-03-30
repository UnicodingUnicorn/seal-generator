#![feature(array_chunks)]
#![feature(iter_intersperse)]

use clap::Parser;
use std::fs;
use thiserror::Error;
use ttf_parser::{ Face };

mod character;
use character::{ PositionedCharacter, CharacterPosition };
mod stl;
use stl::Triangle;
mod svgbuilder;
mod triangles;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about=None)]
struct Config {
    #[clap(value_parser)]
    text: String,
    #[clap(short, long, default_value_t=16)]
    size: u16,
    #[clap(short, long, default_value="output.stl")]
    output: String,
    #[clap(short, long, default_value_t=8)]
    resolution: u64,
    #[clap(long, default_value_t=2.0)]
    character_height: f32,
}

#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("character {0} cannot be found")]
    NoCharacter(char),
    #[error("the font file could not be found")]
    NoFont,
    #[error("{0}")]
    FontError(#[from] ttf_parser::FaceParsingError),
    #[error("could not write output: {0}")]
    WriteError(std::io::Error),
    #[error("{0}")]
    TriangulationError(#[from] earcutr::Error),
}

fn main() {
    let args = Config::parse();
    if let Err(e) = run(&args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run(args:&Config) -> Result<(), GeneratorError> {
    let font_data = fs::read("./ebas927.ttf")
        .map_err(|_| GeneratorError::NoFont)?;
    let font_face = Face::from_slice(&font_data, 0)?;

    // TODO: Pad characters if input string is less than 4
    // TODO: Handle single character seals

    let triangles = args.text.chars()
        .zip(CharacterPosition::squares(args.size as i16))
        .map(|(ch, pos)| Ok(PositionedCharacter::new(ch, &font_face, &pos)
            .ok_or(GeneratorError::NoCharacter(ch))?
            .to_triangles(args.resolution)?
            .iter()
            .map(|t| t.extrude(args.character_height, 0.0))
            .flatten()
            .collect::<Vec<Triangle>>()))
        .collect::<Result<Vec<Vec<Triangle>>, GeneratorError>>()?
        .into_iter().flatten().collect::<Vec<Triangle>>();

    let output = stl::generate_stl(&triangles);
    fs::write(&args.output, &output)
        .map_err(|e| GeneratorError::WriteError(e))?;

    Ok(())
}