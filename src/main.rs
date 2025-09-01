use crate::database::{add_poke, get_poke_by_name, list_pokes, search_pokes_by_name};
use clap::{Parser, Subcommand};
use notify_rust::Notification;
use tokio_cron_scheduler::JobScheduler;
pub mod database;
pub mod notification;

// Display formatting constants
const NAME_WIDTH: usize = 20;
const CRON_WIDTH: usize = 20;
const DETAIL_WIDTH: usize = 40;
const CREATED_WIDTH: usize = 20;
const TOTAL_WIDTH: usize = NAME_WIDTH + CRON_WIDTH + DETAIL_WIDTH + CREATED_WIDTH;

/// Display a single job in the standard format
fn display_job(poke: &crate::database::Poke) {
    let detail = poke.detail.as_deref().unwrap_or("");
    let created = poke.created.format("%Y-%m-%d %H:%M");
    println!(
        "{:<NAME_WIDTH$} {:<CRON_WIDTH$} {:<DETAIL_WIDTH$} {:<CREATED_WIDTH$}",
        poke.name, poke.cron, detail, created
    );
}

/// Display the header for job listings
fn display_job_header() {
    println!(
        "{:<NAME_WIDTH$} {:<CRON_WIDTH$} {:<DETAIL_WIDTH$} {:<CREATED_WIDTH$}",
        "Name", "Cron", "Detail", "Created"
    );
    println!("{:-<TOTAL_WIDTH$}", "");
}

/// Display a list of jobs with optional title and count
fn display_jobs(pokes: &[crate::database::Poke], title: &str, show_count: bool) {
    if pokes.is_empty() {
        println!("{}", title);
        return;
    }

    if show_count {
        println!("{} ({} found):", title, pokes.len());
    } else {
        println!("{}", title);
    }

    display_job_header();
    for poke in pokes {
        display_job(poke);
    }
}

/// Display a single job in detail format
fn display_job_detail(poke: &crate::database::Poke) {
    println!("Job Details:");
    display_job_header();
    display_job(poke);
}

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
        Commands::List { head } => match list_pokes(&pool, head).await {
            Ok(all_pokes) => {
                if all_pokes.is_empty() {
                    println!("No jobs scheduled yet");
                } else {
                    display_jobs(&all_pokes, "Scheduled jobs:", false);
                }
            }
            Err(err) => println!("ERROR: {}", err),
        },
        Commands::Detail { name } => match get_poke_by_name(&pool, &name).await {
            Ok(poke) => {
                display_job_detail(&poke);
            }
            Err(err) => println!("ERROR: {}", err),
        },
        Commands::Search { term } => match search_pokes_by_name(&pool, &term).await {
            Ok(pokes) => {
                let title = format!("Jobs containing '{}'", term);
                display_jobs(&pokes, &title, true);
            }
            Err(err) => println!("ERROR: {}", err),
        },
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
