use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json;

use crate::bindings::{ShortcutBinding, ShortcutBindingFile};

const SHORTCUT_CONFIG_DIR: &str = ".config/vasak";
const SHORTCUT_CONFIG_FILE: &str = "shortcut.json";

#[derive(Debug, Clone)]
pub struct Config {
    pub config_path: PathBuf,
    pub user_home: PathBuf,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let user_home = resolve_user_home()?;
        let config_path = user_home
            .join(SHORTCUT_CONFIG_DIR)
            .join(SHORTCUT_CONFIG_FILE);

        ensure_config_dir(&config_path)?;

        Ok(Config {
            config_path,
            user_home,
        })
    }

    pub fn config_dir(&self) -> &Path {
        self.config_path
            .parent()
            .unwrap_or_else(|| Path::new("/"))
    }

    pub fn load_bindings(&self) -> Result<Vec<ShortcutBinding>, Box<dyn Error>> {
        let raw = fs::read_to_string(&self.config_path)?;

        if raw.trim().is_empty() {
            return Ok(Vec::new());
        }

        let file_bindings: Vec<ShortcutBindingFile> = serde_json::from_str(&raw)?;
        Ok(file_bindings
            .into_iter()
            .map(ShortcutBinding::from_file)
            .collect())
    }
}

fn resolve_user_home() -> Result<PathBuf, Box<dyn Error>> {
    // Si el daemon corre con sudo, SUDO_USER contiene el usuario real
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        let home = format!("/home/{}", sudo_user);
        return Ok(PathBuf::from(home));
    }

    // Si no está en sudo, usar HOME directamente
    let home = std::env::var_os("HOME")
        .ok_or("HOME no esta definido y no se pudo detectar SUDO_USER")?;

    Ok(PathBuf::from(home))
}

fn ensure_config_dir(config_path: &Path) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    if !config_path.exists() {
        fs::write(config_path, "[]\n")?;
        println!(
            "✓ Creado archivo de shortcuts en {}",
            config_path.display()
        );
    }

    Ok(())
}
