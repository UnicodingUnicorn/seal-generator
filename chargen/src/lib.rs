#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate thiserror;

mod expansions;
mod utils;
use expansions::Expansions;

use regex::Regex;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::{ self, DirEntry };
use std::path::Path;
use std::sync::Arc;

lazy_static! {
    static ref LINE_RE:Regex = Regex::new(r"(?m)^U\+[0-9A-F]{4}\t(?P<ch>[^\t])\t(?P<mappings>.+)$").unwrap();
    static ref MAP_RE:Regex = Regex::new(r"(?P<mapping>[^\t\[\]A-Z]+)\[(?P<type>[A-Z]+)\]").unwrap();
    static ref SAVE_RE:Regex = Regex::new(r"(?m)(?P<ch>[^\t])\t(?P<available>true|false)\t(?P<mapping>[^\t]+)").unwrap();
}

#[derive(Debug, Error)]
pub enum ChargenError {
    #[error("error occured reading fs: {0}")]
    IO(#[from] std::io::Error)
}

pub struct CharacterGenerator {
    mappings: HashMap<char, (bool, String)>,
}
impl CharacterGenerator {
    pub fn new(savefile:&str, characters_dir:&str, ids_file:&str) -> Result<Self, ChargenError> {
        Ok(match Self::from_save(savefile) {
            Ok(chargen) => chargen,
            Err(_) => {
                let chargen = Self::from_files(characters_dir, ids_file)?;
                chargen.save(savefile)?;
                chargen      
            },
        })
    }

    pub fn from_save(savefile:&str) -> Result<Self, ChargenError> {
        let raw_save = utils::read_file(savefile)?;
        
        let mut mappings:HashMap<char, (bool, String)> = HashMap::new();
        for line in SAVE_RE.captures_iter(&raw_save) {
            let ch = line["ch"].chars().next().unwrap();
            let available = match &line["available"] {
                "true" => true,
                _ => false,
            };
            let mapping = line["mapping"].to_string();

            mappings.insert(ch, (available, mapping));
        }

        Ok(Self {
            mappings,
        })
    }

    pub fn from_files(characters_dir:&str, ids_file:&str) -> Result<Self, ChargenError> {
        let available_characters = fs::read_dir(Path::new(characters_dir))?
            .collect::<std::io::Result<Vec<DirEntry>>>()?
            .iter()
            .filter(|e| !e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok()?.chars().next())
            .collect::<Vec<char>>();

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

        let unmapped_chars = available_characters.iter().filter(|ch| !mappings.contains_key(ch)).collect::<Vec<&char>>();

        for ch in unmapped_chars {
            mappings.insert(*ch, (true, String::from(*ch)));
        }

        Ok(Self {
            mappings,
        })
    }

    fn assess_mappings(mappings:&[String], available_characters:&[char]) -> (bool, String) {
        let mut has_available = mappings.iter()
            .filter(|m| m.chars().all(|ch| available_characters.iter().any(|ach| *ach == ch)))
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

    fn expand(ch:char, available_characters:&[char], raw_mappings:&[(char, Arc<String>)], mappings:&mut HashMap<char, Arc<Vec<String>>>) -> Arc<Vec<String>> {
        if available_characters.iter().any(|ach| *ach == ch) {
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
                    if available_characters.iter().any(|ach| *ach == ch) {
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
}
