use assert_cmd::Command;
use git2::Repository;
use std::fs::{self};
use std::path::{Path, PathBuf};
use tempfile::tempdir_in;

type Error = Box<dyn std::error::Error>;
const PASSPHRASE: &str = "supersecret";
const SECRET_NOTE: &str = "# Secret Note\nBank: 123456";

#[test]
fn test_encrypt_decrypt_roundtrip() -> Result<(), Error> {
    let temp_dir = tempdir_in(".")?;
    let file_path = create_secret_file(&temp_dir.path().join("notes.md"))?;
    let enc_path = temp_dir.path().join("notes.md.age");
    let dec_path = temp_dir.path().join("notes_decrypted.md");

    grypt::clean_file_to_file(&file_path, &enc_path, PASSPHRASE)?;
    assert!(enc_path.exists());

    let failure = grypt::smudge_file_to_file(&enc_path, &dec_path, "wrongpassphrase");
    assert!(failure.is_err());

    grypt::smudge_file_to_file(&enc_path, &dec_path, PASSPHRASE)?;
    let decrypted_contents = fs::read_to_string(&dec_path)?;
    let original_contents = fs::read_to_string(&file_path)?;
    assert_eq!(original_contents, decrypted_contents);

    Ok(())
}

#[test]
fn test_git_commit_encrypt() -> Result<(), Error> {
    let mut grypt_cmd = Command::cargo_bin("grypt")?;
    let temp_dir = tempdir_in(".")?;
    let repo_path = temp_dir.path();
    let config_path = repo_path.join(".grypt.toml");
    create_secret_file(&repo_path.join("notes.md"))?;

    grypt_cmd
        .arg("init")
        .arg("--passphrase")
        .arg(PASSPHRASE)
        .arg("--config-path")
        .arg(config_path)
        .assert()
        .success();

    Command::new("git")
        .args(&["add", "."])
        .current_dir(repo_path)
        .assert()
        .success();

    Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .assert()
        .success();

    let repo = Repository::open(repo_path)?;
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;
    let tree = commit.tree()?;

    let entry = tree.get_path(Path::new("notes.md"))?;
    let blob = repo.find_blob(entry.id())?;
    let contents = blob.content();

    assert!(
        !contents
            .windows(SECRET_NOTE.len())
            .any(|w| w == SECRET_NOTE.as_bytes()),
        "Plaintext should not be committed!"
    );

    Ok(())
}

fn create_secret_file(path: &Path) -> Result<PathBuf, Error> {
    fs::write(path, SECRET_NOTE)?;
    Ok(path.to_path_buf())
}
