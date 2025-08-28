use crate::database::{add_poke, get_poke_by_name, list_pokes};
use clap::{Parser, Subcommand};
use notify_rust::Notification;
use tokio_cron_scheduler::JobScheduler;
pub mod database;
pub mod notification;

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = "A service to setup recurring notifications"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the background service
    Service {
        #[arg(long, default_value = "false")]
        daemon: bool,
    },
    /// Add a new scheduled job
    Add {
        name: String,
        cron: String,
        detail: Option<String>,
    },
    List {
        #[arg(long)]
        head: Option<i32>,
    },
    Detail {
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sched = JobScheduler::new().await?;
    let pool = database::establish_connection().await?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { name, cron, detail } => {
            match add_poke(&pool, name, cron, detail, &sched).await {
                Ok(()) => println!("Job added successfully"),
                Err(err) => println!("ERROR: {}", err),
            }
        }
        Commands::List { head } => {
            match list_pokes(&pool, head).await {
                Ok(all_pokes) => {
                    if all_pokes.is_empty() {
                        println!("No jobs scheduled yet");
                    } else {
                        println!("Scheduled jobs:");
                        println!(
                            "{:<20} {:<20} {:<40} {:<20}",
                            "Name", "Cron", "Detail", "Created"
                        );
                        println!("{:-<100}", "");
                        for poke in all_pokes {
                            let detail = poke.detail.as_deref().unwrap_or("");
                            let created = poke.created.format("%Y-%m-%d %H:%M");
                            println!(
                                "{:<20} {:<20} {:<40} {:<20}",
                                poke.name, poke.cron, detail, created
                            );
                        }
                    }
                }
                Err(err) => println!("ERROR: {}", err),
            }
        }
        Commands::Detail { name } => {
            match get_poke_by_name(&pool, &name).await {
                Ok(poke) => {
                    let detail = poke.detail.as_deref().unwrap_or("");
                    let created = poke.created.format("%Y-%m-%d %H:%M");
                    println!("Job Details:");
                    println!("{:<20} {:<20} {:<40} {:<20}", "Name", "Cron", "Detail", "Created");
                    println!("{:-<100}", "");
                    println!(
                        "{:<20} {:<20} {:<40} {:<20}",
                        poke.name, poke.cron, detail, created
                    );
                }
                Err(err) => println!("ERROR: {}", err),
            }
        }
        _ => println!("unsupported"),
    }

    // Start the scheduler
    sched.start().await?;

    // Notify the scheduler started correctly
    let _ = Notification::new()
        .appname("Poke Me")
        .summary("Ready")
        .body("Scheduler initialized correctly")
        .icon("clock")
        .show();

    Ok(())
}
