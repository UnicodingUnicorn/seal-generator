use clap::Clap;
use regex::Regex;
use serde::{ Deserialize };

lazy_static! {
        static ref URLS_RE:Regex = Regex::new("\"url\":\\s?\"(?P<url>http[s]?://(?:[a-zA-Z]|[0-9]|[$-_@.&+]|[!*\\(\\),]|(?:%[0-9a-fA-F][0-9a-fA-F]))+)\".*\"descriptionurl\":\\s?\"(?P<description_url>http[s]?://(?:[a-zA-Z]|[0-9]|[$-_@.&+]|[!*\\(\\),]|(?:%[0-9a-fA-F][0-9a-fA-F]))+)\"").unwrap();
}

#[derive(Debug, Clone, Deserialize)]
pub struct CategoryPagination {
    pub cmcontinue: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CategoryMember {
    pub title: String,
    pub pageid: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CategoryMembers {
    pub categorymembers: Vec<CategoryMember>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CategoryList {
    #[serde(rename="continue")]
    pub pagination: Option<CategoryPagination>,   
    pub query: CategoryMembers,
}

#[derive(Debug, Clone, Deserialize)]
pub struct URLs {
    pub url: String,
    pub description_url: String,
}
impl URLs {
    pub fn from_raw(raw:&str) -> Option<Self> {
        let captures = URLS_RE.captures(raw)?;

        Some(Self {
            url: captures["url"].to_string(),
            description_url: captures["description_url"].to_string(),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Item {
    pub download_url: String,
    pub character: char,
}
impl Item {
    pub fn new(download_url: String, character: char) -> Self {
        Self {
            download_url,
            character,
        }
    }
}

#[derive(Clap)]
pub struct Opts {
    #[clap(short, long, default_value = "https://commons.wikimedia.org/w/api.php")]
    pub url: String,
    #[clap(short, long, default_value = "./output")]
    pub output_directory: String,
    #[clap(short, long, default_value = "500")]
    pub limit: u64,
    #[clap(short, long)]
    pub verbose: bool,
    #[clap(short, long, default_value = "Category:Shuowen_seal_script_characters")]
    pub category: String,
}

