mod command;
mod config;
mod terminal;

#[macro_use]
extern crate log;
extern crate config as config_rs;

use anyhow::{Error, Result};
use clap::{Parser, Subcommand, ValueEnum};
use log::{LevelFilter, SetLoggerError};
use serde_json::to_string_pretty;
use simplelog::TermLogger;

#[derive(Clone, Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,

    /// URL of the JumpWire proxy server
    #[arg(short, long)]
    url: Option<url::Url>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Enable timestamps in log lines
    #[arg(long)]
    timestamps: bool,

    /// Token to use for authenticating to the JumpWire API
    #[arg(short, long)]
    token: Option<String>,
}

#[derive(Clone, Debug, Subcommand)]
enum Commands {
    /// Check the current CLI configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Check the status of the proxy server
    Status,

    /// Run a simple ping against the proxy server
    Ping,

    /// Interact with bearer tokens used for authentication
    Token {
        #[command(subcommand)]
        command: TokenCommands,
    },

    /// Login or check authentication using a configured SSO provider
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },

    /// Commands to interact with a proxied database server
    Db {
        #[command(subcommand)]
        command: DbCommands,
    },

    /// Client authentication used to connect to a proxied database
    Client {
        #[command(subcommand)]
        command: ClientCommands,
    },
}

#[derive(Clone, Debug, Subcommand)]
enum ConfigCommands {
    /// Display the current configuration
    Get,
}

#[derive(Clone, Debug, Subcommand)]
enum TokenCommands {
    /// Store the authenticate token for future calls
    Set,

    /// Check permissions on the configured token
    Whoami,

    /// Generate a new authentication token
    ///
    /// Example: `jwctl token generate get:token get:status`
    #[command(arg_required_else_help = true)]
    Generate {
        /// Permissions are pairs of method:action specifying what the token is allowed to do.
        ///
        /// For example, retrieving the server's health information requires the permission `get:status`
        permissions: Vec<String>,
    },
}

#[derive(Clone, Debug, Subcommand)]
enum AuthCommands {
    /// Login with a specified SSO provider
    #[command(arg_required_else_help = true)]
    Login {
        /// The SSO identity provider
        provider: String,
    },

    /// List configured SSO providers
    List,

    /// Check the currently logged in user
    Whoami,
}

#[derive(Clone, Debug, Subcommand)]
enum DbCommands {
    /// List all databases of the given type
    #[command(arg_required_else_help = true)]
    List {
        /// The type of database
        #[arg(value_enum)]
        db_type: DatabaseType,
    },

    /// Approve an authentication request to a database
    #[command(arg_required_else_help = true)]
    Login {
        /// The token generated for the DB login
        token: String,
    },
}

#[derive(Clone, Debug, Subcommand)]
enum ClientCommands {
    /// Get information about a client
    #[command(arg_required_else_help = true)]
    Get {
        /// The ID of the client
        id: String,
    },

    /// Generate a token to use for proxy authentication
    #[command(arg_required_else_help = true)]
    Token {
        /// The ID of the client
        id: String,

        /// Print just the token without other log messages
        #[arg(short, long)]
        quiet: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, ValueEnum)]
