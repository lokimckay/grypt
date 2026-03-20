mod cli;
use grypt::Error;

fn main() -> Result<(), crate::Error> {
    crate::cli::run()
}
