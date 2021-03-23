use crate::ChargenError;
use serde::{ Deserialize, Serialize };
use std::collections::HashMap;
use std::fs::{ self, DirEntry };
use std::path::Path;
use usvg::{ self, Options, Tree, XmlOptions };

#[derive(Debug, Deserialize, Serialize)]
pub struct Characters {
    characters: HashMap<char, String>,
}
impl Characters {
    pub fn from_dir(characters_dir:&str) -> Result<Self, ChargenError> {
        let chars = fs::read_dir(Path::new(characters_dir))?
            .collect::<std::io::Result<Vec<DirEntry>>>()?
            .iter()
            .filter(|e| !e.path().is_dir())
            .filter_map(|e| {
                let file_name = e.file_name().into_string().ok()?;
                let ch = file_name.chars().next()?;

                Some((ch, file_name))
            })
            .map(|(ch, file_name)| {
                let svg_data = Tree::from_file(&file_name, &Options::default())?.to_string(XmlOptions::default());
                Ok((ch, svg_data))
            })
            .collect::<Result<Vec<(char, String)>, usvg::Error>>()?;

        let mut characters = HashMap::new();
        for (ch, svg) in chars {
            characters.insert(ch, svg);
        }

        Ok(Self {
            characters,
        })
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
