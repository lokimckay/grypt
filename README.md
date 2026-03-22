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

✔️ Pushed files are encrypted to `age` blobs.

### Cloning a repository

```shell
git clone https://github.com/username/repository
cd repository
grypt init -p secretpassphrase
```

✔️ Cloned files are decrypted to plaintext.

## How does it work?

- Create and edit plaintext files in your repository as normal.
- Git `clean` and `smudge` filters encrypt and decrypt the files whenever you commit or checkout.
- Local files remain as plaintext, remote repository receives encrypted data.

## Should you use this?

This project should not be used in serious contexts.

This is a simple tool for storing personal documents in private repositories with a little more security.

> [!WARNING]
> If you forget your passphrase and lose your local decrypted files, the encrypted data will be lost forever.
