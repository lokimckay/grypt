use crate::Error;
use age::{Decryptor, Encryptor, scrypt::Identity, secrecy::SecretString};
use git2::Repository;
use std::{
    fs,
    io::{self, Cursor, Read, Write},
    iter,
    path::Path,
};

/// Clean filter: stdin (plaintext) -> stdout (ciphertext)
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

/// Smudge filter: stdin (ciphertext) -> stdout (plaintext)
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
    mut output: W,
    passphrase: &str,
) -> Result<(), Error> {
    let mut plaintext = Vec::new();
    input.read_to_end(&mut plaintext)?;
    let ciphertext = encrypt_bytes(&plaintext, passphrase)?;
    output.write_all(&ciphertext)?;
    Ok(())
}

fn decrypt_stream<R: Read, W: Write>(
    mut input: R,
    mut output: W,
    passphrase: &str,
) -> Result<(), Error> {
    let mut ciphertext = Vec::new();
    input.read_to_end(&mut ciphertext)?;
    let plaintext = decrypt_bytes(&ciphertext, passphrase)?;
    output.write_all(&plaintext)?;
    Ok(())
}

/// Converts plaintext bytes to ciphertext.
/// If a staged blob exists and decrypts to the same plaintext, re-emits
/// the existing staged ciphertext to avoid false-dirty in git status due to non-deterministic encryption.
fn encrypt_bytes(plaintext: &[u8], passphrase: &str) -> Result<Vec<u8>, Error> {
    // Already ciphertext - pass through unchanged.
    if plaintext.starts_with(b"age-encryption.org/v1") {
        return Ok(plaintext.to_vec());
    }

    let staged_ciphertext = try_get_staged_ciphertext()?;
    tracing::debug!(
        "{} staged ciphertext",
        staged_ciphertext
            .is_some()
            .then(|| "Found")
            .unwrap_or("Didn't find")
    );

    // If the staged blob decrypts to the same plaintext, reuse its ciphertext exactly so git sees no change.
    if let Some(staged) = staged_ciphertext {
        if decrypt_bytes(&staged, passphrase)? == plaintext {
            return Ok(staged);
        }
    }

    // Plaintext is new or changed - fresh encryption required.
    let secret = SecretString::from(passphrase.to_string());
    let encryptor = Encryptor::with_user_passphrase(secret);
    let mut ciphertext = Vec::new();
    let mut writer = encryptor.wrap_output(&mut ciphertext)?;
    writer.write_all(plaintext)?;
    writer.finish()?;
    Ok(ciphertext)
}

/// Decrypts ciphertext bytes, returning plaintext.
fn decrypt_bytes(ciphertext: &[u8], passphrase: &str) -> Result<Vec<u8>, Error> {
    // Already plaintext - pass through unchanged.
    if !ciphertext.starts_with(b"age-encryption.org/v1") {
        return Ok(ciphertext.to_vec());
    }

    let mut cursor = Cursor::new(ciphertext);
    let decryptor = Decryptor::new(&mut cursor)?;
    let secret = SecretString::from(passphrase.to_string());
    let identity = Identity::new(secret);
    let mut reader = decryptor.decrypt(iter::once(&identity as _))?;
    let mut plaintext = Vec::new();
    reader.read_to_end(&mut plaintext)?;
    Ok(plaintext)
}

/// Tries to retrieve and return the raw ciphertext bytes of the currently staged file's blob.
/// Relies on the `GRYPT_FILE` environment variable being set by the git filter command
fn try_get_staged_ciphertext() -> Result<Option<Vec<u8>>, Error> {
    let file_path = match std::env::var("GRYPT_FILE").ok() {
        Some(p) if !p.is_empty() => p,
        _ => return Ok(None),
    };

    let repo = match std::env::var("GIT_DIR").ok().filter(|p| !p.is_empty()) {
        Some(git_dir) => Repository::open(git_dir)?,
        None => Repository::discover(".")?,
    };

    let index = repo.index()?;
    let entry = match index.get_path(Path::new(&file_path), 0) {
        Some(entry) => entry,
        None => return Ok(None),
    };

    let blob = repo.find_blob(entry.id)?;
    let content = blob.content();

    if content.starts_with(b"age-encryption.org/v1") {
        Ok(Some(content.to_vec()))
    } else {
        Ok(None)
    }
}
