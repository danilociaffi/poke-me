use crate::database::Poke;

// Display formatting constants
pub const NAME_WIDTH: usize = 20;
pub const CRON_WIDTH: usize = 20;
pub const DETAIL_WIDTH: usize = 40;
pub const CREATED_WIDTH: usize = 20;
pub const TOTAL_WIDTH: usize = NAME_WIDTH + CRON_WIDTH + DETAIL_WIDTH + CREATED_WIDTH;

/// Display a single job in the standard format
pub fn display_job(poke: &Poke) {
    let detail = poke.detail.as_deref().unwrap_or("");
    let created = poke.created.format("%Y-%m-%d %H:%M");
    println!(
        "{:<NAME_WIDTH$} {:<CRON_WIDTH$} {:<DETAIL_WIDTH$} {:<CREATED_WIDTH$}",
        poke.name, poke.cron, detail, created
    );
}

/// Display the header for job listings
pub fn display_job_header() {
    println!(
        "{:<NAME_WIDTH$} {:<CRON_WIDTH$} {:<DETAIL_WIDTH$} {:<CREATED_WIDTH$}",
        "Name", "Cron", "Detail", "Created"
    );
    println!("{:-<TOTAL_WIDTH$}", "");
}

/// Display a list of jobs with optional title and count
pub fn display_jobs(pokes: &[Poke], title: &str, show_count: bool) {
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
pub fn display_job_detail(poke: &Poke) {
    println!("Job Details:");
    display_job_header();
    display_job(poke);
}
