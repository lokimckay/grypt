# Grypt

Encrypt all files in a git repository with a passphrase via [`age`](https://crates.io/crates/age).

## Usage

### Install

```shell
cargo install --git https://github.com/lokimckay/grypt
```

### Pushing to a remote repository

```shell
grypt init -p secretpassphrase
git add .
git commit -m "init"
git push
```

Pushed files will be encrypted with the passphrase `secretpassphrase`.

### Cloning a repository

```shell
git clone https://github.com/username/repository
cd repository
grypt init -p secretpassphrase
```

Cloned files will be decrypted using the passphrase `secretpassphrase`.

## How does it work?

- Create and edit plaintext files in your repository as normal.
- Git `clean` and `smudge` filters encrypt and decrypt the files whenever you commit or pull.
- Your local repository has plaintext, but the remote repository is encrypted.

## Should you use this?

This project should not be used in serious contexts.

This is a simple tool for storing personal documents in private repositories with a little more security.

> [!WARNING]
> If you forget your passphrase and lose your local decyrpted files, the encrypted data will be lost forever.
