use crate::xml::{ parse_raw_map };

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KanjiError {
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
    #[error("xml parsing error: {0}")]
    XML(#[from] quick_xml::Error),
    #[error("improper encoding: {0}")]
    Encoding(#[from] std::string::FromUtf8Error),
    #[error("no character defined")]
    NoCharacter,
    // #[error("too many top-level characters")]
    // TooManyCharacters,
}

pub fn parse(filepath:&str, cache:&HashMap<char, String>) -> Result<(), KanjiError> {
    let (ch, map) = parse_raw_map(filepath)?;
    if map.len() <= 0 {
        return Err(KanjiError::NoCharacter);
    }

    let ch = ch.unwrap(); // Guaranteed to be something if HashMap has entries
    let children = build_children_map(&map);

    // println!("{}", ch);
    // println!("{:#?}", map);
    // println!("{:#?}", children);
    //
    // println!("{}", cache.contains_key(&ch));

    let res = process_characters(ch, &map, &children, cache);
    println!("{:#?}", res);

    Ok(())
}

fn build_children_map(map:&HashMap<char, (Option<char>, String)>) -> HashMap<char, Vec<char>> {
    let mut res = HashMap::new();
    for (ch, (p, _)) in map {
        if !res.contains_key(ch) {
            res.insert(*ch, Vec::new());
        }

        if let Some(p) = p {
            match res.get_mut(p) {
                Some(children) => children.push(*ch),
                None => {
                    res.insert(*p, vec![*ch]);
                },
            };
        }
    }

    res
}

fn process_characters(ch:char, map:&HashMap<char, (Option<char>, String)>, children:&HashMap<char, Vec<char>>, cache:&HashMap<char, String>) -> Vec<String> {
    if let Some(s) = cache.get(&ch) {
        println!("{}", ch);
        vec![s.clone()]
    } else {
        let mut res = Vec::new();
        if let Some((_, s)) = map.get(&ch) {
            res.push(s.clone());
        }

        if let Some(ch_children) = children.get(&ch) {
            for &child in ch_children {
                let mut result = process_characters(child, map, children, cache);
                res.append(&mut result);
            }
        }

        res
    }
}
