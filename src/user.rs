use anyhow::{ensure, Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, path::PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone, Args)]
pub struct User {
    /// The user's ID (must be unique)
    pub id: String,
    /// The user's name
    pub name: String,
    /// The user's email
    pub email: String,

    /// The path to the user's ssh key
    #[clap(long, short)]
    pub sshkey_path: Option<PathBuf>,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} <{}>", self.id, self.name, self.email)
    }
}

impl User {
    pub fn get_sshkey_name(&self) -> String {
        if let Some(path) = &self.sshkey_path {
            path.file_name().unwrap().to_str().unwrap().to_string()
        } else {
            format!("id_{}", self.id)
        }
    }

    pub fn get_sshkey_path(&self, default_sshkey_dir: &PathBuf) -> PathBuf {
        if let Some(path) = &self.sshkey_path {
            path.clone()
        } else {
            default_sshkey_dir.join(&self.get_sshkey_name())
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Users {
    #[serde(flatten)]
    hashmap: HashMap<String, User>,
}

impl Users {
    pub fn new() -> Self {
        Self {
            hashmap: HashMap::new(),
        }
    }

    pub fn open(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            let users = Self::new();
            users.save(path)?;
            return Ok(users);
        }

        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read users file: {}", path.display()))?;
        let users = toml::from_str(&contents)
            .with_context(|| format!("failed to parse users file: {}", path.display()))?;
        Ok(users)
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        if !path.exists() {
            std::fs::create_dir_all(&path.parent().unwrap())
                .with_context(|| format!("failed to create users directory: {}", path.display()))?;
        }

        let contents = toml::to_string(&self)
            .with_context(|| format!("failed to serialize users file: {}", path.display()))?;
        std::fs::write(&path, contents)
            .with_context(|| format!("failed to write users file: {}", path.display()))?;
        Ok(())
    }

    pub fn exists(&self, id: &str) -> bool {
        self.hashmap.contains_key(id)
    }

    pub fn add(&mut self, user: User) -> Result<()> {
        ensure!(
            !self.exists(&user.id),
            "user with id '{}' already exists",
            user.id
        );
        self.hashmap.insert(user.id.clone(), user);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&User> {
        self.hashmap.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut User> {
        self.hashmap.get_mut(id)
    }

    pub fn remove(&mut self, id: &str) -> Option<User> {
        self.hashmap.remove(id)
    }

    pub fn list(&self) -> Vec<&User> {
        self.hashmap.values().collect()
    }
}
