use std::fs;
use std::path::PathBuf;

use crate::Args;
use anyhow::{Error, Result};
// use re-exported version of `CookieStore` for crate compatibility
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use serde::Deserialize;
use std::sync::Arc;

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

    let config_path = config_file()?;

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

/// Return the path to the jwctl configuration file
pub fn config_file() -> Result<PathBuf> {
    let mut path = config_dir()?;
    path.push(CONFIG_FILE);
    Ok(path)
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

/// Load an existing set of cookies, serialized as json
pub fn get_cookie_store() -> Result<Arc<CookieStoreMutex>> {
    let mut path = config_dir()?;
    path.push("cookies.json");
    debug!("Loading cookies from {:?}", path);

    let store = match fs::File::open(path).map(std::io::BufReader::new) {
        Ok(file) => CookieStore::load_json_all(file)
            .map_err(|err| Error::msg(format!("Failed to load cookie file: {err}")))?,
        _ => CookieStore::new(None),
    };
    let store = CookieStoreMutex::new(store);
    let store = Arc::new(store);
    Ok(store)
}

/// Write reqwest cookies back to disk
pub fn save_cookies(cookie_store: Arc<CookieStoreMutex>) -> Result<()> {
    let mut path = config_dir()?;
    fs::create_dir_all(&path)?;
    path.push("cookies.json");
    debug!("Saving cookies to {:?}", path);

    let mut writer = std::fs::File::create(path).map(std::io::BufWriter::new)?;
    let store = cookie_store
        .lock()
        .map_err(|_| Error::msg("Could not lock the cookie store to save cookies"))?;

    store
        .save_incl_expired_and_nonpersistent_json(&mut writer)
        .map_err(|err| Error::msg(format!("Failed to write cookies to disk: {err}")))?;
    Ok(())
}
