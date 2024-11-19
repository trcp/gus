use anyhow::{ensure, Context, Result};
use clap::{Parser, Subcommand};
use once_cell::sync::Lazy;
use rpassword::read_password;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::gus::GitUserSwitcher;
use crate::shell::get_setup_script;
use crate::user::User;

use crate::cmd;

static DEFAULT_CONFIG_PATH: Lazy<PathBuf> =
    Lazy::new(|| dirs::home_dir().unwrap().join(".config/gus/config.toml"));

#[derive(Parser)]
#[clap(name = env!("CARGO_PKG_NAME"), version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"), about = env!("CARGO_PKG_DESCRIPTION"))]
struct Cli {
    #[clap(subcommand)]
    subcmd: Subcommands,

    /// The path to the config file
    #[clap(long, short, default_value = &DEFAULT_CONFIG_PATH.to_str().unwrap())]
    config: PathBuf,
}

#[derive(Subcommand)]
enum Subcommands {
    /// Echo a shell script to setup the shell for this app
    Setup,

    /// Add a new user
    Add {
        #[clap(flatten)]
        user: User,
    },

    /// Remove a user
    Remove {
        /// The ID of the user to remove
        id: String,
    },

    /// Switch to a user
    Set {
        /// The ID of the user to switch to
        id: String,
    },

    /// List all users
    List,

    /// catch UNKONWN subcommands
    #[clap(external_subcommand)]
    Unknown(Vec<String>),
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    let mut gus = GitUserSwitcher::from(&cli.config);

    match cli.subcmd {
        Subcommands::Setup => {
            println!("{}", get_setup_script())
        }
        Subcommands::Add { user } => {
            ensure!(
                !gus.exists_user(&user.id),
                "user with id '{}' already exists",
                user.id
            );

            let is_required_sshkey_passphrase = if let Some(sshkey_path) = &user.sshkey_path {
                !sshkey_path.exists()
            } else {
                true
            };

            let sshkey_passphrase = if is_required_sshkey_passphrase {
                print!("Enter new ssh key passphrase (10+ chars recommended): ");
                io::stdout().flush().unwrap();
                Some(read_password().context("failed to read ssh key passphrase")?)
            } else {
                None
            };

            gus.add_user(user, sshkey_passphrase.as_deref())?;
        }
        Subcommands::Remove { id } => {
            gus.remove_user(&id)?;
        }
        Subcommands::Set { id } => {
            gus.switch_user(&id)?;
        }
        Subcommands::List => {
            for user in gus.list_users() {
                println!("{}", user);
            }
        }
        Subcommands::Unknown(args) => {
            if args.is_empty() {
                println!("No unknown subcommand provided.");
            } else {
                println!("Unrecognized subcommand: {}", args[0]);
                let join_cmd = args.join(" ");
                println!("Command: {}", join_cmd);
                cmd::cmd::run_cmd(&join_cmd);
            }
        }
    }

    Ok(())
}
