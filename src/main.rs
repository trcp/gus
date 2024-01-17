use anyhow::Result;

mod cli;
mod config;
mod gus;
mod shell;
mod sshkey;
mod user;

use crate::cli::run;

fn main() -> Result<()> {
    run()?;
    Ok(())
}
