use std::collections::HashMap;

use crate::config::{get_cookie_store, Config};
use crate::http::{client, maybe_add_auth};
use crate::manifests::{self, MANIFEST_API};

use anyhow::{Error, Result};

use inquire::{Confirm, CustomType, Select, Text};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const PROXY_SCHEMA_API: &str = "/proxy-schemas";

// Main struct for a connection resource
#[derive(Serialize, Deserialize, Debug)]
pub struct ProxySchema {
    // The UUID of the DB that this schema pertains to.
    pub manifest_id: String,

    // The name of the schema.
    pub name: String,

    // Mapping of field names to their label.
    // Fields without a label do not need to be explicitly configured.
    pub fields: HashMap<String, String>,
}

// -------------------- CLI Functions ------------------- //

pub fn list(config: &Config) -> Result<Value> {
    let manifest_id = select_manifest(config)?;

    let url = create_url(config, manifest_id);

    let cookie_store = get_cookie_store()?;
    let request = client(&cookie_store)?.get(url);

    let resp = maybe_add_auth(request, config.token.clone())
        .send()?
        .json()?;

    Ok(resp)
}

pub fn get_by_id(config: Config, id: String) -> Result<Value> {
    let manifest_id = select_manifest(&config)?;
    let mid = manifest_id.clone();

    let url = create_url(&config, mid);
    let full_url = format!("{}/{}", url, id);

    let cookie_store = get_cookie_store()?;
    let request = client(&cookie_store)?.get(full_url);

    let resp = maybe_add_auth(request, config.token).send()?.json()?;
    Ok(resp)
}

pub fn delete(config: Config, id: String) -> Result<Value> {
    let manifest_id = select_manifest(&config)?;
    let mid = manifest_id.clone();

    let url = create_url(&config, mid);
    let full_url = format!("{}/{}", url, id);

    let cookie_store = get_cookie_store()?;
    let request = client(&cookie_store)?.delete(full_url);

    let resp = maybe_add_auth(request, config.token).send()?.json()?;
    Ok(resp)
}

pub fn create(config: Config) -> Result<Value> {
    let manifest_id = select_manifest(&config)?;

    let name = prompt_for_name().unwrap();
    let fields_result = prompt_for_fields();

    fields_result.and_then(|fields| {
        let mid = manifest_id.clone();
        let proxy_schema = ProxySchema {
            name,
            manifest_id,
            fields,
        };

        let url = create_url(&config, mid);
        let cookie_store = get_cookie_store()?;
        let request = client(&cookie_store)?.post(url).json(&proxy_schema);

        let resp = maybe_add_auth(request, config.token).send()?.json()?;
        Ok(resp)
    })
}

// ------------------------------------------------------ //

// ------------------ Prompt Functions ------------------ //

fn select_manifest(config: &Config) -> Result<String> {
    let manifests = get_list_manifests(config)?;

    let mut keys = HashMap::new();

    let manifest_names: Vec<String> = manifests
        .iter()
        .map(|m| {
            let name = m.get("name").unwrap().as_str();
            let id = m.get("id").unwrap().as_str();

            let key = format!("{} ({})", name, id);
            keys.insert(key.clone(), id.to_string());
            key
        })
        .collect();

    let manifest_name = Select::new("Select a manifest", manifest_names)
        .prompt()?
        .to_string();

    keys.get(&manifest_name)
        .ok_or_else(|| Error::msg("Could not find manifest"))
        .cloned()
}

fn prompt_for_fields() -> Result<HashMap<String, String>> {
    let mut fields = HashMap::new();

    loop {
        let field_name = Text::new("Field name").prompt().unwrap();
        let field_label = Select::new("Field label", vec!["pii", "secret"])
            .prompt()
            .unwrap()
            .to_string();

        fields.insert(field_name, field_label);

        let add_another = Confirm::new("Add another field?")
            .with_default(true)
            .prompt()
            .unwrap();

        if !add_another {
            break;
        }
    }

    Ok(fields)
}

fn prompt_for_name() -> Result<String> {
    let name = CustomType::<String>::new("What is the name of your proxy-schema?")
        .with_parser(&|input| {
            if input.is_empty() {
                Err(())
            } else {
                let is_valid = input
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');

                if !is_valid {
                    return Err(());
                }

                Ok(input.to_string())
            }
        })
        .with_help_message("Schema names must be alphanumeric, underscores, or dashes")
        .with_error_message("Please use a valid name")
        .prompt()
        .unwrap();

    Ok(name)
}

// ------------------------------------------------------ //

fn create_url(config: &Config, mainfest_id: String) -> url::Url {
    let full_url = format!("{}/{}/{}", MANIFEST_API, mainfest_id, PROXY_SCHEMA_API);

    let mut url = config.url.clone();

    url.set_path(full_url.as_str());
    url
}

fn get_list_manifests(config: &Config) -> Result<Vec<HashMap<String, String>>> {
    let manifests = manifests::list(config);

    let list: Vec<Value> = match manifests {
        Ok(value) => value
            .as_array()
            .ok_or_else(|| Error::msg("could not get manifest"))?
            .to_owned(),
        Err(e) => return Err(e),
    };

    list.into_iter()
        .map(|m| {
            let obj = m
                .as_object()
                .ok_or_else(|| Error::msg("Value is not an object"))?;

            // Create a new HashMap for each object
            let mut map = HashMap::new();

            // Extract the "id" key and convert to String
            if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                map.insert("id".to_string(), id.to_string());
            } else {
                return Err(Error::msg("Key 'id' not found or is not a string"));
            }

            // Extract the "name" key and convert to String
            if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                map.insert("name".to_string(), name.to_string());
            } else {
                return Err(Error::msg("Key 'name' not found or is not a string"));
            }

            Ok(map)
        })
        .collect()
}
