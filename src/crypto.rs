use crate::Error;
use age::{Decryptor, Encryptor, scrypt::Identity, secrecy::SecretString};
use std::{
    fs,
    io::{self, Cursor, Read, Write},
    iter,
    path::Path,
};

pub fn clean(passphrase: &str) -> Result<(), Error> {
    encrypt_stream(io::stdin().lock(), io::stdout().lock(), passphrase)
}

pub fn clean_file(file_path: impl AsRef<Path>, passphrase: &str) -> Result<(), Error> {
    let input = fs::File::open(file_path)?;
    encrypt_stream(input, io::stdout().lock(), passphrase)
}

pub fn clean_file_to_file(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    passphrase: &str,
) -> Result<(), Error> {
    let input = fs::File::open(input_path)?;
    let output = fs::File::create(output_path)?;
    encrypt_stream(input, output, passphrase)
}

pub fn smudge(passphrase: &str) -> Result<(), Error> {
    decrypt_stream(io::stdin().lock(), io::stdout().lock(), passphrase)
}

pub fn smudge_file(file_path: impl AsRef<Path>, passphrase: &str) -> Result<(), Error> {
    let input = fs::File::open(file_path)?;
    decrypt_stream(input, io::stdout().lock(), passphrase)
}

pub fn smudge_file_to_file(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    passphrase: &str,
) -> Result<(), Error> {
    let input = fs::File::open(input_path)?;
    let output = fs::File::create(output_path)?;
    decrypt_stream(input, output, passphrase)
}

fn encrypt_stream<R: Read, W: Write>(
    mut input: R,
    output: W,
    passphrase: &str,
) -> Result<(), Error> {
    let passphrase = SecretString::from(passphrase.to_string());
    let encryptor = Encryptor::with_user_passphrase(passphrase);
    let mut writer = encryptor.wrap_output(output)?;
    io::copy(&mut input, &mut writer)?;
    writer.finish()?;
    Ok(())
}

fn decrypt_stream<R: Read, W: Write>(
    mut input: R,
    mut output: W,
    passphrase: &str,
) -> Result<(), Error> {
    let mut buffer = Vec::new();
    input.read_to_end(&mut buffer)?;

    if !buffer.starts_with(b"age-encryption.org/v1") {
        // Already plaintext - pass through unchanged
        output.write_all(&buffer)?;
        return Ok(());
    }

    let mut cursor = Cursor::new(buffer);
    let decryptor = Decryptor::new(&mut cursor)?;
    let passphrase = SecretString::from(passphrase.to_string());
    let identity = Identity::new(passphrase);
    let mut reader = decryptor.decrypt(iter::once(&identity as _))?;
    io::copy(&mut reader, &mut output)?;
    Ok(())
}
