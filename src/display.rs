use crate::database::Poke;

// Display formatting constants
pub const NAME_WIDTH: usize = 20;
pub const CRON_WIDTH: usize = 20;
pub const DETAIL_WIDTH: usize = 40;
pub const SOUND_WIDTH: usize = 8;
pub const CREATED_WIDTH: usize = 20;
pub const TOTAL_WIDTH: usize = NAME_WIDTH + CRON_WIDTH + DETAIL_WIDTH + SOUND_WIDTH + CREATED_WIDTH;

/// Wrap text to fit within a specified width
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if text.len() <= width {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.len() + word.len() + 1 <= width {
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        } else {
            if !current_line.is_empty() {
                lines.push(current_line.clone());
            }
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Display a single job in the standard format with text wrapping
pub fn display_job(poke: &Poke) {
    let detail = poke.detail.as_deref().unwrap_or("");
    let created = poke.created.format("%Y-%m-%d %H:%M");
    let sound_status = if poke.sound_enabled { "ON" } else { "OFF" };

    // Wrap the detail text
    let detail_lines = wrap_text(detail, DETAIL_WIDTH);

    // Display the first line with all columns
    if let Some(first_line) = detail_lines.first() {
        println!(
            "{:<NAME_WIDTH$} {:<CRON_WIDTH$} {:<DETAIL_WIDTH$} {:<SOUND_WIDTH$} {:<CREATED_WIDTH$}",
            poke.name, poke.cron, first_line, sound_status, created
        );
    }

    // Display additional detail lines (indented)
    for line in detail_lines.iter().skip(1) {
        println!(
            "{:<NAME_WIDTH$} {:<CRON_WIDTH$} {:<DETAIL_WIDTH$} {:<SOUND_WIDTH$} {:<CREATED_WIDTH$}",
            "", "", line, "", ""
        );
    }
}

/// Display the header for job listings
pub fn display_job_header() {
    println!(
        "{:<NAME_WIDTH$} {:<CRON_WIDTH$} {:<DETAIL_WIDTH$} {:<SOUND_WIDTH$} {:<CREATED_WIDTH$}",
        "Name", "Cron", "Detail", "Sound", "Created"
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
        // Add a separator line between jobs for better readability
        if pokes
            .iter()
            .position(|p| std::ptr::eq(p, poke))
            .unwrap_or(0)
            < pokes.len() - 1
        {
            println!("{:-<TOTAL_WIDTH$}", "");
        }
    }
}

/// Display a single job in detail format with better formatting
pub fn display_job_detail(poke: &Poke) {
    println!("Job Details:");
    println!("{:=<TOTAL_WIDTH$}", "");
    display_job_header();
    display_job(poke);
    println!("{:=<TOTAL_WIDTH$}", "");
}
