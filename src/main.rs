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
    /// Print the current CLI config
    Config,

    /// Check the status of the proxy server
    Status,

    /// Run a simple ping against the proxy server
    Ping,

    /// Store the authenticate token for future calls
    Authenticate,
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
        Commands::Config => info!("Current configuration:\n{:#?}", config),
        Commands::Status => {
            let resp = command::status(config)?;
            info!("Remote status:\n{}", to_string_pretty(&resp)?);
        }
        Commands::Ping => {
            let resp = command::ping(config)?;
            info!("Ping response: {:?}", resp);
        }
        Commands::Authenticate => {
            command::authenticate(config)?;
            info!("Authentication token stored!");
        }
    };

    Ok(())
}
