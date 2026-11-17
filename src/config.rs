use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Default)]
pub struct Config {
    pub verbose: bool,
    pub loaded_files: Vec<PathBuf>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    verbose: Option<bool>,
}

#[derive(Debug)]
pub struct ConfigError(String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for ConfigError {}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let cwd = env::current_dir()
            .map_err(|error| ConfigError(format!("current directory: {error}")))?;
        let mut directories = config_directories();
        let project_root = cwd
            .ancestors()
            .find(|dir| dir.join(".git").exists())
            .unwrap_or(&cwd);
        directories.push(project_root.join(".rh"));

        let mut config = Self::default();
        for directory in directories {
            for name in ["config.toml", "config.json"] {
                let path = directory.join(name);
                let contents = match fs::read_to_string(&path) {
                    Ok(contents) => contents,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                    Err(error) => return Err(ConfigError(format!("{}: {error}", path.display()))),
                };
                let file: ConfigFile = if name.ends_with(".toml") {
                    toml::from_str(&contents)
                        .map_err(|error| ConfigError(format!("{}: {error}", path.display())))?
                } else {
                    serde_json::from_str(&contents)
                        .map_err(|error| ConfigError(format!("{}: {error}", path.display())))?
                };
                if let Some(verbose) = file.verbose {
                    config.verbose = verbose;
                }
                config.loaded_files.push(path);
            }
        }
        Ok(config)
    }
}

fn absolute_env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}

#[cfg(not(windows))]
fn config_directories() -> Vec<PathBuf> {
    let mut directories: Vec<_> = env::var_os("XDG_CONFIG_DIRS")
        .filter(|value| !value.is_empty())
        .map(|value| {
            env::split_paths(&value)
                .filter(|path| path.is_absolute())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec![PathBuf::from("/etc/xdg")])
        .into_iter()
        .rev()
        .map(|path| path.join("rh"))
        .collect();

    #[cfg(target_os = "macos")]
    directories.push(PathBuf::from("/Library/Application Support/rh"));

    if let Some(user) = absolute_env_path("XDG_CONFIG_HOME")
        .or_else(|| absolute_env_path("HOME").map(|home| home.join(".config")))
    {
        directories.push(user.join("rh"));
    }

    #[cfg(target_os = "macos")]
    if let Some(home) = absolute_env_path("HOME") {
        directories.push(home.join("Library/Application Support/rh"));
    }

    directories
}

#[cfg(windows)]
fn config_directories() -> Vec<PathBuf> {
    let mut directories = Vec::new();
    if let Some(system) = absolute_env_path("PROGRAMDATA") {
        directories.push(system.join("rh"));
    }
    if let Some(home) = absolute_env_path("USERPROFILE") {
        directories.push(home.join(".config").join("rh"));
    }
    if let Some(user) = absolute_env_path("APPDATA") {
        directories.push(user.join("rh"));
    }
    directories
}
