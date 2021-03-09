use std::fmt::{ self, Debug, Formatter };

pub struct Mapping {
    character: char,
    mapping: String,
}
impl Mapping {
    pub fn new(character:&str, mapping:&str) -> Option<Self> {
        let character = character.chars().next()?;

        Some(Self {
            character,
            mapping: mapping.to_string(),
        })
    }

    pub fn new_char(character:char, mapping:&str) -> Self {
        Self {
            character,
            mapping: mapping.to_string(),
        }
    }

    pub fn character(&self) -> char {
        self.character
    }

    pub fn mapping_len(&self) -> usize{
        self.mapping.chars().count()
    }

    pub fn mapping(&self) -> &str {
        &self.mapping
    }
}
impl Debug for Mapping {
    fn fmt(&self, formatter:&mut Formatter) -> fmt::Result {
        write!(formatter, "{} ➞ {}", self.character, self.mapping)
    }
}
