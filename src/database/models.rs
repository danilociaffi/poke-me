use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::str::FromStr;

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

        // Validate cron expression
        if !is_valid_cron(&cron_str) {
            return Err(format!("Invalid cron expression: {}", cron_str));
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
    cron::Schedule::from_str(cron).is_ok()
}
