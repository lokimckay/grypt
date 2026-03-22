use crate::{AGE_MAGIC_HEADER, Config, Error, write_passphrase};
use git2::Repository;
use std::{env, fmt::Write, fs, path::Path, process::Command};

/// Initializes a new repo with grypt.
pub fn init(passphrase: &str, config_path: &Path) -> Result<(), Error> {
    let config = ensure_grypt_config(config_path, passphrase)?;
    git2::Repository::init(&config.repository_path)?;
    add_git_attributes(&config, &config_path)?;
    add_git_config(&config)?;
    add_git_ignore(&config)?;

    if repo_has_encrypted_files(&config.repository_path)? {
        tracing::debug!("Encrypted files detected, decrypting...");
        decrypt_all()?;
    }

    Ok(())
}

/// Forces all repo files to run through the smudge filter.
/// Useful after a fresh clone of an encrypted repo.
pub fn decrypt_all() -> Result<(), Error> {
    let repo = Repository::discover(".")?;
    let workdir = repo
        .workdir()
        .ok_or("bare repositories are not supported")?;
    let index = repo.index()?;

    // Delete all indexed files.
    for entry in index.iter() {
        let path = workdir.join(String::from_utf8(entry.path)?);
        if path.exists() {
            fs::remove_file(&path)?;
        }
    }

    // Force git to checkout and smudge every deleted file.
    // We use standard git instead of git2 because git2 bypasses the smudge filter.
    let checkout = Command::new("git")
        .arg("checkout-index")
        .arg("--force")
        .arg("--all")
        .status()?;

    if !checkout.success() {
        return Err("git checkout-index failed".into());
    }

    // Refresh the index stat cache so git doesn't see the re-written files as dirty.
    let refresh = Command::new("git")
        .args(["add", "--renormalize", "."])
        .status()?;

    if !refresh.success() {
        return Err("git add --renormalize failed".into());
    }

    Ok(())
}

/// Returns true if any blob in the current index starts with the age magic header.
fn repo_has_encrypted_files(repo_path: &Path) -> Result<bool, Error> {
    let repo = Repository::open(repo_path)?;
    let index = repo.index()?;

    for entry in index.iter() {
        let blob = repo.find_blob(entry.id)?;
        if blob.content().starts_with(AGE_MAGIC_HEADER) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Reads from the given config file path if it exists, or writes the default config to that path.
fn ensure_grypt_config(config_path: &Path, passphrase: &str) -> Result<Config, Error> {
    let config = match Config::read(config_path)? {
        Some(config) => config,
        None => {
            let config = Config::default();
            Config::write(&config, config_path)?;
            config
        }
    };

    let base_dir = config_path.parent().ok_or("Config path has no parent")?;
    write_passphrase(passphrase, &base_dir.join(&config.passphrase_path))?;
    let config = config.resolve_paths(base_dir)?;
    tracing::debug!("Config: {:#?}", config);
    Ok(config)
}

fn add_git_attributes(config: &Config, config_path: &Path) -> Result<(), Error> {
    let attributes_path = config.repository_path.join(".gitattributes");

    let mut contents = String::new();
    for pattern in &config.include_patterns {
        writeln!(contents, "{} filter=grypt diff=grypt -text", pattern)?;
    }

    for pattern in &config.exclude_patterns {
        writeln!(contents, "{} -filter -diff", pattern)?;
    }

    writeln!(contents, ".git* -filter -diff")?;
    writeln!(contents, "{} -filter -diff", config.passphrase_filename()?)?;
    writeln!(
        contents,
        "{} -filter -diff",
        Config::filename_string(&config_path)?
    )?;

    fs::write(&attributes_path, contents)?;

    Ok(())
}

fn add_git_config(config: &Config) -> Result<(), Error> {
    let exe_path = path_to_string(env::current_exe()?.as_path())?;
    let pass_path = Config::make_path_relative(&config.repository_path, &config.passphrase_path)?;
    let pass_path = pass_path.to_str().ok_or("Invalid path")?.to_string();
    let clean_cmd = format!("{} clean -p {} -f %f", exe_path, pass_path);
    let smudge_cmd = format!("{} smudge -p {}", exe_path, pass_path);
    let diff_cmd = format!("{} -f", smudge_cmd);
    let repo = Repository::open(&config.repository_path)?;
    let mut config = repo.config()?;
    config.set_str("filter.grypt.clean", &clean_cmd)?;
    config.set_str("filter.grypt.smudge", &smudge_cmd)?;
    config.set_str("filter.grypt.required", "true")?;
    config.set_str("diff.grypt.textconv", &diff_cmd)?;
    Ok(())
}

fn add_git_ignore(config: &Config) -> Result<(), Error> {
    let ignore_path = config.repository_path.join(".gitignore");
    fs::write(&ignore_path, config.passphrase_filename()?)?;

    Ok(())
}

fn path_to_string(path: &Path) -> Result<String, Error> {
    let path = path.canonicalize()?;
    let path = path.to_str().ok_or("Invalid path")?.to_string();
    #[cfg(windows)]
    let path = {
        let path = path.strip_prefix(r"\\?\").unwrap_or(&path).to_string();
        let path = path.replace("\\", "/");
        path
    };
    Ok(path)
}
