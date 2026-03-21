use crate::Error;
use clap::{Parser, Subcommand};
use grypt::{clean, init, read_passphrase, smudge, smudge_file};
use std::path::PathBuf;

pub fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            passphrase,
            config_path,
        } => {
            init(&passphrase, &config_path)?;
        }
        Commands::Clean { passphrase_path } => {
            let passphrase = read_passphrase(&passphrase_path)?;
            clean(&passphrase)?;
        }
        Commands::Smudge {
            passphrase_path,
            file_path,
        } => {
            let passphrase = read_passphrase(&passphrase_path)?;
            match file_path {
                Some(file_path) => smudge_file(file_path, &passphrase)?,
                None => smudge(&passphrase)?,
            }
        }
    }

    Ok(())
}

#[derive(Parser)]
#[command(
    name = "grypt",
    about = "Git filter encryption tool via age",
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize repo
    Init {
        /// Passphrase to use for encryption
        #[arg(short = 'p', long)]
        passphrase: String,
        /// Path to repo
        #[arg(short = 'c', long, default_value = ".grypt.toml")]
        config_path: PathBuf,
    },

    /// Encrypt stdin -> stdout
    Clean {
        /// Path to passphrase file
        #[arg(short = 'p', long, default_value = ".passphrase")]
        passphrase_path: PathBuf,
    },

    /// Decrypt stdin -> stdout
    Smudge {
        /// Path to passphrase file
        #[arg(short = 'p', long, default_value = ".passphrase")]
        passphrase_path: PathBuf,
        /// Path to file to decrypt. Reads from stdin if not specified.
        #[arg(short = 'f', long)]
        file_path: Option<PathBuf>,
    },
}
