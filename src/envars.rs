use crate::error::ErrorReport;
use std::{collections::HashMap, fs, path::Path};

pub const CONFIG_FILE_DELIMITER: char = '=';

#[derive(Debug)]
pub struct AppConfig {
    values: HashMap<String, String>,
}

impl AppConfig {
    const COMMENT_START: char = '#';

    pub fn init(file_path: &Path, delimiter: char) -> Result<Self, ErrorReport> {
        Ok(AppConfig {
            values: fs::read_to_string(file_path)?
                .lines()
                .map(|line| line.trim())
                .filter(|line| !(line.is_empty() && line.starts_with(Self::COMMENT_START)))
                .filter_map(|line| {
                    if let Some((k, v)) = line.split_once(delimiter) {
                        Some((k.to_owned(), v.to_owned()))
                    } else {
                        None
                    }
                })
                .collect(),
        })
    }

    pub fn get_var(&self, name: &str) -> Option<&String> {
        self.values.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, CONFIG_FILE_DELIMITER};
    use std::path::Path;

    #[test]
    fn create() {
        let config_path = Path::new(env!("APP_CONFIG_FILE_PATH"));
        let res = AppConfig::init(config_path, CONFIG_FILE_DELIMITER).unwrap();
        assert_eq!(res.values.is_empty(), false, "Env vars map is empty");
    }
}
