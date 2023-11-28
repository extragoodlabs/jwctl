use std::collections::HashMap;

use crate::config::{get_cookie_store, Config};
use crate::http::{client, maybe_add_auth};
use crate::manifests::{self, MANIFEST_API};

use anyhow::{Error, Result};
use base64::engine::general_purpose;
use serde_json::Value;

use clap::Subcommand;

use base64::Engine;
use inquire::{
    Confirm, CustomType, Editor, InquireError, Password, PasswordDisplayMode, Select, Text,
};
use serde::{Deserialize, Serialize};
use serde_json::{Error, Value};

#[derive(Clone, Debug, Subcommand)]
pub enum ProxySchemaCommands {
    /// Get all manifests
    All,

    /// Get information about a manifest
    #[command(arg_required_else_help = true)]
    Get {
        /// The ID of the manifest
        id: String,
    },

    /// Delete a manifest
    #[command(arg_required_else_help = true)]
    Delete {
        /// The ID of the manifest
        id: String,
    },

    /// Create a manifest
    Create,
}

const PROXY_SCHEMA_API: &str = "/proxy-schemas";

fn create_url(config: &Config, mainfest_id: String) -> url::Url {
    let full_url = format!("{}/{}/{}", MANIFEST_API, mainfest_id, PROXY_SCHEMA_API);

    let mut url = config.url.clone();

    url.set_path(full_url.as_str());
    url
}

fn get_list_manifests(config: Config) -> Result<Vec<HashMap<String, String>>> {
    let manifests = manifests::all(config);

    let list = match manifests {
        Ok(value) => value
            .as_array()
            .ok_or_else(|| Error::msg("could not get manifest"))?
            .to_owned(),
        Err(e) => return Err(e),
    };

    let names = list.map(|manifests| {
        manifests
            .iter()
            .map(|manifest| {
                let mut map = HashMap::new();

                map.insert(manifest.name.clone(), manifest.id.clone());
                map
            })
            .collect()
    });

    Ok(names)
}
