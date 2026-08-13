use error::error::Error;
use std::{fs::File, io::BufReader};

pub fn read_json<'a, T: serde::de::DeserializeOwned>(path: &str) -> Result<T, Error> {
    let file = File::open(path)?;

    let reader = BufReader::new(file);

    let data: T = serde_json::from_reader(reader)?;

    Ok(data)
}

#[cfg(test)]
mod tests {
    use crate::read_json;

    /// Typed parsing belongs to `core::boss`; this only covers reading the file.
    #[test]
    fn read_test() {
        let data: serde_json::Value = read_json(r"./tests/test.json").unwrap();

        assert_eq!(data["id"], 65535);
        assert_eq!(data["name"]["ko"], "시험용");
        assert_eq!(data["heavy"]["normal"]["hp"], 300000);
        assert_eq!(data["heavy"]["normal"]["armor_type"], "heavy");
        assert_eq!(
            data["heavy"]["normal"]["phase_switching_hp"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
    }
}
