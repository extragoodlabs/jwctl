use crate::config::{get_cookie_store, Config};
use crate::http::{client, maybe_add_auth};

use anyhow::Result;
use serde_json::Value;

use inquire::{Confirm, CustomType, InquireError, Password, PasswordDisplayMode, Select, Text};
use serde::{Deserialize, Serialize};

// constant for the manifest API
const MANIFEST_API: &str = "/api/v1/manifests";

// Enum for root_type which can be extended as needed
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum RootType {
    Postgresql,
    Mysql,
}

// Struct for PostgreSQL configuration
#[derive(Serialize, Deserialize, Debug)]
pub struct PostgresqlConfig {
    #[serde(rename = "type")]
    pub type_field: RootType,
    pub hostname: String,
    pub database: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssl: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
}

// Struct for MySQL configuration
#[derive(Serialize, Deserialize, Debug)]
pub struct MysqlConfig {
    #[serde(rename = "type")]
    pub type_field: RootType,
    pub hostname: String,
    pub database: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssl: Option<bool>,
}

// Enum to encapsulate different configuration types
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Configuration {
    Postgresql(PostgresqlConfig),
    Mysql(MysqlConfig),
}

// Struct for Vault credentials
#[derive(Serialize, Deserialize, Debug)]
pub struct VaultCredentials {
    pub database: String,
    pub role: String,
}

// Struct for PostgreSQL credentials
#[derive(Serialize, Deserialize, Debug)]
pub struct PostgresqlCredentials {
    pub username: String,
    pub password: String,
}

// Struct for MySQL credentials
#[derive(Serialize, Deserialize, Debug)]
pub struct MysqlCredentials {
    pub username: String,
    pub password: String,
}

// Enum to encapsulate different credential types
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Credentials {
    Postgresql(PostgresqlCredentials),
    Mysql(MysqlCredentials),
    Vault(VaultCredentials),
}

// Main struct for a connection resource
#[derive(Serialize, Deserialize, Debug)]
pub struct NewManifest {
    pub name: String,
    pub root_type: RootType,
    pub configuration: Configuration,
    pub credentials: Credentials,
}

// ------------------ CLI Functions ------------------ //

pub fn list(config: Config) -> Result<Value> {
    let mut url = config.url;
    url.set_path(MANIFEST_API);

    let cookie_store = get_cookie_store()?;
    let request = client(&cookie_store)?.get(url);

    let resp = maybe_add_auth(request, config.token).send()?.json()?;
    Ok(resp)
}

pub fn get_by_id(config: Config, id: String) -> Result<Value> {
    let full_url = format!("{}/{}", MANIFEST_API, id);

    let mut url = config.url;
    url.set_path(full_url.as_str());

    let cookie_store = get_cookie_store()?;
    let request = client(&cookie_store)?.get(url);

    let resp = maybe_add_auth(request, config.token).send()?.json()?;
    Ok(resp)
}

pub fn delete(config: Config, id: String) -> Result<Value> {
    let full_url = format!("{}/{}", MANIFEST_API, id);

    let mut url = config.url;
    url.set_path(full_url.as_str());

    let cookie_store = get_cookie_store()?;
    let request = client(&cookie_store)?.delete(url);

    let resp = maybe_add_auth(request, config.token).send()?.json()?;
    Ok(resp)
}

pub fn create(config: Config) -> Result<Value> {
    let manifest_result = prompt_user_for_manifest();

    manifest_result.and_then(|manifest| {
        let mut url = config.url;
        url.set_path(MANIFEST_API);

        let cookie_store = get_cookie_store()?;
        let request = client(&cookie_store)?.put(url).json(&manifest);

        let resp = maybe_add_auth(request, config.token).send()?.json()?;

        Ok(resp)
    })
}

// ------------------------------------------------------ //

// ------------------ Prompt Functions ------------------ //

fn prompt_for_root_type() -> RootType {
    let options: Vec<&str> = vec!["PostgreSQL", "MySQL"];

    let ans: Result<&str, InquireError> =
        Select::new("Select the type manifest you want to create.", options).prompt();

    let choice = ans.unwrap();

    match choice {
        "PostgreSQL" => RootType::Postgresql,
        "MySQL" => RootType::Mysql,
        _ => panic!("Invalid option selected"),
    }
}

