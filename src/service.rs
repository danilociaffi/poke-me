use crate::{
    database::{establish_connection, list_pokes},
    notification::setup_notification,
};
use notify_rust::Notification;
use std::fs;
use std::path::Path;
use tokio_cron_scheduler::JobScheduler;

const PID_FILE: &str = "/tmp/poke_me.pid";
const CONTROL_FILE: &str = "/tmp/poke_me.control";
const REFRESH_FILE: &str = "/tmp/poke_me.refresh";

/// Run the background notification service
pub async fn run_service(daemon: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Poke Me notification service...");

    // Create PID file
    let pid = std::process::id();
    fs::write(PID_FILE, pid.to_string())?;
    println!("Service PID: {}", pid);

    // Create control file for graceful shutdown
    fs::write(CONTROL_FILE, "running")?;

    // Establish database connection
    let pool = establish_connection().await?;

    // Create and start the job scheduler
    let mut sched = JobScheduler::new().await?;

    // Load existing jobs from database and set them up
    load_jobs_into_scheduler(&pool, &mut sched).await?;

    // Start the scheduler
    sched.start().await?;
    println!("Scheduler started successfully");

    // Show initial notification
    let _ = Notification::new()
        .appname("Poke Me")
        .summary("Service Started")
        .body(&format!(
            "Notification service running with {} jobs",
            list_pokes(&pool, None).await?.len()
        ))
        .icon("clock")
        .show();

    if daemon {
        println!("Running in daemon mode. Service will continue in background.");
        println!("Use 'poke_me stop' to stop the service.");
    } else {
        println!(
            "Service running. Press Ctrl+C to stop or use 'poke_me stop' from another terminal."
        );
    }

    // Keep the service running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Check if we should stop
        if !Path::new(CONTROL_FILE).exists() {
            println!("Control file removed, shutting down gracefully...");
            break;
        }

        // Check if we should refresh jobs
        if Path::new(REFRESH_FILE).exists() {
            println!("Refresh file detected, reloading jobs...");
            fs::remove_file(REFRESH_FILE)?;

            // Clear existing jobs and reload
            sched.shutdown().await?;
            sched = JobScheduler::new().await?;
            load_jobs_into_scheduler(&pool, &mut sched).await?;
            sched.start().await?;
            println!("Jobs refreshed successfully");
        }

        // Optional: periodic health check
        if let Err(err) = pool.acquire().await {
            eprintln!("Database connection error: {}", err);
        }
    }

    // Cleanup
    cleanup_service_files()?;
    Ok(())
}

/// Load jobs from database into the scheduler
async fn load_jobs_into_scheduler(
    pool: &sqlx::SqlitePool,
    sched: &mut JobScheduler,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load existing jobs from database and set them up
    let existing_jobs = list_pokes(pool, None).await?;
    println!("Found {} existing scheduled jobs", existing_jobs.len());

    for poke in &existing_jobs {
        match setup_notification(&poke, sched).await {
            Ok(()) => println!("Loaded job: {}", poke.name),
            Err(err) => eprintln!("Failed to load job {}: {}", poke.name, err),
        }
    }

    Ok(())
}

/// Signal the service to refresh its jobs
pub fn signal_refresh() -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(PID_FILE).exists() {
        return Err("Service is not running".into());
    }

    // Create refresh file to signal the service
    fs::write(REFRESH_FILE, "refresh")?;
    println!("Refresh signal sent to service");
    Ok(())
}

/// Stop the running service
pub fn stop_service() -> Result<(), Box<dyn std::error::Error>> {
    // Check if service is running
    if !Path::new(PID_FILE).exists() {
        return Err("Service is not running".into());
    }

    // Read PID from file
    let pid_content = fs::read_to_string(PID_FILE)?;
    let pid: u32 = pid_content.trim().parse()?;

    // Check if process is still running
    if !is_process_running(pid) {
        cleanup_service_files()?;
        return Err("Service is not running (PID file was stale)".into());
    }

    // Remove control file to signal graceful shutdown
    if Path::new(CONTROL_FILE).exists() {
        fs::remove_file(CONTROL_FILE)?;
        println!("Stopping service (PID: {})...", pid);

        // Wait a bit for graceful shutdown
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Force kill if still running
        if is_process_running(pid) {
            println!("Force killing service...");
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
        }
    }

    cleanup_service_files()?;
    println!("Service stopped successfully");
    Ok(())
}

/// Check if a process is running
fn is_process_running(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

/// Clean up service files
fn cleanup_service_files() -> Result<(), Box<dyn std::error::Error>> {
    if Path::new(PID_FILE).exists() {
        fs::remove_file(PID_FILE)?;
    }
    if Path::new(CONTROL_FILE).exists() {
        fs::remove_file(CONTROL_FILE)?;
    }
    if Path::new(REFRESH_FILE).exists() {
        fs::remove_file(REFRESH_FILE)?;
    }
    Ok(())
}
