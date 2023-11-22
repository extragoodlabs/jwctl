use anyhow::Result;
use reqwest::blocking::RequestBuilder;
use std::sync::Arc;

pub fn client(
    cookie_store: &Arc<reqwest_cookie_store::CookieStoreMutex>,
) -> Result<reqwest::blocking::Client> {
    let client = reqwest::blocking::ClientBuilder::new()
        .cookie_store(true)
        .cookie_provider(Arc::clone(cookie_store))
        .build()?;
    Ok(client)
}

pub fn maybe_add_auth(request: RequestBuilder, token: Option<String>) -> RequestBuilder {
    match token {
        Some(token) => request.bearer_auth(token),
        None => request,
    }
}
