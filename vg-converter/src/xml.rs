use crate::kanji::KanjiError;
// use crate::tree::Node;

use quick_xml::Reader;
use quick_xml::events::{ BytesStart, Event };
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub type RawKanjiMap = HashMap<char, (Option<char>, String)>;

pub fn parse_raw_map(filename:&str) -> Result<(Option<char>, RawKanjiMap), KanjiError> {
    let file = File::open(Path::new(filename))?;
    let buf = BufReader::new(file);

    let mut reader = Reader::from_reader(buf);
    reader.trim_text(true);
    let mut buf = Vec::new();

    let mut top_char = None;

    let mut tmp = HashMap::new();
    let mut stack = Vec::new();
    let mut elems_stack = Vec::new();

    loop {
        match reader.read_event(&mut buf)? {
            Event::Start(ref e) => match get_element(b"kvg:element", e)? {
                Some(ch) => {
                    if top_char == None {
                        top_char = Some(ch);
                    }

                    if elems_stack.len() > 0 {
                        let parent = match elems_stack.len() > 1 {
                            true => Some(elems_stack[elems_stack.len() - 2]),
                            false => None,
                        };

                        add_to_entry(parent, elems_stack[elems_stack.len() - 1], &[], &mut tmp);
                    }

                    stack.push(ch);
                    elems_stack.push(ch);
                },
                None => {
                    stack.push('\0');
                    if elems_stack.len() > 0 {
                        let parent = match elems_stack.len() > 1 {
                            true => Some(elems_stack[elems_stack.len() - 2]),
                            false => None,
                        };

                        add_to_entry(parent, elems_stack[elems_stack.len() - 1], &filter_kvg(&entag(e))?, &mut tmp);
                    }
                },
            },
            Event::Empty(ref e) => {
                if elems_stack.len() > 0 {
                    let parent = match elems_stack.len() > 1 {
                        true => Some(elems_stack[elems_stack.len() - 2]),
                        false => None,
                    };

                    add_to_entry(parent, elems_stack[elems_stack.len() - 1], &filter_kvg(&enslashtag(e))?, &mut tmp);
                }
            },
            Event::Text(ref e) => {
                if elems_stack.len() > 0 {
                    let parent = match elems_stack.len() > 1 {
                        true => Some(elems_stack[elems_stack.len() - 2]),
                        false => None,
                    };

                    add_to_entry(parent, elems_stack[elems_stack.len() - 1], e, &mut tmp);
                }
            },
            Event::End(ref e) => {
                let ch = stack.pop().unwrap();
                if ch != '\0' {
                    elems_stack.pop();
                } else {
                    if elems_stack.len() > 0 {
                        let parent = match elems_stack.len() > 1 {
                            true => Some(elems_stack[elems_stack.len() - 2]),
                            false => None,
                        };

                        add_to_entry(parent, elems_stack[elems_stack.len() - 1], &entag(e), &mut tmp);
                    }
                }
            },
            Event::Eof => break,
            _ => (),
        };
    }

    let res = tmp.iter()
        .map(|(k, (p, v))| Ok((*k, (*p, String::from_utf8(v.to_vec())?))))
        .collect::<Result<RawKanjiMap, std::string::FromUtf8Error>>()?;

    Ok((top_char, res))
}

fn get_element(name:&[u8], e:&BytesStart) -> Result<Option<char>, KanjiError> {
    Ok(e.attributes()
        .find(|a| if let Ok(a) = a { a.key == name } else { false })
        .transpose()?
        .map(|ch| String::from_utf8(ch.unescaped_value()?.to_vec()).map_err(|e| KanjiError::Encoding(e)))
        .transpose()?
        .map(|ch| ch.chars().next())
        .flatten())
}

fn add_to_entry(parent: Option<char>, key:char, value:&[u8], map:&mut HashMap<char, (Option<char>, Vec<u8>)>) {
    match map.get_mut(&key) {
        Some((_, ref mut v)) => {
            v.extend(value.iter());
        },
        None => {
            map.insert(key, (parent, value.to_vec()));
        },
    }
}

fn entag(value:&[u8]) -> Vec<u8> {
    let mut res = vec![60]; // '<'
    res.extend(value.iter());
    res.push(62); // '>'
    res.push(10); // '\n'

    res
}

fn enslashtag(value:&[u8]) -> Vec<u8> {
    let mut res = vec![60]; // '<'
    res.extend(value.iter());
    res.push(47); // '/'
    res.push(62); // '>'
    res.push(10); // '\n'

    res
}

fn filter_kvg(value:&[u8]) -> Result<Vec<u8>, std::string::FromUtf8Error> {
    Ok(String::from_utf8(value.to_vec())?
        .split(" ")
        .filter(|s| !s.starts_with("kvg"))
        .collect::<Vec<&str>>()
        .join(" ")
        .as_bytes()
        .to_vec())
}
