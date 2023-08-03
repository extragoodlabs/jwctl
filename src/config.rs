use std::fs;
use std::path::PathBuf;

use crate::Args;
use anyhow::{Error, Result};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub url: url::Url,
    pub token: Option<String>,
}

const TOKEN_FILE: &str = ".token";
const CONFIG_FILE: &str = "config.yaml";

/// Load and merge configuration from multiple sources. In decreasing
/// preference order, configuration values are loaded from:
/// - command line options
/// - env vars prefixed with `JW_`
/// - ~/.config/jwctl/config.yaml
pub fn load_config(args: Args) -> Result<Config> {
    let dir_path = config_dir()?;
    debug!("Loading configuration from {:?}", dir_path);

    let mut config_path = dir_path.clone();
    config_path.push(CONFIG_FILE);

    // Create a config from the token file, if it exists, and
    // merge it into the main config.
    let mut token_path = dir_path.clone();
    token_path.push(TOKEN_FILE);
    let token = fs::read_to_string(token_path).ok();
    let token_config = config::Config::builder()
        .set_override_option("token", token)?
        .build()?;

    let config = config::Config::builder()
        .add_source(config::File::from(config_path).required(false))
        .add_source(token_config)
        .add_source(config::Environment::with_prefix("JW"))
        .add_source(args)
        .build()?
        .try_deserialize()?;

    Ok(config)
}

fn config_dir() -> Result<PathBuf> {
    let mut path = home::home_dir().ok_or(Error::msg("Unable to find home dir!"))?;
    path.push(".config");
    path.push("jwctl");
    Ok(path)
}

/// Store an authentication token into a local file.
pub fn save_token(token: String) -> Result<()> {
    let mut path = config_dir()?;
    fs::create_dir_all(&path)?;

    path.push(TOKEN_FILE);
    info!("Saving token to {:?}", path);
    fs::write(path, token)?;
    Ok(())
}
