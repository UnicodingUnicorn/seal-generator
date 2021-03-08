#![feature(assoc_char_funcs)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate thiserror;

mod client;
mod downloader;
mod types;

use clap::Clap;
use downloader::Downloader;
use types::Opts;

#[tokio::main]
async fn main() {
    let opts:Opts = Opts::parse();
    let downloader = match Downloader::new(&opts).await {
        Ok(downloader) => downloader,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        },
    };

    if let Err(e) = downloader.download(&opts.category).await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
