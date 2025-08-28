use crate::database::Poke;
use notify_rust::Notification;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn setup_notification(
    poke: &Poke,
    sched: &JobScheduler,
) -> Result<(), Box<dyn std::error::Error>> {
    // Clone the data needed for the notification
    let name = poke.name.clone();
    let detail = poke.detail.clone();

    // Setup notifications for the new job
    let job = Job::new(poke.cron.clone(), move |_uuid, _l| {
        let _ = Notification::new()
            .summary(&name)
            .body(detail.as_deref().unwrap_or(""))
            .appname("Poke Me")
            .icon("clock")
            .show();
    })?;

    sched.add(job).await?;
    Ok(())
}
