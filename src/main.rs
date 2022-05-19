use std::fs;
use ttf_parser::{ Face };

mod character;
use character::{ Character, CharacterPosition };
mod svgbuilder;

fn main() {
    let font_data = fs::read("./ebas927.ttf").unwrap();
    let font_face = Face::from_slice(&font_data, 0).unwrap();

    let test = "林海之印";
    let squares = CharacterPosition::squares(400);
    let output = test.chars()
        .zip(squares)
        .map(|(ch, pos)| {
            let character = Character::new(ch, &font_face)?;
            Some(character.svg(&pos))
        })
        .collect::<Option<Vec<String>>>()
        .unwrap()
        .join("\n");

    println!("<svg>\n{}\n</svg>", output);
}
