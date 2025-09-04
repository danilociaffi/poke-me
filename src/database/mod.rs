pub use models::Poke;
pub mod models;
use crate::notification::setup_notification;
use sqlx::{sqlite::SqlitePool, Row};
use std::path::Path;
use tokio_cron_scheduler::JobScheduler;

/// Check if we're running in development mode (cargo run) vs production mode (installed binary)
fn is_development_mode() -> bool {
    // Check if the executable path contains "target" (indicating cargo run)
    if let Ok(exe_path) = std::env::current_exe() {
        exe_path.to_string_lossy().contains("target")
    } else {
        false
    }
}

pub async fn establish_connection() -> Result<SqlitePool, sqlx::Error> {
    // Determine database path based on how the binary is being run
    let db_path = if is_development_mode() {
        // Development mode (cargo run): use current directory
        "poke.db".to_string()
    } else {
        // Production mode (installed binary): use systemd service location
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let db_dir = Path::new(&home_dir).join(".local/share/poke_me");

        // Create the directory if it doesn't exist
        if !db_dir.exists() {
            std::fs::create_dir_all(&db_dir).expect("Failed to create poke_me data directory");
        }

        db_dir.join("poke.db").to_string_lossy().to_string()
    };

    // Create database file if it doesn't exist
    if !Path::new(&db_path).exists() {
        // Create empty database file
        std::fs::File::create(&db_path).expect("Failed to create database file");
    }

    // Connect to the database
    let pool = SqlitePool::connect(&format!("sqlite:{}", db_path)).await?;

    // Run migrations
    run_migrations(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Create the poke table if it doesn't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS poke (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            cron TEXT NOT NULL,
            detail TEXT,
            sound_enabled BOOLEAN NOT NULL DEFAULT 0,
            created TIMESTAMP NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn add_poke<T>(
    pool: &SqlitePool,
    name: T,
    cron: T,
    detail: Option<T>,
    sound_enabled: bool,
    sched: &JobScheduler,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Into<String>,
{
    let poke = Poke::new(name, cron, detail, sound_enabled)?;

    // Start a transaction
    let mut tx = pool.begin().await?;

    // Check if name already exists
    let existing = sqlx::query("SELECT COUNT(*) FROM poke WHERE name = ?")
        .bind(&poke.name)
        .fetch_one(&mut *tx)
        .await?;

    let count: i64 = existing.get(0);
    if count > 0 {
        return Err(format!("A job with name '{}' already exists", poke.name).into());
    }

    // Insert the job
    let _result = sqlx::query(
        "INSERT INTO poke (name, cron, detail, sound_enabled, created) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&poke.name)
    .bind(&poke.cron)
    .bind(&poke.detail)
    .bind(&poke.sound_enabled)
    .bind(&poke.created)
    .execute(&mut *tx)
    .await?;

    // Set up notification
    match setup_notification(&poke, sched).await {
        Ok(_) => {
            tx.commit().await?;
            Ok(())
        }
        Err(err) => {
            tx.rollback().await?;
            Err(err)
        }
    }
}

pub async fn list_pokes(pool: &SqlitePool, head: Option<i32>) -> Result<Vec<Poke>, sqlx::Error> {
    let limit_clause = if let Some(limit) = head {
        format!(" LIMIT {}", limit)
    } else {
        String::new()
    };

    let query = format!("SELECT * FROM poke ORDER BY created DESC{}", limit_clause);
    sqlx::query_as::<_, Poke>(&query).fetch_all(pool).await
}

pub async fn get_poke_by_name(
    pool: &SqlitePool,
    name: &str,
) -> Result<Poke, Box<dyn std::error::Error>> {
    let poke = sqlx::query_as::<_, Poke>("SELECT * FROM poke WHERE name = ?")
        .bind(name)
        .fetch_optional(pool)
        .await?;

    poke.ok_or_else(|| format!("No job found with name: {}", name).into())
}

pub async fn search_pokes_by_name(
    pool: &SqlitePool,
    search_term: &str,
) -> Result<Vec<Poke>, sqlx::Error> {
    let search_pattern = format!("%{}%", search_term);
    sqlx::query_as::<_, Poke>("SELECT * FROM poke WHERE name LIKE ? ORDER BY created DESC")
        .bind(search_pattern)
        .fetch_all(pool)
        .await
}

/// Remove a job by name
pub async fn remove_poke(pool: &SqlitePool, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the job exists first
    let existing = sqlx::query("SELECT COUNT(*) FROM poke WHERE name = ?")
        .bind(name)
        .fetch_one(pool)
        .await?;

    let count: i64 = existing.get(0);
    if count == 0 {
        return Err(format!("No job found with name '{}'", name).into());
    }

    // Delete the job
    let _result = sqlx::query("DELETE FROM poke WHERE name = ?")
        .bind(name)
        .execute(pool)
        .await?;

    Ok(())
}

/// Toggle sound on/off for an existing job
pub async fn toggle_poke_sound(
    pool: &SqlitePool,
    name: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Check if the job exists first
    let existing = sqlx::query("SELECT sound_enabled FROM poke WHERE name = ?")
        .bind(name)
        .fetch_optional(pool)
        .await?;

    let current_sound = existing.ok_or_else(|| format!("No job found with name '{}'", name))?;
    let current_sound_enabled: bool = current_sound.get(0);

    // Toggle the sound setting
    let new_sound_enabled = !current_sound_enabled;

    // Update the job
    let _result = sqlx::query("UPDATE poke SET sound_enabled = ? WHERE name = ?")
        .bind(new_sound_enabled)
        .bind(name)
        .execute(pool)
        .await?;

    Ok(new_sound_enabled)
}
