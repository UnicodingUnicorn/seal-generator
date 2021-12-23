#![feature(path_file_prefix)]

mod characters;
use crate::characters::{ build_cache };
mod kanji;
use kanji::{ parse };
mod xml;

use std::time::Instant;

fn main() {
    // let kanji_dir = "../kanjivg/kanji";
    let start = Instant::now();
    let cache = build_cache("../characters").unwrap();
    println!("{}ms", start.elapsed().as_millis());
    if let Err(e) = parse("../kanjivg/kanji/07503.svg", &cache) {
        println!("{}", e);
    }
}
