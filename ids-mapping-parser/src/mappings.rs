use crate::expansions::{ Expansions, ExpansionsMethods };
use crate::mapping::Mapping;
use crate::utils;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

pub type Mappings = Vec<Mapping>;
pub trait MappingsMethods {
    fn parse_ids(ids:&str) -> Self;
    fn from_ids(ids:&str) -> Self;
    fn expand(&self, character:char, mappings:&mut HashMap<char, Arc<Expansions>>) -> Arc<Expansions>;
}
impl MappingsMethods for Mappings {
    fn parse_ids(ids:&str) -> Self {
        let line_re = Regex::new(r"(?m)^U\+[0-9A-F]{4}\t(?P<ch>[^\t])\t(?P<mappings>.+)$").unwrap();
        let map_re = Regex::new(r"(?P<mapping>[^\t\[\]A-Z]+)").unwrap();

        line_re.captures_iter(&ids)
            .map(|line| {
                map_re.captures_iter(&line["mappings"])
                    .filter_map(|mapping| Mapping::new(&line["ch"], &mapping["mapping"]))
                    .collect::<Self>()
            })
            .flatten()
            .collect::<Self>()
    }

    fn from_ids(ids:&str) -> Self {
        let mut mappings_list = Self::parse_ids(ids);
        mappings_list.sort_by(|a, b| a.mapping_len().cmp(&b.mapping_len()));

        let mut raw_mappings:HashMap<char, Arc<Expansions>> = HashMap::new();
        for mapping in mappings_list.iter() {
            mappings_list.expand(mapping.character(), &mut raw_mappings);
        }

        let mappings = raw_mappings.iter()
            .map(|(ch, mappings)| {
                let (mapping, _) = mappings.iter().fold(("", 0), |(acc, max), mapping| {
                    if mapping.chars().count() > max {
                        (&mapping, mapping.chars().count())
                    } else {
                        (acc, max)
                    }
                });

                Mapping::new_char(*ch, mapping)
            })
            .collect::<Self>();

        mappings
    }

    fn expand(&self, character:char, mappings:&mut HashMap<char, Arc<Expansions>>) -> Arc<Expansions> {
        if let Some(expansions) = mappings.get(&character) {
            return expansions.clone();
        }

        let expansions = self.iter()
            .filter(|mapping| mapping.character() == character)
            .map(|mapping| mapping.mapping().to_string())
            .collect::<Vec<String>>();

        let mut res = expansions.iter()
            .map(|expansion| {
                if expansion == &String::from(character) {
                    return vec![String::from(character)];
                }

                let mut res = vec![String::new()];
                for ch in expansion.chars() {
                    if utils::is_ids_char(ch) {
                        res.push_char(ch);
                    } else if let Some(mappings) = mappings.get(&ch) {
                        res.merge_strings(mappings);
                    } else {
                        let expansions = self.expand(ch, mappings);
                        res.merge_strings(&expansions);
                    }
                }

                res
            })
            .flatten()
            .collect::<Expansions>();

        if res.len() == 0 {
            res.push(String::from(character));
        }

        let res = Arc::new(res);
        let _ = mappings.insert(character, res.clone());

        res
    }
}
