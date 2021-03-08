use async_std::fs::{ self, File };
use async_std::prelude::*;
use async_std::path::Path;
use crate::client::APIClient;
use crate::types::{ Item, Opts };
use pbr::ProgressBar;
use std::sync::Arc;
use regex::Regex;

lazy_static! {
    static ref SEAL_CHAR_RE:Regex = Regex::new(r"(?P<ch>[\u4E00-\u62FF\u6300-\u77FF\u7800-\u8CFF\u8D00-\u9FFF\u3400-\u4DBF\u{20000}-\u{215FF}\u{21600}-\u{230FF}\u{23100}-\u{245FF}\u{24600}-\u{260FF}\u{26100}-\u{275FF}\u{27600}-\u{290FF}\u{29100}-\u{2A6DF}\u{2A700}-\u{2B73F}\u{2B740}–\u{2B81F}\u{2B820}–\u{2CEAF}\u{2CEB0}–\u{2EBEF}\u{30000}–\u{3134F}])-seal\.svg").unwrap();
}

#[derive(Debug, Error)]
pub enum DownloaderError {
    #[error("error performing file io: {0}")]
    IO(#[from] async_std::io::Error),
    #[error("error performing request: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("cannot find download URLs for {0}")]
    MissingURLs(String),
    #[error("cannot find character for {0} in description")]
    MissingCharacter(String),
}

pub struct Downloader {
    client: Arc<APIClient>,
    output_dir: String,
    verbose: bool,
}
impl Downloader {
    pub async fn new(opts:&Opts) -> Result<Self, DownloaderError> {
        let client = Arc::new(APIClient::new(&opts.url, opts.limit));

        let output_path = Path::new(&opts.output_directory);
        if !output_path.exists().await {
            fs::create_dir_all(output_path).await?;
        }

        Ok(Self {
            client,
            output_dir: opts.output_directory.clone(),
            verbose: opts.verbose,
        })
    }

    pub async fn download(&self, category:&str) -> Result<(), DownloaderError> {
        let mut next = None;

        if self.verbose {
            println!("retrieving category \"{}\"", category);
        }

        let mut i = 1;
        while let Some(n) = self.download_page(category, &next, i).await? {
            next = Some(n);
            i += 1;
        }

        println!("done");

        Ok(())
    }

    async fn download_page(&self, category:&str, page:&Option<String>, i:u64) -> Result<Option<String>, DownloaderError> {
        if self.verbose {
            println!("retrieving page {}", i);
        }

        let list = self.client.list_items(category, page).await?;
        let next_page = match &list.pagination {
            Some(cp) => Some(cp.cmcontinue.clone()),
            None => None,
        };

        let mut pb = match self.verbose {
            true => Some(ProgressBar::new(list.query.categorymembers.len() as u64)),
            false => None,
        };

        for cm in list.query.categorymembers {
            if cm.title.ends_with(".svg") {
                match self.get_information(&cm.title).await {
                    Ok(item) => self.download_character(&item).await?,
                    Err(e) => match e {
                        DownloaderError::MissingCharacter(_) => {
                            println!("warning: {}", e);
                        },
                        _ => return Err(e),
                    },
                };

                if let Some(ref mut pb) = pb {
                    pb.inc();
                }
            }
        }

        if let Some(ref mut pb) = pb {
            pb.finish_print("done");
        }

        Ok(next_page)
    }

    async fn get_information(&self, filename:&str) -> Result<Item, DownloaderError>  {
        let urls = match self.client.get_urls(filename).await? {
            Some(urls) => urls,
            None => return Err(DownloaderError::MissingURLs(filename.to_string())),
        };

        let ch = match SEAL_CHAR_RE.captures(filename) {
            Some(captures) => captures["ch"].chars().next().unwrap(),
            None => match self.client.get_character(&urls.description_url).await? {
                Some(ch) => ch,
                None => return Err(DownloaderError::MissingCharacter(filename.to_string())),
            },
        };
        
        Ok(Item::new(urls.url, ch))
    }

    async fn download_character(&self, info:&Item) -> Result<(), DownloaderError> {
        let mut res = self.client.get_client().get(&info.download_url).send().await?;
        let mut file = File::create(Path::new(&self.output_dir).join(&format!("{}.svg", info.character))).await?;
        while let Some(chunk) = res.chunk().await? {
            file.write_all(&chunk).await?;
        }

        Ok(())
    }
}
