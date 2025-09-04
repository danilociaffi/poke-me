mod cli;
mod database;
mod display;
mod notification;
mod service;

use clap::Parser;
use cli::{handle_commands, Cli};
use service::run_service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .filter_module("sqlx", log::LevelFilter::Warn) // Only show sqlx warnings/errors
        .init();

    let cli = Cli::parse();

    match cli.command {
        cli::Commands::Service { daemon } => {
            run_service(daemon).await?;
        }
        command => {
            // Handle all other commands
            let pool = database::establish_connection().await?;
            handle_commands(command, &pool).await?;
        }
    }

    Ok(())
}
