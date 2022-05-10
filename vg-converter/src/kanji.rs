use crate::characters::CacheEntry;
use crate::xml::{ parse_raw_map };

use std::collections::HashMap;
use thiserror::Error;
use usvg::{ NodeExt, PathBbox, Tree };

#[derive(Debug, Error)]
pub enum KanjiError {
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
    #[error("file has no name")]
    Name,
    #[error("xml parsing error: {0}")]
    XML(#[from] quick_xml::Error),
    #[error("improper encoding: {0}")]
    Encoding(#[from] std::string::FromUtf8Error),
    #[error("no character defined")]
    NoCharacter,
    // #[error("too many top-level characters")]
    // TooManyCharacters,
    #[error("svg error: {0}")]
    SVG(#[from] usvg::Error),
    #[error("the hashmap's characters have become mismatched")]
    CharacterMismatch,
    #[error("unable to calculate bounding box")]
    BBox,
}

pub fn parse(filepath:&str, cache:&HashMap<char, CacheEntry>) -> Result<String, KanjiError> {
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

    let options = usvg::Options::default();
    let (bbox, parts) = process_characters(ch, &map, &children, cache, &options)?;
    let target_bbox = PathBbox::new(0.0, 0.0, 300.0, 300.0).unwrap();
    let matrix = get_transformation_matrix(&bbox, &target_bbox);

    Ok(format!("<svg xmlns=\"http://www.w3.org/2000/svg\">\n\t<g transform=\"{}\">{}\n\t</g>\n</svg>", matrix, parts.join("\n")))
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

fn process_characters(ch:char, map:&HashMap<char, (Option<char>, String)>, children:&HashMap<char, Vec<char>>, cache:&HashMap<char, CacheEntry>, options:&usvg::Options) -> Result<(PathBbox, Vec<String>), KanjiError> {
    if let Some(seal) = cache.get(&ch) {
        let (_, mut s) = map.get(&ch).ok_or(KanjiError::CharacterMismatch)?.clone();
        accumulate_children(ch, &mut s, map, children);

        let original = Tree::from_str(&format!("<svg xmlns=\"http://www.w3.org/2000/svg\">{}</svg>", s), &options.to_ref())?;
        let original_bbox = original.root().calculate_bbox().ok_or(KanjiError::BBox)?;
        // println!("{}", original_bbox);

        let matrix = get_transformation_matrix(&seal.bbox, &original_bbox);
        let converted = format!("<g transform=\"{}\">\n{}\n</g>", matrix, seal.svg);
        let converted_tree = Tree::from_str(&format!("<svg xmlns=\"http://www.w3.org/2000/svg\">{}</svg>", converted), &options.to_ref())?;
        let converted_bbox = converted_tree.root().calculate_bbox().ok_or(KanjiError::BBox)?;

        Ok((converted_bbox, vec![converted]))
    } else {
        let mut res = Vec::new();
        let mut bbox = PathBbox::new_bbox();

        if let Some((_, s)) = map.get(&ch) {
            res.push(s.clone());

            let parent = Tree::from_str(&format!("<svg xmlns=\"http://www.w3.org/2000/svg\">{}</svg>", s), &options.to_ref())?;
            let parent_bbox = parent.root().calculate_bbox().ok_or(KanjiError::BBox)?;

            bbox = bbox.expand(parent_bbox);
        }

        if let Some(ch_children) = children.get(&ch) {
            for &child in ch_children {
                let (child_bbox, mut result) = process_characters(child, map, children, cache, options)?;
                res.append(&mut result);

                bbox = bbox.expand(child_bbox);
            }
        }

        Ok((bbox, res))
    }
}

fn accumulate_children(ch:char, acc:&mut String, map:&HashMap<char, (Option<char>, String)>, children:&HashMap<char, Vec<char>>) {
    if let Some(ch_children) = children.get(&ch) {
        for child in ch_children {
            accumulate_children(*child, acc, map, children);
        }
    }

    if let Some((_, s)) = map.get(&ch) {
        acc.push_str(s);
    }
}

fn get_transformation_matrix(from:&PathBbox, to:&PathBbox) -> String {
    let dx = to.x() - from.x();
    let dy = to.y() - from.y();
    let aw = to.width() / from.width();
    let ah = to.height() / from.height();

    format!("matrix({} 0 0 {} {} {})", aw, ah, dx, dy)
}
