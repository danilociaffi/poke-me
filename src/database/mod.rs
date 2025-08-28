pub use models::Poke;
pub mod models;
use sqlx::{sqlite::SqlitePool, Row};
use crate::notification::setup_notification;
use tokio_cron_scheduler::JobScheduler;

pub async fn establish_connection() -> Result<SqlitePool, sqlx::Error> {
    SqlitePool::connect("sqlite:poke_me.db").await
}

pub async fn add_poke<T>(
    pool: &SqlitePool,
    name: T,
    cron: T,
    detail: Option<T>,
    sched: &JobScheduler,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Into<String>,
{
    let poke = Poke::new(name, cron, detail)?;
    
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
        "INSERT INTO poke (name, cron, detail, created) VALUES (?, ?, ?, ?)"
    )
    .bind(&poke.name)
    .bind(&poke.cron)
    .bind(&poke.detail)
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
    sqlx::query_as::<_, Poke>(&query)
        .fetch_all(pool)
        .await
}

pub async fn get_poke_by_name(pool: &SqlitePool, name: &str) -> Result<Poke, Box<dyn std::error::Error>> {
    let poke = sqlx::query_as::<_, Poke>("SELECT * FROM poke WHERE name = ?")
        .bind(name)
        .fetch_optional(pool)
        .await?;
    
    poke.ok_or_else(|| format!("No job found with name: {}", name).into())
}
