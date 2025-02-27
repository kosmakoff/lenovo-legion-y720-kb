use crate::error::Error;
use crate::error::Error::Problem;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

const CONFIG_DIR: &str = "/var/lib/lenovo-kb";
const CONFIG_PATH: &str = "/var/lib/lenovo-kb/state.toml";

#[derive(Serialize, Deserialize)]
pub struct LedConfig {
    pub is_on: bool,
}

impl LedConfig {
    pub fn load() -> Result<Self, Error> {
        if Path::new(CONFIG_PATH).exists() {
            let mut file = File::open(CONFIG_PATH)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            toml::from_str(&contents).or(Ok(Self::default()))
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    fn default() -> Self {
        Self { is_on: true }
    }

    pub fn save(&self) -> Result<(), Error> {
        let contents = toml::to_string(self)
            .or_else(|_| Err(Problem("Could not serialize config".to_owned())))?;
        fs::create_dir_all(CONFIG_DIR)?;
        let mut file = File::create(CONFIG_PATH)?;
        file.write_all(contents.as_bytes())
            .or_else(|_| Err(Problem("Could not write the config to file".to_owned())))?;

        Ok(())
    }
}
