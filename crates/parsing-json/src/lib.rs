use error::error::Error;
use serde_json;
use std::{fs::File, io::BufReader};

pub fn parse_stats<'a, T: serde::de::DeserializeOwned>(path: &str) -> Result<T, Error> {
    let file = File::open(path)?;

    let reader = BufReader::new(file);

    let data: T = serde_json::from_reader(reader)?;

    Ok(data)
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
