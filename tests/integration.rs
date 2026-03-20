use assert_cmd::Command;
use git2::Repository;
use std::fs::{self};
use std::path::PathBuf;
use tempfile::{TempDir, tempdir_in};

type Error = Box<dyn std::error::Error>;
const PASSPHRASE: &str = "supersecret";

#[test]
fn test_encrypt_decrypt_roundtrip() -> Result<(), Error> {
    let temp_dir = tempdir_in(".")?;
    let file_path = create_file(&temp_dir, "notes.md", "# Secret Note\nBank: 123456");
    let enc_path = temp_dir.path().join("notes.md.age");
    let dec_path = temp_dir.path().join("notes_decrypted.md");

    grypt::clean_file_to_file(&file_path, &enc_path, PASSPHRASE)?;
    assert!(enc_path.exists());

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
    let passphrase_path = repo_path.join(".passphrase");
    let notes_path = repo_path.join("notes.md");

    grypt_cmd
        .arg("init")
        .arg("--passphrase")
        .arg(PASSPHRASE)
        .arg("--repository-path")
        .arg(repo_path)
        .arg("--passphrase-path")
        .arg(passphrase_path)
        .assert()
        .success();

    fs::write(&notes_path, "# Note\nBank: 123456")?;

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

    let entry = tree.get_path(std::path::Path::new("notes.md"))?;
    let blob = repo.find_blob(entry.id())?;
    let contents = blob.content();

    assert!(
        !contents.windows("# Note\nBank: 123456".len())
            .any(|w| w == b"# Note\nBank: 123456"),
        "Plaintext should not be committed!"
    );

    Ok(())
}

fn create_file(repo: &TempDir, name: &str, contents: &str) -> PathBuf {
    let path = repo.path().join(name);
    fs::write(&path, contents).unwrap();
    path
}