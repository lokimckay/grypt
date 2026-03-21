use crate::{Config, Error, write_passphrase};
use git2::Repository;
use std::{env, fmt::Write, fs, path::Path};

pub fn init(passphrase: &str, config_path: &Path) -> Result<(), Error> {
    let config = ensure_grypt_config(config_path, passphrase)?;
    git2::Repository::init(&config.repository_path)?;
    add_git_attributes(&config, &config_path)?;
    add_git_config(&config)?;
    add_git_ignore(&config)?;
    Ok(())
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
    let passphrase_path =
        Config::make_path_relative(&config.repository_path, &config.passphrase_path)?;
    let passphrase_path = passphrase_path.to_str().ok_or("Invalid path")?.to_string();
    let clean_cmd = format!("{} clean --passphrase-path {}", exe_path, passphrase_path);
    let smudge_cmd = format!("{} smudge --passphrase-path {}", exe_path, passphrase_path);
    let diff_cmd = format!("{} --file-path", smudge_cmd);
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
