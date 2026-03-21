# Grypt

Encrypt all files in a git repository with a passphrase using [`age`](https://crates.io/crates/age).

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
