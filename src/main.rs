#![feature(array_chunks)]
#![feature(iter_intersperse)]

use clap::Parser;
use std::fs;
use thiserror::Error;
use ttf_parser::{ Face };

mod character;
use character::{ PositionedCharacter, CharacterPosition };
mod svgbuilder;
mod triangles;
use triangles::Triangles;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about=None)]
struct Config {
    #[clap(value_parser)]
    text: String,
    #[clap(short, long, default_value_t=16)]
    size: u16,
    #[clap(short, long)]
    output: Option<String>,
    #[clap(short, long, default_value_t=8)]
    resolution: u64,
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

    let output = args.text.chars()
        .zip(CharacterPosition::squares(args.size as i16))
        .map(|(ch, pos)| Ok(PositionedCharacter::new(ch, &font_face, &pos)
            .ok_or(GeneratorError::NoCharacter(ch))?
            .to_triangles(args.resolution)?))
        .map(|triangles:Result<Vec<Triangles>, GeneratorError>| Ok(triangles?.iter()
            .map(|t| t.svg(false))
            .intersperse(String::from("\n"))
            .collect::<String>()))
        .collect::<Result<Vec<String>, GeneratorError>>()?
        .join("\n");

    let output = format!("<svg>\n{}\n</svg>", output);
    if let Some(output_file) = &args.output {
        fs::write(output_file, output.as_bytes())
            .map_err(|e| GeneratorError::WriteError(e))?;
    } else {
        println!("{}", output);
    }

    Ok(())
}