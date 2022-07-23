use std::fs;
use thiserror::Error;
use ttf_parser::{ Face };

mod character;
use character::{ Character, CharacterPosition };
mod svgbuilder;

#[derive(Debug, Error)]
enum GeneratorError {
    #[error("character {0} cannot be found")]
    NoCharacter(char),
    #[error("the font file could not be found")]
    NoFont,
    #[error("{0}")]
    FontError(#[from] ttf_parser::FaceParsingError),
}

fn main() {
    let test = "林海之印";

    run(test).unwrap();
}

fn run(s:&str) -> Result<String, GeneratorError> {
    let font_data = fs::read("./ebas927.ttf")
        .map_err(|_| GeneratorError::NoFont)?;
    let font_face = Face::from_slice(&font_data, 0)?;

    let squares = CharacterPosition::squares(15);
    let output = s.chars()
        .zip(squares)
        .map(|(ch, pos)| Ok(Character::new(ch, &font_face)
            .ok_or(GeneratorError::NoCharacter(ch))?
            .positioned(&pos)
            .svg()))
        .collect::<Result<Vec<String>, GeneratorError>>()?
        .join("\n");

    Ok(format!("<svg>\n<g transform=\"scale(-1, 1)\">{}\n</g></svg>", output))
}
