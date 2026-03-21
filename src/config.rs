use crate::Error;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub repository_path: PathBuf,
    pub passphrase_path: PathBuf,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        let passphrase_path = ".passphrase";
        Self {
            repository_path: PathBuf::from("."),
            passphrase_path: PathBuf::from(passphrase_path),
            include_patterns: vec!["*".to_string()],
            exclude_patterns: vec![".git*".to_string(), passphrase_path.to_string()],
        }
    }
}

impl Config {
    /// Reads from the given config file path if it exists, or returns None if it doesn't.
    /// Errors if the file is invalid.
    pub fn read(path: &Path) -> Result<Option<Config>, Error> {
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(Some(config))
    }

    /// Writes the given config to the given path.
    /// Errors if the file cannot be written or if the config is invalid.
    pub fn write(config: &Config, path: &Path) -> Result<(), Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let config = toml::to_string_pretty(config)?;
        fs::write(path, config)?;
        Ok(())
    }

    pub fn resolve_paths(self, base: &Path) -> Self {
        Self {
            repository_path: Self::resolve_path(base, &self.repository_path),
            passphrase_path: Self::resolve_path(base, &self.passphrase_path),
            ..self
        }
    }

    pub fn make_path_relative(base_path: &Path, abs_path: &Path) -> Result<PathBuf, Error> {
        let rel_path = abs_path
            .strip_prefix(base_path)
            .map_err(|_| "abs_path is not inside base_path")?;

        Ok(rel_path.to_path_buf())
    }

    fn resolve_path(base: &Path, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            base.join(path)
        }
    }
}

pub fn read_passphrase(path: &Path) -> Result<String, Error> {
    Ok(fs::read_to_string(path)?.trim().to_string())
}

pub fn write_passphrase(passphrase: &str, path: &Path) -> Result<(), Error> {
    let mut file = fs::File::create(path)?;
    file.write_all(passphrase.as_bytes())?;
    Ok(())
}
