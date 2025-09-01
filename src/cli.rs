use crate::{
    database::{add_poke, get_poke_by_name, list_pokes, remove_poke, search_pokes_by_name},
    display::{display_job_detail, display_jobs},
};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = "A service to setup recurring notifications"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the background notification service
    Service {
        /// Run the service as a daemon process
        #[arg(long, default_value = "false")]
        daemon: bool,
    },
    /// Add a new scheduled notification job
    Add {
        /// Unique name for the job
        name: String,
        /// Cron expression (format: "sec min hour day month weekday")
        cron: String,
        /// Optional description or message for the notification
        detail: Option<String>,
    },
    /// List all scheduled notification jobs
    List {
        /// Limit the number of jobs to display
        #[arg(long)]
        head: Option<i32>,
    },
    /// Show detailed information for a specific job by exact name
    Detail {
        /// Exact name of the job to display
        name: String,
    },
    /// Search for jobs by name pattern (partial matching)
    Search {
        /// Search term to match against job names
        term: String,
    },
    /// Remove a scheduled job by name
    Remove {
        /// Name of the job to remove
        name: String,
    },
}

pub async fn handle_commands(
    command: Commands,
    pool: &sqlx::SqlitePool,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Add { name, cron, detail } => {
            let sched = tokio_cron_scheduler::JobScheduler::new().await?;
            match add_poke(pool, name, cron, detail, &sched).await {
                Ok(()) => println!("Job added successfully"),
                Err(err) => println!("ERROR: {}", err),
            }
        }
        Commands::List { head } => match list_pokes(pool, head).await {
            Ok(all_pokes) => {
                if all_pokes.is_empty() {
                    println!("No jobs scheduled yet");
                } else {
                    display_jobs(&all_pokes, "Scheduled jobs:", false);
                }
            }
            Err(err) => println!("ERROR: {}", err),
        },
        Commands::Detail { name } => match get_poke_by_name(pool, &name).await {
            Ok(poke) => {
                display_job_detail(&poke);
            }
            Err(err) => println!("ERROR: {}", err),
        },
        Commands::Search { term } => match search_pokes_by_name(pool, &term).await {
            Ok(pokes) => {
                let title = format!("Jobs containing '{}'", term);
                display_jobs(&pokes, &title, true);
            }
            Err(err) => println!("ERROR: {}", err),
        },
        Commands::Remove { name } => match remove_poke(pool, &name).await {
            Ok(()) => println!("Job '{}' removed successfully", name),
            Err(err) => println!("ERROR: {}", err),
        },
        Commands::Service { .. } => {
            // Service command is handled separately in main.rs
            unreachable!("Service command should be handled in main.rs");
        }
    }

    Ok(())
}