enum DatabaseType {
    Postgresql,
    Mysql,
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl config_rs::Source for Args {
    fn clone_into_box(&self) -> Box<dyn config_rs::Source + Send + Sync> {
        Box::new((*self).clone())
    }

    fn collect(&self) -> Result<config_rs::Map<String, config_rs::Value>, config_rs::ConfigError> {
        let mut m = config_rs::Map::new();

        match &self.url {
            Some(url) => {
                let value = config_rs::ValueKind::String(url.to_string());
                m.insert("url".to_string(), value.into());
            }
            None => (),
        };

        match &self.token {
            Some(token) => {
                let value = config_rs::ValueKind::String(token.to_string());
                m.insert("token".to_string(), value.into());
            }
            None => (),
        };

        Ok(m)
    }
}

fn setup_logging(args: &Args) -> Result<(), SetLoggerError> {
    let log_level = if args.verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    let ts_log_level = if args.timestamps {
        LevelFilter::Error
    } else {
        LevelFilter::Trace
    };

    let config = simplelog::ConfigBuilder::new()
        .set_time_level(ts_log_level)
        .set_thread_level(LevelFilter::Trace)
        .set_target_level(LevelFilter::Trace)
        .build();

    TermLogger::init(
        log_level,
        config,
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
}

fn main() -> Result<()> {
    let args = Args::parse();

    setup_logging(&args)?;
    debug!("Debug logging enabled");

    let config = config::load_config(args.clone())?;

    match &args.command {
        Commands::Config { command } => match command {
            ConfigCommands::Get => command::config_get(config)?,
        },
        Commands::Status => {
            let resp = command::status(config)?;
            info!("Remote status:\n{}", to_string_pretty(&resp)?);
        }
        Commands::Ping => {
            let resp = command::ping(config)?;
            info!("Ping response: {:?}", resp);
        }
        Commands::Token { command } => match command {
            TokenCommands::Set => {
                command::authenticate(config)?;
                info!("Authentication token stored!");
            }
            TokenCommands::Whoami => {
                let resp = command::token_whoami(config)?;
                info!("whoami:\n{}", to_string_pretty(&resp)?);
            }
            TokenCommands::Generate { permissions } => {
                let resp = command::generate_token(config, permissions)?;
                info!("Token generated:\n{}", to_string_pretty(&resp)?);
            }
        },
        Commands::Auth { command } => match command {
            AuthCommands::List => {
                let resp = command::auth_list(config)?;
                info!(
                    "Configured SSO identity providers:\n{}",
                    to_string_pretty(&resp)?
                );
            }
            AuthCommands::Login { provider } => {
                let resp = command::auth_login(config, provider)?;
                match resp.get("error") {
                    Some(err) => error!("{}", to_string_pretty(&err)?),
                    _ => info!("Authenticated!"),
                };
            }
            AuthCommands::Whoami => {
                let resp = command::sso_whoami(config)?;
                info!("whoami:\n{}", to_string_pretty(&resp)?);
            }
        },
        Commands::Db { command } => match command {
            DbCommands::List { db_type } => {
                let dbs = command::list_dbs(config, db_type.to_string())?;
                println!("{:36} Name", "ID");
                dbs.iter()
                    .for_each(|(id, name)| println!("{:} {:}", id, name));
            }
            DbCommands::Login { token } => {
                let dbs = command::check_db_token(&config, token)?;
                let items: Vec<(&String, &String)> = dbs.iter().collect();
                if items.is_empty() {
                    error!("No matching databases!");
                    return Ok(());
                }

                info!("Choose a database to connect to:");

                // prepare an interactive terminal to let the user choose
                // which DB to authenticate to
                let mut term = terminal::setup_terminal()?;
                let (id, name) = terminal::run_list_selection(&mut term, items)?;
                terminal::restore_terminal(&mut term)?;

                debug!("Authenticating to database {:}", id);
                command::approve_db_authentication(&config, token, id)?;
                info!("Authentication request to {:} is approved!", name);
            }
        },
        Commands::Client { command } => match command {
            ClientCommands::Get { id } => {
                let resp = command::client_get(config, id)?;
                info!("Client information:\n{}", to_string_pretty(&resp)?);
            }
            ClientCommands::Token { id, quiet } => {
                let resp = command::client_token(config, id)?;
                let token = resp
                    .get("token")
                    .ok_or(Error::msg("Missing token in response"))?
                    .as_str()
                    .ok_or(Error::msg("Missing token in response"))?;
                if *quiet {
                    println!("{}", token);
                } else {
                    let manifest_id = resp
                        .get("manifest_id")
                        .ok_or(Error::msg("Missing manifest_id in response"))?
                        .as_str()
                        .ok_or(Error::msg("Missing manifest_id in response"))?;
                    info!(
                        "Token generated:\n\nusername: {}\npassword: {}",
                        manifest_id, token
                    );
                }
            }
        },
    };

    Ok(())
}
