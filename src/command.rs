use std::collections::HashMap;

use crate::config::{get_cookie_store, save_cookies, Config};
use anyhow::{Error, Result};
use itertools::Itertools;
use reqwest::blocking::RequestBuilder;
use serde_json::Value;
use std::sync::Arc;

/// Retrieve status information from the proxy server
pub fn status(config: Config) -> Result<Value> {
    let mut url = config.url;
    url.set_path("/_jumpwire/status");
    let cookie_store = get_cookie_store()?;
    let request = client(&cookie_store)?.get(url);
    let resp = maybe_add_auth(request, config.token).send()?.json()?;
    Ok(resp)
}

/// Issue a ping command expecting to get back a pong
pub fn ping(config: Config) -> Result<String> {
    let mut url = config.url;
    url.set_path("/_jumpwire/ping");
    let cookie_store = get_cookie_store()?;
    let request = client(&cookie_store)?.get(url);
    let resp = maybe_add_auth(request, config.token).send()?.text()?;
    Ok(resp)
}

/// Authenticate with a token, storing it in the local config file
pub fn authenticate(config: Config) -> Result<()> {
    let token = config.token.ok_or(Error::msg("No token provided"))?;
    crate::config::save_token(token)
}

/// Print out the configuration
pub fn config_get(config: Config) -> Result<()> {
    info!("Current configuration:\n{:#?}", config);
    Ok(())
}

/// Check configured token permissions
pub fn token_whoami(config: Config) -> Result<Value> {
    let mut url = config.url;
    url.set_path("/_jumpwire/token");
    let cookie_store = get_cookie_store()?;
    let request = client(&cookie_store)?.get(url);
    let resp = maybe_add_auth(request, config.token).send()?.json()?;
    Ok(resp)
}

/// Generate a new token with specific permissions
pub fn generate_token(config: Config, permissions: &[String]) -> Result<Value> {
    let mut url = config.url;
    url.set_path("/_jumpwire/token");

    let permissions: HashMap<&str, Vec<&str>> = permissions
        .iter()
        .map(|p| {
            let mut parts = p.splitn(2, ':');
            let method = parts.next().ok_or(Error::msg("Invalid permission"))?;
            let action = parts.next().ok_or(Error::msg("Invalid permission"))?;
            Ok::<(&str, &str), Error>((method, action))
        })
        .process_results(|iter| iter.into_group_map())?;

    let mut body = HashMap::new();
    body.insert("permissions", permissions);

    let cookie_store = get_cookie_store()?;
    let request = client(&cookie_store)?.post(url).json(&body);
    let result = maybe_add_auth(request, config.token).send()?.json()?;
    Ok(result)
}

/// List all configured SSO providers
pub fn auth_list(config: Config) -> Result<Value> {
    let mut url = config.url;
    url.set_path("/sso");
    let cookie_store = get_cookie_store()?;
    let request = client(&cookie_store)?.get(url);
    let resp = maybe_add_auth(request, config.token).send()?.json()?;
    Ok(resp)
}

/// Start an SSO login flow
pub fn auth_login(config: Config, idp: &str) -> Result<Value> {
    let target = "/sso/result";

    let mut url = config.url.clone();
    url.set_path("/sso/auth/signin");
    url.path_segments_mut()
        .map_err(|_| Error::msg("Could not set URL path"))?
        .push(idp);
    url.query_pairs_mut()
        .append_pair("target_url", &urlencoding::encode(target));

    open::that(url.as_str())?;
    info!("The login URL will open automatically in your browser. If it does not, you can enter it directly:\n\n{:}\n\nAfter authenticating, enter the code displayed:", url.to_string());

    let code = read_code()?;

    let mut url = config.url.clone();
    url.set_path("/sso/validate");
    let mut body = HashMap::new();
    body.insert("sso_code", code);

    let cookie_store = get_cookie_store()?;
    let result = client(&cookie_store)?
        .post(url)
        .json(&body)
        .send()?
        .json()?;
    save_cookies(cookie_store)?;

    Ok(result)
}

/// Check the currently authenticated user
pub fn sso_whoami(config: Config) -> Result<Value> {
    let mut url = config.url;
    url.set_path("/sso/whoami");
    let cookie_store = get_cookie_store()?;

    let request = client(&cookie_store)?.get(url);
    let resp = maybe_add_auth(request, config.token).send()?.json()?;
    Ok(resp)
}

fn read_code() -> Result<String> {
    let mut guess = String::new();

    std::io::stdin()
        .read_line(&mut guess)
        .map_err(|_| Error::msg("Failed to read line"))?;

    Ok(guess.trim().to_string())
}

fn client(
    cookie_store: &Arc<reqwest_cookie_store::CookieStoreMutex>,
) -> Result<reqwest::blocking::Client> {
    let client = reqwest::blocking::ClientBuilder::new()
        .cookie_store(true)
        .cookie_provider(Arc::clone(cookie_store))
        .build()?;
    Ok(client)
}

fn maybe_add_auth(request: RequestBuilder, token: Option<String>) -> RequestBuilder {
    match token {
        Some(token) => request.bearer_auth(token),
        None => request,
    }
}
