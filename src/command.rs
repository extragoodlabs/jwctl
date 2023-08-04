use crate::config::Config;
use anyhow::{Error, Result};
use reqwest::blocking::RequestBuilder;
use serde_json::Value;

/// Retrieve status information from the proxy server
pub fn status(config: Config) -> Result<Value, reqwest::Error> {
    let mut url = config.url;
    url.set_path("/_jumpwire/status");
    let request = reqwest::blocking::Client::new().get(url);
    maybe_add_auth(request, config.token).send()?.json()
}

/// Issue a ping command expecting to get back a pong
pub fn ping(config: Config) -> Result<String, reqwest::Error> {
    let mut url = config.url;
    url.set_path("/_jumpwire/ping");
    let request = reqwest::blocking::Client::new().get(url);
    maybe_add_auth(request, config.token).send()?.text()
}

/// Authenticate with a token, storing it in the local config file
pub fn authenticate(config: Config) -> Result<()> {
    let token = config.token.ok_or(Error::msg("No token provided"))?;
    crate::config::save_token(token)
}

pub fn config_get(config: Config) -> Result<()> {
    info!("Current configuration:\n{:#?}", config);
    Ok(())
}

pub fn whoami(config: Config) -> Result<Value, reqwest::Error> {
    let mut url = config.url;
    url.set_path("/_jumpwire/token");
    let request = reqwest::blocking::Client::new().get(url);
    maybe_add_auth(request, config.token).send()?.json()
}

fn maybe_add_auth(request: RequestBuilder, token: Option<String>) -> RequestBuilder {
    match token {
        Some(token) => request.bearer_auth(token),
        None => request,
    }
}
