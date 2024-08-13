#![feature(array_chunks)]
#![feature(iter_intersperse)]

use clap::Parser;
use std::vec::IntoIter;
use std::fs;
use std::iter::Zip;
use std::str::Chars;
use thiserror::Error;
use ttf_parser::{ Face };

mod border;
use border::Border;
mod character;
use character::{ PositionedCharacter, CharacterPosition };
mod stl;
use stl::Triangle;
mod svgbuilder;
mod triangles;
use triangles::Triangles;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about=None)]
struct Config {
    #[clap(value_parser)]
    text: String,
    #[clap(short, long, default_value_t=16.0)]
    size: f32,
    #[clap(short, long, default_value="output.stl")]
    output: String,
    #[clap(long, default_value_t=8)]
    resolution: u64,
    #[clap(long, default_value_t=1.0)]
    spacing: f32,
    #[clap(long, default_value_t=1.0)]
    border_thickness: f32,
    #[clap(short, long, default_value_t=25.0)]
    height: f32,
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
    let mut args = Config::parse();
    if let Err(e) = run(&mut args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run(args:&mut Config) -> Result<(), GeneratorError> {
    let font_data = fs::read("./ebas927.ttf")
        .map_err(|_| GeneratorError::NoFont)?;
    let font_face = Face::from_slice(&font_data, 0)?;

    let mut triangles = positioned_text(&mut args.text, args.size)
        .map(|(ch, pos)| Ok(PositionedCharacter::new(ch, &font_face, &pos)
            .ok_or(GeneratorError::NoCharacter(ch))?
            .to_triangles(args.resolution)?
            .iter()
            .map(|t| t.extrude(args.character_height, args.height))
            .flatten()
            .collect::<Vec<Triangle>>()))
        .collect::<Result<Vec<Vec<Triangle>>, GeneratorError>>()?
        .into_iter().flatten().collect::<Vec<Triangle>>();

    let outer_size = args.size + args.spacing * 2.0 + args.border_thickness * 2.0;
    
    // Add stamp body
    triangles.append(&mut Triangles::square(outer_size)?.extrude(args.height, 0.0));

    // Add border
    triangles.append(&mut Border::new(outer_size, args.border_thickness).to_triangles(args.resolution)?.extrude(args.character_height, args.height));

    let output = stl::generate_stl(&triangles);
    fs::write(&args.output, &output)
        .map_err(|e| GeneratorError::WriteError(e))?;

    Ok(())
}

fn positioned_text(text:&mut String, size:f32) -> Zip<Chars<'_>, IntoIter<CharacterPosition>> {
    let len = text.chars().count();
    if len == 1 {
        return text.chars().zip(vec![CharacterPosition::centered(size)]);
    } else if len == 2 {
        text.push_str("之印");
    } else if len == 3 {
        text.push_str("印");
    }

    text.chars()
        .zip(CharacterPosition::squares(size))
}