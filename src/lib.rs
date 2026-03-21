mod config;
mod crypto;
mod init;

pub use config::{Config, read_passphrase, write_passphrase};
pub use crypto::{clean, clean_file, clean_file_to_file, smudge, smudge_file, smudge_file_to_file};
pub use init::init;
pub type Error = Box<dyn std::error::Error>;
