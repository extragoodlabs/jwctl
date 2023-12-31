use std::collections::HashMap;

use crate::config::{get_cookie_store, save_cookies, Config};
use crate::http::{client, maybe_add_auth};
use anyhow::{Error, Result};
use itertools::Itertools;

use serde::Deserialize;
use serde_json::Value;

/// Retrieve status information from the proxy server
pub fn status(config: Config) -> Result<Value> {
    let mut url = config.url;
    url.set_path("/api/v1/status");
    let cookie_store = get_cookie_store()?;
    let request = client(&cookie_store)?.get(url);
    let resp = maybe_add_auth(request, config.token).send()?.json()?;
    Ok(resp)
}

/// Issue a ping command expecting to get back a pong
pub fn ping(config: Config) -> Result<String> {
    let mut url = config.url;
    url.set_path("/ping");
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
    url.set_path("/api/v1/token");
    let cookie_store = get_cookie_store()?;
    let request = client(&cookie_store)?.get(url);
    let resp = maybe_add_auth(request, config.token).send()?.json()?;
    Ok(resp)
}

/// Generate a new token with specific permissions
pub fn generate_token(config: Config, permissions: &[String]) -> Result<Value> {
    let mut url = config.url;
    url.set_path("/api/v1/token");

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

    match open::that(url.as_str()) {
        Ok(()) => (),
        Err(err) => debug!("Failed to open URL automatically: {:}", err),
    };

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

/// List all known databses of the given type
pub fn list_dbs(config: Config, db_type: String) -> Result<HashMap<String, String>> {
    let mut url = config.url;
    url.set_path(format!("/api/v1/manifests/{db_type}").as_str());
    let cookie_store = get_cookie_store()?;

    let request = client(&cookie_store)?.get(url);
    let resp: HashMap<String, String> = maybe_add_auth(request, config.token).send()?.json()?;

    match resp.get("error") {
        None => Ok(resp),
        Some(err) => Err(Error::msg(err.to_string())),
    }
}

/// Check that a DB access token is valid, returning all possible
/// databases that it can be authenticate to.
pub fn check_db_token(config: &Config, token: &String) -> Result<HashMap<String, String>> {
    let mut url = config.url.clone();
    url.set_path(format!("/api/v1/auth/{token}").as_str());
    let cookie_store = get_cookie_store()?;
    let request = client(&cookie_store)?.get(url);
    let resp: HashMap<String, String> = maybe_add_auth(request, config.token.clone())
        .send()?
        .json()?;

    match resp.get("error") {
        None => Ok(resp),
        Some(err) => Err(Error::msg(err.to_string())),
    }
}

/// Approve a token for a DB authentication request, associating it with the currently
/// logged in user.
pub fn approve_db_authentication(config: &Config, token: &String, db_id: &String) -> Result<()> {
    let mut url = config.url.clone();
    url.set_path(format!("/api/v1/auth/{token}").as_str());
    let cookie_store = get_cookie_store()?;

    let mut body = HashMap::new();
    body.insert("manifest_id", db_id);

    let request = client(&cookie_store)?.put(url).json(&body);
    let resp: HashMap<String, String> = maybe_add_auth(request, config.token.clone())
        .send()?
        .json()?;

    match resp.get("error") {
        None => Ok(()),
        Some(err) => Err(Error::msg(err.to_string())),
    }
}

/// Retrieve information about a particular proxy client
pub fn client_get(config: Config, id: &String) -> Result<HashMap<String, Value>> {
    let mut url = config.url.clone();
    url.set_path(format!("/api/v1/client/{id}").as_str());
    let cookie_store = get_cookie_store()?;
    let request = client(&cookie_store)?.get(url);
    let resp: HashMap<String, Value> = maybe_add_auth(request, config.token.clone())
        .send()?
        .json()?;

    match resp.get("error") {
        None => Ok(resp),
        Some(err) => Err(Error::msg(err.to_string())),
    }
}

/// Generate an authentication token for a proxy client
pub fn client_token(config: &Config, id: &String) -> Result<ClientTokenData> {
    let mut url = config.url.clone();
    url.set_path(format!("/api/v1/client/{id}/token").as_str());
    let cookie_store = get_cookie_store()?;
    let request = client(&cookie_store)?.put(url);
    let resp: ClientTokenResponse = maybe_add_auth(request, config.token.clone())
        .send()?
        .json()?;

    match resp {
        ClientTokenResponse::Error(ApiError { error }) => Err(Error::msg(error.to_string())),
        ClientTokenResponse::Ok(data) => Ok(data),
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ClientTokenResponse {
    Error(ApiError),
    Ok(ClientTokenData),
}

#[derive(Deserialize)]
pub struct ApiError {
    pub error: String,
}

#[derive(Deserialize)]
pub struct ClientTokenData {
    pub id: String,
    pub token: String,
    pub manifest_id: String,
    pub protocol: String,
    pub port: u32,

    #[serde(default)]
    pub database: Option<String>,
}

fn read_code() -> Result<String> {
    let mut guess = String::new();

    std::io::stdin()
        .read_line(&mut guess)
        .map_err(|_| Error::msg("Failed to read line"))?;

    Ok(guess.trim().to_string())
}
