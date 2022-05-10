use crate::kanji::KanjiError;

use quick_xml::{ Reader };
use quick_xml::events::{ Event };
use std::collections::HashMap;
use std::fs::{ self, File };
use std::io::{ BufReader, Read };
use std::path::Path;
use usvg::{ NodeExt, PathBbox, Tree };

pub struct CacheEntry {
    pub svg: String,
    pub bbox: PathBbox,
}

pub fn build_cache(characters_dir:&str) -> Result<HashMap<char, CacheEntry>, KanjiError> {
    let mut cache = HashMap::new();
    for path in fs::read_dir(Path::new(characters_dir))? {
        let path = path?;
        if path.file_type()?.is_file() {
            let mut reader = BufReader::new(File::open(path.path())?);
            let mut contents = String::new();
            reader.read_to_string(&mut contents)?;

            // There should always be a filename
            let key = match path.path().file_prefix().unwrap().to_string_lossy().to_string().chars().next() {
                Some(key) => key,
                None => return Err(KanjiError::Name),
            };

            let svg = get_paths(&contents)?;
            let options = usvg::Options::default();
            let tree = Tree::from_str(&format!("<svg xmlns=\"http://www.w3.org/2000/svg\">{}</svg>", svg), &options.to_ref())?;
            let bbox = tree.root().calculate_bbox().ok_or(KanjiError::BBox)?;

            let entry = CacheEntry {
                svg,
                bbox,
            };

            let _ = cache.insert(key, entry);
        }
    }

    Ok(cache)
}

fn get_paths(contents:&str) -> Result<String, KanjiError> {
    let mut reader = Reader::from_str(&contents);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut s = String::new();
    loop {
        match reader.read_event(&mut buf)? {
            Event::Empty(ref e) => {
                if e.name() == b"path" {
                    let d = match e.attributes()
                        .with_checks(false)
                        .find(|attr| match attr {
                            Ok(attr) => attr.key == b"d",
                            Err(_) => false,
                        }) {
                            Some(attr) => String::from_utf8(attr?.value.to_vec())?,
                            None => String::new(),
                        };

                    s.push_str(&format!("<path d=\"{}\"/>", d.replace("\n", "")));
                }
            },
            Event::Eof => break,
            _ => (),
        };
    }

    Ok(s)
}
