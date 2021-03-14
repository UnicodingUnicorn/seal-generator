use std::fs::File;
use std::io::{ Read, Write };
use std::path::Path;

pub fn is_ids_char(ch:char) -> bool {
    12272 <= (ch as u32) && (ch as u32) <= 12287
}

pub fn read_file(filename:&str) -> Result<String, std::io::Error> {
    let mut file = File::open(Path::new(filename))?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;

    Ok(s)
}

pub fn write_file(filename:&str, contents:&str) -> Result<(), std::io::Error> {
    let mut file = File::create(Path::new(filename))?;
    file.write_all(contents.as_bytes())
}

pub fn strlen(s:&str) -> usize {
    s.chars().count()
}
