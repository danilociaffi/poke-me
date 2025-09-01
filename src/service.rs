use crate::{
    database::{establish_connection, list_pokes},
    notification::setup_notification,
};
use notify_rust::Notification;
use tokio_cron_scheduler::JobScheduler;

/// Run the background notification service
pub async fn run_service(daemon: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Poke Me notification service...");

    // Establish database connection
    let pool = establish_connection().await?;

    // Create and start the job scheduler
    let sched = JobScheduler::new().await?;

    // Load existing jobs from database and set them up
    let existing_jobs = list_pokes(&pool, None).await?;
    println!("Found {} existing scheduled jobs", existing_jobs.len());

    for poke in &existing_jobs {
        match setup_notification(&poke, &sched).await {
            Ok(()) => println!("Loaded job: {}", poke.name),
            Err(err) => eprintln!("Failed to load job {}: {}", poke.name, err),
        }
    }

    // Start the scheduler
    sched.start().await?;
    println!("Scheduler started successfully");

    // Show initial notification
    let _ = Notification::new()
        .appname("Poke Me")
        .summary("Service Started")
        .body(&format!(
            "Notification service running with {} jobs",
            existing_jobs.len()
        ))
        .icon("clock")
        .show();

    if daemon {
        println!("Running in daemon mode. Service will continue in background.");
        println!("Use Ctrl+C to stop the service.");
    } else {
        println!("Service running. Press Ctrl+C to stop.");
    }

    // Keep the service running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

        // Optional: periodic health check
        if let Err(err) = pool.acquire().await {
            eprintln!("Database connection error: {}", err);
        }
    }
}
