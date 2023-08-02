use crate::config::Config;
use anyhow::Result;
use serde_json::Value;

/// Retrieve status information from the proxy server
pub fn status(config: Config) -> Result<Value, reqwest::Error> {
    let mut url = config.url;
    url.set_path("/_jumpwire/status");
    reqwest::blocking::get(url)?.json()
}

/// Issue a ping command expecting to get back a pong
pub fn ping(config: Config) -> Result<String, reqwest::Error> {
    let mut url = config.url;
    url.set_path("/_jumpwire/ping");
    reqwest::blocking::get(url)?.text()
}
