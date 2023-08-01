use crate::Args;
use anyhow::{Error, Result};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub url: url::Url,
}

/// Load and merge configuration from multiple sources. In decreasing
/// preference order, configuration values are loaded from:
/// - command line options
/// - env vars prefixed with `JW_`
/// - ~/.config/jwctl.config.yaml
pub fn load_config(args: Args) -> Result<Config> {
    let mut path = home::home_dir().ok_or(Error::msg("Unable to find home dir!"))?;
    path.push(".config");
    path.push("jwctl");
    path.push("config.yaml");
    debug!("Loading configuration from {:?}", path);

    let config = config::Config::builder()
        .add_source(config::File::from(path).required(false))
        .add_source(config::Environment::with_prefix("JW"))
        .add_source(args)
        .build()?
        .try_deserialize()?;

    Ok(config)
}
