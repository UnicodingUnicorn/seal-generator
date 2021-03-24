#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate thiserror;

mod characters;
mod expansions;
mod idc;
pub mod svg_data;
mod utils;

use characters::Characters;
use expansions::Expansions;
use idc::{ IDC, IDCer };

use regex::Regex;
use serde::{ Deserialize, Serialize };
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

lazy_static! {
    static ref LINE_RE:Regex = Regex::new(r"(?m)^U\+[0-9A-F]{4}\t(?P<ch>[^\t])\t(?P<mappings>.+)$").unwrap();
    static ref MAP_RE:Regex = Regex::new(r"(?P<mapping>[^\t\[\]A-Z]+)\[(?P<type>[A-Z]+)\]").unwrap();
    static ref SAVE_RE:Regex = Regex::new(r"(?m)^(?P<ch>[^\s])\s+(?P<available>true|false)\s+(?P<mapping>[^\s]+)$").unwrap();
}

#[derive(Debug, Error)]
pub enum ChargenError {
    #[error("error occured reading fs: {0}")]
    IO(#[from] std::io::Error),
    #[error("error reading svg: {0}")]
    SVG(#[from] usvg::Error),
    #[error("svg data for the character {0} is not unavailable")]
    Unsupported(char),
    #[error("not enough characters supplied for the IDC sequence")]
    IDCNotEnough,
    #[error("ideographic description character {0} is not supported")]
    IDCUnsupported(char),
    #[error("the renderer was supplied with negative width and height")]
    InvalidWidthAndHeight,
    #[error("mapping does not reduce to one character")]
    IDCIncompleteReduction,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CharacterGenerator {
    pub mappings: HashMap<char, (bool, String)>,
    characters: Characters,
}
impl CharacterGenerator {
    pub fn from_files(characters_dir:&str, ids_file:&str) -> Result<Self, ChargenError> {
        let available_characters = Characters::from_dir(characters_dir)?;

        let raw_ids = utils::read_file(ids_file)?;
        let mut ids_mappings = LINE_RE.captures_iter(&raw_ids)
            .map(|line| MAP_RE.captures_iter(&line["mappings"])
                 .filter(|mapping| mapping["type"].chars().any(|ch| ch == 'G'))
                 .filter_map(|mapping| Some((line["ch"].chars().next()?, Arc::new(mapping["mapping"].to_string()))))
                .collect::<Vec<(char, Arc<String>)>>())
            .flatten()
            .collect::<Vec<(char, Arc<String>)>>();
 
        ids_mappings.sort_by(|(_, a), (_, b)| utils::strlen(a).cmp(&utils::strlen(b)));

        let mut expanded_mappings:HashMap<char, Arc<Vec<String>>> = HashMap::new();
        for (ch, _) in ids_mappings.iter() {
            let _ = Self::expand(*ch, &available_characters, &ids_mappings, &mut expanded_mappings);
        }

        let mut mappings:HashMap<char, (bool, String)> = HashMap::new();
        for (ch, m) in expanded_mappings.iter() {
            mappings.insert(*ch, Self::assess_mappings(m, &available_characters));
        }

        let unmapped_chars = available_characters.except(&mappings);

        for ch in unmapped_chars {
            mappings.insert(ch, (true, String::from(ch)));
        }

        Ok(Self {
            mappings,
            characters: available_characters,
        })
    }

    fn assess_mappings(mappings:&[String], available_characters:&Characters) -> (bool, String) {
        let mut has_available = mappings.iter()
            .filter(|m| m.chars().all(|ch| available_characters.has(ch) || utils::is_ids_char(ch)))
            .collect::<Vec<&String>>();

        has_available.sort_by(|a, b| a.chars().count().cmp(&b.chars().count()));

        if has_available.len() > 0 {
            return (true, has_available[0].to_string());
        }

        let (mapping, _) = mappings.iter()
            .fold(("", usize::MAX), |(acc, min), mapping| {
                if mapping.chars().count() < min {
                    (&mapping, mapping.chars().count())
                } else {
                    (acc, min)
                }
            });

        (false, mapping.to_string())
    }

    fn expand(ch:char, available_characters:&Characters, raw_mappings:&[(char, Arc<String>)], mappings:&mut HashMap<char, Arc<Vec<String>>>) -> Arc<Vec<String>> {
        if available_characters.has(ch) {
            return Arc::new(Expansions::from_character(ch));
        }

        if let Some(expansions) = mappings.get(&ch) {
            return expansions.clone();
        }

        let mut res = raw_mappings.iter()
            .filter(|(mch, _)| *mch == ch)
            .map(|(_, mapping)| {
                if **mapping == String::from(ch) {
                    return Expansions::from_character(ch);
                }

                let mut res:Vec<String> = Expansions::new_blank();
                for ch in mapping.chars() {
                    if available_characters.has(ch) {
                        res.push_char(ch);
                    } else if utils::is_ids_char(ch) {
                        res.push_char(ch);
                    } else if let Some(mappings) = mappings.get(&ch) {
                        res.merge_strings(mappings);
                    } else {
                        let expansions = Self::expand(ch, available_characters, raw_mappings, mappings);
                        res.merge_strings(&expansions);
                    }
                }

                res
            })
            .flatten()
            .collect::<Vec<String>>();

        if res.len() == 0 {
            res.push(String::from(ch));
        }

        let res = Arc::new(res);
        let _ = mappings.insert(ch, res.clone());

        res
    }

    pub fn get_mapping(&self, ch:char) -> Option<(bool, Cow<str>)> {
        self.mappings.get(&ch).map(|(available, mapping)| (*available, Cow::from(mapping)))
    }

    pub fn to_string(&self) -> String {
        let entries = self.mappings.iter()
            .map(|(ch, (available, mapping))| format!("{}\t{}\t{}", ch, available, mapping))
            .collect::<Vec<String>>();

        entries.join("\n")
    }

    pub fn save(&self, filename:&str) -> Result<(), ChargenError> {
        let contents = self.to_string();
        utils::write_file(filename, &contents)?;

        Ok(())
    }

    pub fn svg(&self, ch:char) -> Option<String> {
        let idc = IDC::new(300.0, 300.0, &self.characters).unwrap();
        let (available, mapping) = self.mappings.get(&ch)?;
        if !available {
            return None;
        }

        let characters = IDCer::new(&idc, &mapping)
            .map(|d| d.map(|d| d.svg().to_string()))
            .collect::<Result<Vec<String>, ChargenError>>()
            .map(|d| match d.len() > 0 {
                true => Some(d[0].clone()),
                false=> None,
            })
            .ok()
            .flatten()?;

            /*
        println!("{:#?}", characters);

        Some(self.characters.get(ch)?.svg())
        */
        Some(characters)
    }
}
