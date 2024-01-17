use anyhow::{ensure, Context, Result};
use std::path::PathBuf;

use crate::config::Config;
use crate::shell::write_session_script;
use crate::sshkey::generate_ssh_key;
use crate::user::{User, Users};

pub struct GitUserSwitcher {
    users: Users,
    config: Config,
}

impl From<&PathBuf> for GitUserSwitcher {
    fn from(config_path: &PathBuf) -> Self {
        let config = Config::open(config_path).unwrap();
        let users = Users::open(&config.users_file_path).unwrap();
        Self { users, config }
    }
}

impl GitUserSwitcher {
    pub fn add_user(&mut self, user: User, sshkey_passphrase: Option<&str>) -> Result<()> {
        self.users.add(user.clone())?;

        let sshkey_path = user.get_sshkey_path(&self.config.default_sshkey_dir);

        if !sshkey_path.exists() {
            let pass = sshkey_passphrase.context("ssh key passphrase required")?;

            generate_ssh_key(
                self.config.default_sshkey_type.clone(),
                &user.get_sshkey_name(),
                &pass,
                &sshkey_path,
            )
            .with_context(|| format!("failed to generate ssh key for user: {}", &user.id))?;
        }

        self.users.save(&self.config.users_file_path)?;
        Ok(())
    }

    pub fn remove_user(&mut self, id: &str) -> Result<()> {
        ensure!(
            self.users.exists(id),
            "user with id '{}' does not exist",
            id
        );
        self.users.remove(id);
        self.users.save(&self.config.users_file_path)?;
        Ok(())
    }

    pub fn switch_user(&self, id: &str) -> Result<()> {
        ensure!(
            self.users.exists(id),
            "user with id '{}' does not exist",
            id
        );
        let user = self.users.get(id).unwrap();

        let script = format!(
            "\
            export GIT_USER_SWITCHER_USER_ID=\"{id}\"\n\
            export GIT_AUTHOR_NAME=\"{name}\"\n\
            export GIT_AUTHOR_EMAIL=\"{email}\"\n\
            export GIT_COMMITTER_NAME=\"{name}\"\n\
            export GIT_COMMITTER_EMAIL=\"{email}\"\n\
            export GIT_SSH_COMMAND=\"ssh -i {sshkey_path} -F /dev/null\"\n\
            ",
            id = user.id,
            name = user.name,
            email = user.email,
            sshkey_path = user
                .get_sshkey_path(&self.config.default_sshkey_dir)
                .to_string_lossy()
        );

        write_session_script(&script)?;

        Ok(())
    }

    pub fn list_users(&self) -> Vec<&User> {
        self.users.list()
    }

    pub fn get_user(&self, id: &str) -> Option<&User> {
        self.users.get(id)
    }

    pub fn exists_user(&self, id: &str) -> bool {
        self.users.exists(id)
    }
}
