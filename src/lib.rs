mod config;
mod crypto;
mod init;

pub(crate) use config::{Config, write_passphrase};
pub(crate) use crypto::AGE_MAGIC_HEADER;

pub use config::read_passphrase;
pub use crypto::{clean, clean_file_to_file, smudge, smudge_file, smudge_file_to_file};
pub use init::init;
pub type Error = Box<dyn std::error::Error>;
