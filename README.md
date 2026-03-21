# Grypt

Encrypt all files in a git repository with a passphrase using [`age`](https://crates.io/crates/age).

## How does it work?

- Create and edit plaintext files in your repository as normal.
- Git `clean` and `smudge` filters encrypt and decrypt the files whenever you commit or pull.
- Your local repository has plaintext, but the remote repository is encrypted.

## Should you use this?

I wouldn't recommend using this project to store business critical workplace secrets.

I created this simple tool so I could store personal documents in private repositories with a little more security.
If you forget your passphrase and lose your local copy of the repository, the encrypted data will be lost forever.

## Usage

### Pushing to a remote repository

```bash
cargo install --git https://github.com/lokimckay/grypt
cd /path/to/repo
grypt init -p secretpassphrase
git add .
git commit -m "init"
git push
```

Pushed files will be encrypted with the passphrase `secretpassphrase`.

### Cloning a repository

TBA

## Development

### Running commands

```bash
cargo run init -c ./tmp/.grypt.toml -p passphrase
```

### Testing

```bash
cargo test
```
