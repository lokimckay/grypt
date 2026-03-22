use crate::Error;
use clap::{Parser, Subcommand};
use grypt::{clean, init, read_passphrase, smudge, smudge_file};
use std::{env, io, path::PathBuf};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn run() -> Result<(), Error> {
    init_tracing();
    let cli = Cli::parse();
    tracing::debug!("CLI parsed: {:?}", cli);

    match cli.command {
        Commands::Init {
            passphrase,
            config_path,
        } => {
            init(&passphrase, &config_path)?;
        }
        Commands::Clean {
            passphrase_path,
            file_path,
        } => {
            record_file_path(&file_path);
            let passphrase = read_passphrase(&passphrase_path)?;
            clean(&passphrase)?;
        }
        Commands::Smudge {
            passphrase_path,
            file_path,
        } => {
            record_file_path(&file_path);
            let passphrase = read_passphrase(&passphrase_path)?;
            match file_path {
                Some(file_path) => smudge_file(file_path, &passphrase)?,
                None => smudge(&passphrase)?,
            }
        }
    }

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(io::stderr))
        .with(EnvFilter::from_default_env())
        .init();
}

fn record_file_path(file_path: &Option<PathBuf>) {
    if let Some(file_path) = file_path {
        unsafe {
            env::set_var("GRYPT_FILE", file_path);
        }
        tracing::debug!("GRYPT_FILE set to {}", file_path.display());
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "grypt",
    about = "Git filter encryption tool via age",
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
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

    /// Encrypt stdin/file -> stdout
    Clean {
        /// Path to passphrase file
        #[arg(short = 'p', long, default_value = ".passphrase")]
        passphrase_path: PathBuf,
        /// Path to file to encrypt. Reads from stdin if not specified.
        #[arg(short = 'f', long)]
        file_path: Option<PathBuf>,
    },

    /// Decrypt stdin/file -> stdout
    Smudge {
        /// Path to passphrase file
        #[arg(short = 'p', long, default_value = ".passphrase")]
        passphrase_path: PathBuf,
        /// Path to file to decrypt. Reads from stdin if not specified.
        #[arg(short = 'f', long)]
        file_path: Option<PathBuf>,
    },
}
