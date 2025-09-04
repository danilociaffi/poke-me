use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Poke {
    pub id: i64,
    pub name: String,
    pub cron: String,
    pub detail: Option<String>,
    pub sound_enabled: bool,
    pub created: NaiveDateTime,
}

impl Poke {
    pub fn new<T: Into<String>>(
        name: T,
        cron: T,
        detail: Option<T>,
        sound_enabled: bool,
    ) -> Result<Self, String> {
        let cron_str = cron.into();
        let name_str = name.into();

        // Validate cron expression using tokio-cron-scheduler's format (6 fields: second, minute, hour, day, month, weekday)
        if !is_valid_cron(&cron_str) {
            return Err(format!("Invalid cron expression: {}. Expected format: 'second minute hour day month weekday'", cron_str));
        }

        Ok(Poke {
            id: 0, // Will be set by database
            name: name_str,
            cron: cron_str,
            detail: detail.map(|d| d.into()),
            sound_enabled,
            created: Utc::now().naive_local(),
        })
    }
}

fn is_valid_cron(cron: &str) -> bool {
    // Use tokio_cron_scheduler's Job::new to validate the cron expression
    // This ensures compatibility with the scheduler that will actually use it
    tokio_cron_scheduler::Job::new(cron, |_, _| {}).is_ok()
}
