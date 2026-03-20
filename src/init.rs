use crate::{Error, write_passphrase};
use git2::Repository;
use std::{env, fs, path::Path};

pub fn init(repo_path: &Path, passphrase_path: &Path, passphrase: &str) -> Result<(), Error> {
    git2::Repository::init(repo_path)?;
    write_passphrase(passphrase, passphrase_path)?;
    add_git_attributes(repo_path, passphrase_path)?;
    add_git_config(repo_path, passphrase_path)?;
    add_git_ignore(repo_path, passphrase_path)?;
    Ok(())
}

fn add_git_attributes(repo_path: &Path, passphrase_path: &Path) -> Result<(), Error> {
    let filename = passphrase_filename(passphrase_path)?;
    let attributes_path = repo_path.join(".gitattributes");
    fs::write(
        &attributes_path,
        format!(
            r#"
* filter=grypt -text
* diff=grypt
.gitattributes -filter -diff
.gitignore -filter -diff
{} -filter -diff"#,
            filename
        ),
    )?;

    let repo = Repository::open(repo_path)?;
    let mut index = repo.index()?;
    index.add_path(Path::new(".gitattributes"))?;
    index.write()?;

    Ok(())
}

fn add_git_config(repo_path: &Path, passphrase_path: &Path) -> Result<(), Error> {
    let exe_path = path_to_string(env::current_exe()?.as_path())?;
    let passphrase_path = path_to_string(passphrase_path)?;
    let clean_cmd = format!("{} clean --passphrase-path {}", exe_path, passphrase_path);
    let smudge_cmd = format!("{} smudge --passphrase-path {}", exe_path, passphrase_path);
    let diff_cmd = format!("{} --file-path", smudge_cmd);
    let repo = Repository::open(repo_path)?;
    let mut config = repo.config()?;
    config.set_str("filter.grypt.clean", &clean_cmd)?;
    config.set_str("filter.grypt.smudge", &smudge_cmd)?;
    config.set_str("filter.grypt.required", "true")?;
    config.set_str("diff.grypt.textconv", &diff_cmd)?;
    Ok(())
}

fn add_git_ignore(repo_path: &Path, passphrase_path: &Path) -> Result<(), Error> {
    let filename = passphrase_filename(passphrase_path)?;
    let ignore_path = repo_path.join(".gitignore");
    fs::write(&ignore_path, filename)?;

    let repo = Repository::open(repo_path)?;
    let mut index = repo.index()?;
    index.add_path(Path::new(".gitignore"))?;
    index.write()?;

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

fn passphrase_filename(passphrase_path: &Path) -> Result<String, Error> {
    Ok(passphrase_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid path")?
        .to_string())
}
