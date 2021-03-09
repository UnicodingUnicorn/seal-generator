use std::fs::File;
use std::io::Read;
use std::path::Path;

mod mapping;
mod mappings;
mod expansions;
mod utils;
use mappings::{ Mappings, MappingsMethods };

fn main() {
    // let ids = read_file("../cjkvi-ids/ids.txt").unwrap();
    let ids = read_file("test.txt").unwrap();

    let mappings = Mappings::from_ids(&ids);
    println!("{:#?}", mappings);
}

fn read_file(filename:&str) -> Result<String, std::io::Error> {
    let mut file = File::open(Path::new(filename))?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;

    Ok(s)
}
