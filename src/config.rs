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
        Self {
            repository_path: PathBuf::from("."),
            passphrase_path: PathBuf::from(".passphrase"),
            include_patterns: vec!["*".to_string()],
            exclude_patterns: vec![],
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

    /// Returns the filename of the passphrase file.
    pub fn passphrase_filename(&self) -> Result<String, Error> {
        Self::filename_string(&self.passphrase_path)
    }

    /// Returns the filename of the given path.
    pub fn filename_string(path: &Path) -> Result<String, Error> {
        Ok(path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("Invalid path")?
            .to_string())
    }

    pub fn resolve_paths(self, base: &Path) -> Result<Self, Error> {
        Ok(Self {
            repository_path: Self::make_path_absolute(base, &self.repository_path)?,
            passphrase_path: Self::make_path_absolute(base, &self.passphrase_path)?,
            ..self
        })
    }

    pub fn make_path_relative(base_path: &Path, abs_path: &Path) -> Result<PathBuf, Error> {
        let rel_path = abs_path.strip_prefix(base_path)?;
        Ok(rel_path.to_path_buf())
    }

    fn make_path_absolute(base_path: &Path, rel_path: &Path) -> Result<PathBuf, Error> {
        let path = match rel_path.is_absolute() {
            true => rel_path.to_path_buf(),
            false => base_path.join(rel_path),
        };
        Ok(path.canonicalize()?)
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
