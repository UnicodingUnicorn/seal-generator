use clap::Parser;
use std::fs;
use thiserror::Error;
use ttf_parser::{ Face };

mod character;
use character::{ Character, CharacterPosition };
mod svgbuilder;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about=None)]
struct Config {
    #[clap(value_parser)]
    text: String,
    #[clap(short, long, default_value_t=15)]
    size: u16,
    #[clap(short, long)]
    output: Option<String>,
}

#[derive(Debug, Error)]
enum GeneratorError {
    #[error("character {0} cannot be found")]
    NoCharacter(char),
    #[error("the font file could not be found")]
    NoFont,
    #[error("{0}")]
    FontError(#[from] ttf_parser::FaceParsingError),
    #[error("could not write output: {0}")]
    WriteError(std::io::Error),
}

fn main() {
    let args = Config::parse();
    if let Err(e) = run(&args) {
        println!("{}", e);
        std::process::exit(1);
    }
}

fn run(args:&Config) -> Result<(), GeneratorError> {
    let font_data = fs::read("./ebas927.ttf")
        .map_err(|_| GeneratorError::NoFont)?;
    let font_face = Face::from_slice(&font_data, 0)?;

    let output = args.text.chars()
        .zip(CharacterPosition::squares(args.size as i16))
        .map(|(ch, pos)| Ok(Character::new(ch, &font_face)
            .ok_or(GeneratorError::NoCharacter(ch))?
            .positioned(&pos)
            .svg()))
        .collect::<Result<Vec<String>, GeneratorError>>()?
        .join("\n");

    let output = format!("<svg>\n<g transform=\"scale(-1, 1)\">{}\n</g></svg>", output);
    if let Some(output_file) = &args.output {
        fs::write(output_file, output.as_bytes())
            .map_err(|e| GeneratorError::WriteError(e))?;
    } else {
        println!("{}", output);
    }

    Ok(())
}
