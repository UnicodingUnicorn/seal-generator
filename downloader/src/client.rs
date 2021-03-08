use crate::types::{ CategoryList, URLs };
use regex::Regex;
use reqwest::Client;

lazy_static! {
    static ref SEAL_CODE_RE:Regex = Regex::new(r"&#(?P<code>[0-9]+);-seal\.svg").unwrap();
    static ref SEAL_CHAR_RE:Regex = Regex::new(r"(?P<ch>[\u4E00-\u62FF\u6300-\u77FF\u7800-\u8CFF\u8D00-\u9FFF])-seal\.svg").unwrap();
}

pub struct APIClient {
    base_url: String,
    client: Client,
    cmlimit: u64,
}
impl APIClient {
    pub fn new(base_url:&str, cmlimit:u64) -> Self {
        let base_url = base_url.to_string();
        let client = Client::new();

        Self {
            base_url,
            client,
            cmlimit,
        }
    }

    fn with_defaults(&self, custom_queries:&str) -> String {
        format!("{}?action=query&format=json&{}", self.base_url, custom_queries)
    }

    pub async fn list_items(&self, category_name:&str, page:&Option<String>) -> reqwest::Result<CategoryList> {
        let url = self.with_defaults(&format!("list=categorymembers&cmlimit={}&cmtitle={}", self.cmlimit, category_name));
        let url = match page {
            Some(page) => format!("{}&cmcontinue={}", url, page),
            None => url,
        };

        Ok(self.client.get(&url).send().await?.json::<CategoryList>().await?)
    }

    pub async fn get_urls(&self, filename:&str) -> reqwest::Result<Option<URLs>> {
        let url = self.with_defaults(&format!("prop=imageinfo&iiprop=url&titles={}", filename));

        let res = self.client.get(&url).send().await?.text().await?;
        let urls = URLs::from_raw(&res);

        Ok(urls)
    }

    pub async fn get_character(&self, description_url:&str) -> reqwest::Result<Option<char>>{
        let res = self.client.get(description_url).send().await?.text().await?;

        let ch = match SEAL_CODE_RE.captures(&res) {
            Some(captures) => char::from_u32(captures["code"].parse::<u32>().unwrap()),
            None => match SEAL_CHAR_RE.captures(&res) {
                Some(captures) => captures["ch"].chars().next(),
                None => None,
            },
        };

        Ok(ch)
    }

    pub fn get_client(&self) -> &Client {
        &self.client
    }
}

