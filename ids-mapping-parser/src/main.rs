use clap::Clap;

mod mapping;
mod mappings;
mod expansions;
mod utils;
use mappings::{ Mappings, MappingsMethods };

#[derive(Clap)]
struct Opts {
    #[clap(short, long)]
    input: String,
    #[clap(short, long, default_value = "./output.txt")]
    output: String,
}

fn main() {
    let opts:Opts = Opts::parse();

    let ids = match utils::read_file(&opts.input) {
        Ok(ids) => ids,
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        },
    };

    let mappings = Mappings::from_ids(&ids);
    if let Err(e) = utils::write_file(&opts.output, &mappings) {
        println!("{}", e);
        std::process::exit(1);
    }
}
