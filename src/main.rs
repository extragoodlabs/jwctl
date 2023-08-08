mod command;
mod config;

#[macro_use]
extern crate log;
extern crate config as config_rs;

use anyhow::Result;
use clap::{Parser, Subcommand};
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
                let resp = command::whoami(config)?;
                info!("whoami:\n{}", to_string_pretty(&resp)?);
            }
            TokenCommands::Generate { permissions } => {
                let resp = command::generate_token(config, permissions)?;
                info!("Token generated:\n{}", to_string_pretty(&resp)?);
            }
        },
    };

    Ok(())
}
