use crate::Error;
use std::{fs, io::Write, path::Path};

pub fn read_passphrase(path: &Path) -> Result<String, Error> {
    Ok(fs::read_to_string(path)?.trim().to_string())
}

pub fn write_passphrase(passphrase: &str, path: &Path) -> Result<(), Error> {
    let mut file = fs::File::create(path)?;
    file.write_all(passphrase.as_bytes())?;
    Ok(())
}