fn prompt_for_vault() -> bool {
    let ans = Confirm::new("Are you using Vault to generate your credentials?")
        .with_default(false)
        .with_help_message("If you're not sure, select 'no'")
        .prompt();

    ans.map_err(|_e| false).unwrap()
}

// PostgreSQL configuration prompt function
fn prompt_for_postgresql_config() -> PostgresqlConfig {
    let hostname = Text::new("What is your PostgreSQL hostname?")
        .prompt()
        .unwrap();

    let database = Text::new("What is your PostgreSQL database name?")
        .prompt()
        .unwrap();

    let schema_str = Text::new("What is your PostgreSQL schema?")
        .with_default("public")
        .prompt();

    let port = CustomType::<u16>::new("What port number is PostgresSQL running on?")
        .with_error_message("Please type a valid number")
        .with_default(5432)
        .prompt()
        .unwrap();

    let ssl = Confirm::new("Are you using SSL?")
        .with_default(true)
        .prompt()
        .unwrap();

    let schema = match schema_str {
        Ok(schema) => {
            if schema.is_empty() {
                None
            } else {
                Some(schema)
            }
        }
        Err(_) => None,
    };

    PostgresqlConfig {
        type_field: RootType::Postgresql,
        hostname,
        database,
        port: Some(port),
        ssl: Some(ssl),
        schema,
    }
}

// MySQL configuration prompt function
fn prompt_for_mysql_config() -> MysqlConfig {
    let hostname = Text::new("What is your MySQL hostname?").prompt().unwrap();

    let database = Text::new("What is your MySQL database name?")
        .prompt()
        .unwrap();

    let port = CustomType::<u16>::new("What port number is MySQL running on?")
        .with_error_message("Please type a valid number")
        .with_default(3306)
        .prompt()
        .unwrap();

    let ssl = Confirm::new("Are you using SSL?")
        .with_default(true)
        .prompt()
        .unwrap();

    MysqlConfig {
        type_field: RootType::Mysql,
        hostname,
        database,
        port: Some(port),
        ssl: Some(ssl),
    }
}

fn prompt_for_vault_credentials() -> VaultCredentials {
    let database = Text::new("What Vault database are you using?")
        .prompt()
        .unwrap();

    let role = Text::new("What Vault role are you using?")
        .prompt()
        .unwrap();

    VaultCredentials { database, role }
}

// Prompt for PostgreSQL credentials
fn prompt_for_postgresql_credentials() -> PostgresqlCredentials {
    let username = Text::new("What is your PostgreSQL username?")
        .prompt()
        .unwrap();

    let password = Password::new("What is your PostgreSQL password?")
        .with_display_mode(PasswordDisplayMode::Masked)
        .prompt()
        .unwrap();

    PostgresqlCredentials {
        username: username,
        password: password,
    }
}

// Prompt for MySQL credentials
fn prompt_for_mysql_credentials() -> MysqlCredentials {
    let username = Text::new("What is your MySQL username?").prompt().unwrap();

    let password = Text::new("What is your MySQL password?").prompt().unwrap();

    MysqlCredentials {
        username: username,
        password: password,
    }
}

fn prompt_user_for_manifest() -> Result<NewManifest> {
    let name = CustomType::<String>::new("What is the name of your manifest?")
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
        .with_help_message("Manifest names must be alphanumeric, underscores, or dashes")
        .with_error_message("Please use a valid name")
        .prompt()
        .unwrap();

    let root_type = prompt_for_root_type();

    let configuration = match root_type {
        RootType::Postgresql => Configuration::Postgresql(prompt_for_postgresql_config()),
        RootType::Mysql => Configuration::Mysql(prompt_for_mysql_config()),
    };

    let is_vault = prompt_for_vault();
    let credentials = match is_vault {
        true => Credentials::Vault(prompt_for_vault_credentials()),
        false => match root_type {
            RootType::Postgresql => Credentials::Postgresql(prompt_for_postgresql_credentials()),
            RootType::Mysql => Credentials::Mysql(prompt_for_mysql_credentials()),
        },
    };

    let manifest = NewManifest {
        name,
        root_type,
        configuration,
        credentials,
    };

    Ok(manifest)
}

// ------------------------------------------------------ //
