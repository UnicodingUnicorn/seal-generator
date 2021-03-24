use crate::ChargenError;
use crate::svg_data::SVGData;
use serde::{ Deserialize, Serialize };
use std::collections::HashMap;
use std::fs::{ self, DirEntry };
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Characters {
    characters: HashMap<char, SVGData>,
}
impl Characters {
    pub fn from_dir(characters_dir:&str) -> Result<Self, ChargenError> {
        let characters_dir = Path::new(characters_dir);

        let chars = fs::read_dir(characters_dir)?
            .collect::<std::io::Result<Vec<DirEntry>>>()?
            .iter()
            .filter(|e| !e.path().is_dir())
            .filter_map(|e| {
                let file_name = e.file_name().into_string().ok()?;
                let ch = file_name.chars().next()?;

                Some((ch, file_name))
            })
            .map(|(ch, file_name)| {
                let svg_data = SVGData::from_file(characters_dir.join(Path::new(&file_name)))?;
                Ok((ch, svg_data))
            })
            .collect::<Result<Vec<(char, SVGData)>, usvg::Error>>()?;

        let mut characters = HashMap::new();
        for (ch, svg) in chars {
            characters.insert(ch, svg);
        }

        Ok(Self {
            characters,
        })
    }

    pub fn get(&self, ch:char) -> Option<&SVGData> {
        self.characters.get(&ch)
    }

    pub fn has(&self, ch:char) -> bool {
        self.characters.contains_key(&ch)
    }

    pub fn except<T>(&self, other:&HashMap<char, T>) -> Vec<char> {
        self.characters.keys()
            .filter(|ch| !other.contains_key(ch))
            .map(|ch| *ch)
            .collect::<Vec<char>>()
    }
}
