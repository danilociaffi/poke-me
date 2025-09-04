use crate::database::Poke;
use log::{debug, error, info};
use notify_rust::Notification;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn setup_notification(
    poke: &Poke,
    sched: &JobScheduler,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!(
        "Setting up notification for job: '{}' with cron: '{}'",
        poke.name, poke.cron
    );

    // Clone the data needed for the notification
    let name = poke.name.clone();
    let detail = poke.detail.clone();
    let sound_enabled = poke.sound_enabled;

    // Setup notifications for the new job
    let job = Job::new(poke.cron.clone(), move |_uuid, _l| {
        info!("Job '{}' executing notification", name);

        let mut notification = Notification::new();
        notification
            .summary(&name)
            .body(detail.as_deref().unwrap_or(""))
            .appname("Poke Me")
            .icon("clock")
            .hint(notify_rust::Hint::Urgency(notify_rust::Urgency::Normal));

        // Only add sound if enabled for this job
        if sound_enabled {
            notification.hint(notify_rust::Hint::SoundName("message-new-instant".into()));
        }

        let result = notification.show();
        match result {
            Ok(_) => info!("Notification for '{}' displayed successfully", name),
            Err(e) => error!("Failed to display notification for '{}': {}", name, e),
        }
    })?;

    debug!(
        "Job '{}' created successfully, adding to scheduler",
        poke.name
    );
    sched.add(job).await?;
    debug!("Job '{}' added to scheduler successfully", poke.name);
    Ok(())
}
