use crate::config::{get_cookie_store, Config};
use crate::http::{client, maybe_add_auth};

use anyhow::Result;
use base64::engine::general_purpose;
// use reqwest::blocking::RequestBuilder;
use serde_json::Value;

use clap::Subcommand;

use base64::Engine;
use inquire::{
    Confirm, CustomType, Editor, InquireError, Password, PasswordDisplayMode, Select, Text,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Subcommand)]
pub enum ManifestCommands {
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

// constant for the manifest API
const MANIFEST_API: &str = "/api/v1/manifests";

pub fn all(config: Config) -> Result<Value> {
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

// Enum for root_type which can be extended as needed
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum RootType {
    Postgresql,
    Mysql,
    Openapi,
}

// Struct for PostgreSQL configuration
#[derive(Serialize, Deserialize, Debug)]
pub struct PostgresqlConfig {
    #[serde(rename = "type")]
    pub type_field: String, // Since 'type' is a reserved keyword in Rust
    pub hostname: String,
    pub database: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssl: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_database: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_role: Option<String>,
}

// Struct for MySQL configuration
#[derive(Serialize, Deserialize, Debug)]
pub struct MysqlConfig {
    #[serde(rename = "type")]
    pub type_field: String,
    pub hostname: String,
    pub database: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssl: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_database: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_role: Option<String>,
}

// Struct for OpenAPI configuration
#[derive(Serialize, Deserialize, Debug)]
pub struct OpenapiConfig {
    pub url: String,
    pub http_schema: String,
    pub auth_type: String,
    pub schema: String,
}

// Enum to encapsulate different configuration types
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Configuration {
    Postgresql(PostgresqlConfig),
    Mysql(MysqlConfig),
    Openapi(OpenapiConfig),
}

// Struct for PostgreSQL credentials
#[derive(Serialize, Deserialize, Debug)]
pub struct PostgresqlCredentials {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_database: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_role: Option<String>,
}

// Struct for MySQL credentials
#[derive(Serialize, Deserialize, Debug)]
pub struct MysqlCredentials {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_database: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_role: Option<String>,
}

// Struct for OpenAPI credentials
#[derive(Serialize, Deserialize, Debug)]
pub struct OpenapiCredentials {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_field: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

// Enum to encapsulate different credential types
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Credentials {
    Postgresql(PostgresqlCredentials),
    Mysql(MysqlCredentials),
    Openapi(OpenapiCredentials),
}

// Main struct for a connection resource
#[derive(Serialize, Deserialize, Debug)]
pub struct NewManifest {
    pub name: String,
    pub root_type: RootType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<Configuration>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<Credentials>,
}

pub fn create(config: Config) -> Result<Value> {
    let manifest_result = prompt_user_for_manifest();

    manifest_result.and_then(|manifest| {
        let mut url = config.url;
        url.set_path(MANIFEST_API);

        let cookie_store = get_cookie_store()?;
        let request = client(&cookie_store)?.put(url).json(&manifest);

        let resp = maybe_add_auth(request, config.token).send()?.json()?;

        return Ok(resp);
    })
}

fn prompt_for_root_type() -> RootType {
    let options: Vec<&str> = vec!["PostgreSQL", "MySQL", "OpenAPI"];

    let ans: Result<&str, InquireError> =
        Select::new("Select the type manifest you want to create.", options).prompt();

    let choice = ans.unwrap();

    match choice {
        "PostgreSQL" => RootType::Postgresql,
        "MySQL" => RootType::Mysql,
        "OpenAPI" => RootType::Openapi,
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

    let port = CustomType::<u16>::new("What port number is PostgresSQL running on?")
        .with_error_message("Please type a valid number")
        .with_default(5432)
        .prompt()
        .unwrap();

    let ssl = Confirm::new("Are you using SSL?")
        .with_default(true)
        .prompt()
        .unwrap();

    let schema_str = Text::new("What is your PostgreSQL schema?")
        .with_placeholder("leave blank if not sure")
        .prompt();

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

    let mut config = PostgresqlConfig {
        type_field: "postgresql".to_string(),
        hostname,
        database,
        port: Some(port),
        ssl: Some(ssl),
        schema,
        vault_database: None,
        vault_role: None,
    };

    let is_using_vault = prompt_for_vault();

    if is_using_vault {
        let vault_database = Text::new("What Vault database are you using?")
            .prompt()
            .unwrap();

        let vault_role = Text::new("What Vault role are you using?")
            .prompt()
            .unwrap();

        config.vault_database = Some(vault_database);
        config.vault_role = Some(vault_role);
    }

    config
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

    let mut config = MysqlConfig {
        type_field: "mysql".to_string(),
        hostname,
        database,
        port: Some(port),
        ssl: Some(ssl),
        vault_database: None,
        vault_role: None,
    };

    let is_using_vault = prompt_for_vault();

    if is_using_vault {
        let vault_database = Text::new("What Vault database are you using?")
            .prompt()
            .unwrap();

        let vault_role = Text::new("What Vault role are you using?")
            .prompt()
            .unwrap();

        config.vault_database = Some(vault_database);
        config.vault_role = Some(vault_role);
    }

    config
}

// OpenAPI configuration prompt function
fn prompt_for_openapi_config() -> OpenapiConfig {
    let url = Text::new("What is your OpenAPI url?").prompt().unwrap();

    let http_schema = Select::new("What is your OpenAPI HTTP schema?", vec!["http", "https"])
        .prompt()
        .unwrap();

    let auth_type = Select::new(
        "What type of authentication are you using?",
        vec!["basic", "bearer", "none"],
    )
    .prompt()
    .unwrap();

    let schema = Editor::new("What is your OpenAPI schema?")
        .with_formatter(&|submission| {
            let char_count = submission.chars().count();
            if char_count <= 20 {
                submission.into()
            } else {
                let mut substr: String = submission.chars().take(17).collect();
                substr.push_str("...");
                substr
            }
        })
        .prompt()
        .unwrap();

    if schema.is_empty() {
        println!("You must provide a schema.");
        return prompt_for_openapi_config();
    }

    OpenapiConfig {
        url,
        http_schema: http_schema.to_string(),
        auth_type: auth_type.to_string(),
        schema,
    }
}

// Prompt for PostgreSQL credentials
fn prompt_for_postgresql_credentials(is_vault: bool) -> PostgresqlCredentials {
    match is_vault {
        true => {
            let vault_database = Text::new("What Vault database are you using?")
                .prompt()
                .unwrap();

            let vault_role = Text::new("What Vault role are you using?")
                .prompt()
                .unwrap();

            PostgresqlCredentials {
                username: None,
                password: None,
                vault_database: Some(vault_database),
                vault_role: Some(vault_role),
            }
        }
        false => {
            let username = Text::new("What is your PostgreSQL username?")
                .prompt()
                .unwrap();

            let password = Password::new("What is your PostgreSQL password?")
                .with_display_mode(PasswordDisplayMode::Masked)
                .prompt()
                .unwrap();

            PostgresqlCredentials {
                username: Some(username),
                password: Some(password),
                vault_database: None,
                vault_role: None,
            }
        }
    }
}

// Prompt for MySQL credentials
fn prompt_for_mysql_credentials(is_vault: bool) -> MysqlCredentials {
    match is_vault {
        true => {
            let vault_database = Text::new("What Vault database are you using?")
                .prompt()
                .unwrap();

            let vault_role = Password::new("What Vault role are you using?")
                .with_display_mode(PasswordDisplayMode::Masked)
                .prompt()
                .unwrap();

            MysqlCredentials {
                username: None,
                password: None,
                vault_database: Some(vault_database),
                vault_role: Some(vault_role),
            }
        }
        false => {
            let username = Text::new("What is your MySQL username?").prompt().unwrap();

            let password = Text::new("What is your MySQL password?").prompt().unwrap();

            MysqlCredentials {
                username: Some(username),
                password: Some(password),
                vault_database: None,
                vault_role: None,
            }
        }
    }
}

// Prompt for OpenAPI credentials
fn prompt_for_openapi_credentials() -> OpenapiCredentials {
    let auth_type = Select::new(
        "What auth type is your OpenAPI API using?",
        vec!["basic", "bearer", "none"],
    )
    .prompt()
    .unwrap();

    match auth_type {
        "basic" => {
            let username = Text::new("Enter a username").prompt().unwrap();
            let password = Password::new("Enter a password")
                .with_display_mode(PasswordDisplayMode::Masked)
                .prompt()
                .unwrap();

            let un_and_pw = format!("{}:{}", username, password);
            let encoded_credentials = general_purpose::STANDARD.encode(un_and_pw.as_bytes());

            OpenapiCredentials {
                type_field: Some("basic".to_owned()),
                token: Some(encoded_credentials),
            }
        }
        "bearer" => {
            let token = Text::new("Enter a Bearer token").prompt().unwrap();

            OpenapiCredentials {
                type_field: Some("bearer".to_owned()),
                token: Some(token),
            }
        }
        "none" | "" => OpenapiCredentials {
            type_field: None,
            token: None,
        },
        _ => panic!("Invalid option selected"),
    }
}

fn prompt_user_for_manifest() -> Result<NewManifest> {
    let name = CustomType::<String>::new("What is the name of your manifest?")
        .with_parser(&|input| {
            if input.is_empty() {
                return Err(());
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
        RootType::Postgresql => Some(Configuration::Postgresql(prompt_for_postgresql_config())),
        RootType::Mysql => Some(Configuration::Mysql(prompt_for_mysql_config())),
        RootType::Openapi => Some(Configuration::Openapi(prompt_for_openapi_config())),
    };

    let is_vault = match &configuration {
        Some(Configuration::Postgresql(config)) => config.vault_database.is_some(),
        Some(Configuration::Mysql(config)) => config.vault_database.is_some(),
        Some(Configuration::Openapi(_)) => false,
        None => false,
    };

    let credentials = match root_type {
        RootType::Postgresql => Some(Credentials::Postgresql(prompt_for_postgresql_credentials(
            is_vault,
        ))),
        RootType::Mysql => Some(Credentials::Mysql(prompt_for_mysql_credentials(is_vault))),
        RootType::Openapi => Some(Credentials::Openapi(prompt_for_openapi_credentials())),
    };

    let manifest = NewManifest {
        name,
        root_type,
        configuration,
        credentials,
    };

    Ok(manifest)
}
