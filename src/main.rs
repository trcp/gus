use anyhow::Result;

mod cli;
mod config;
mod gus;
mod shell;
mod sshkey;
mod user;
mod cmd;

use crate::cli::run;

fn main() -> Result<()> {
    run()?;
    Ok(())
}
