use std::collections::HashMap;
use std::fs::{ self, File };
use std::io::{ BufReader, Read };
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
    #[error("file has no name")]
    Name,
}

pub fn build_cache(characters_dir:&str) -> Result<HashMap<char, String>, CacheError> {
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
                None => return Err(CacheError::Name),
            };

            let _ = cache.insert(key, contents);
        }
    }

    Ok(cache)
}
