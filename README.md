# Poke Me

A service that allows you to schedule recurring system notifications using cron expressions. 

## Installation

### Prerequisites
- Rust 1.70+ and Cargo
- SQLite3

### Build & Install
```bash
git clone <repository-url>
cd poke_me
cargo build --release

# Install to your system PATH (Linux/macOS)
cargo install --path .

# The binary will be installed to ~/.cargo/bin/
# Make sure this directory is in your PATH

# Or manually copy the binary
sudo cp target/release/poke_me /usr/local/bin/
```

### Systemd Service Installation (Linux)

For automatic startup and persistent background operation, you can install `poke_me` as a systemd service:

```bash
# Build and install the binary
cargo install --path .

# Verify installation
which poke_me
# Should output: /home/username/.cargo/bin/poke_me

# Run the automated setup script
cd scripts
./setup-systemd.sh
```

**Note:** Copy and edit the template file for more fine tuned configurations.

### Database separation
The systemd service uses a separate database location:
- **Systemd service**: `~/.local/share/poke_me/poke.db`
- **Development mode** (`cargo run`): `./poke.db` (local directory)

This allows you to:
- Use `cargo run` for development with local database
- Use `poke_me` commands with the systemd service database
- Keep development and production data separate

## Usage

```bash
A service to setup recurring notifications

Usage: poke_me <COMMAND>

Commands:
  service       Start the background notification service
  add           Add a new scheduled notification job
  list          List all scheduled notification jobs
  detail        Show detailed information for a specific job by exact name
  search        Search for jobs by name pattern (partial matching)
  remove        Remove a scheduled job by name
  toggle-sound  Toggle sound on/off for an existing job
  stop          Stop the running notification service
  help          Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
### Basic Commands

```bash
# Add eye rest reminder every 20 minutes
poke_me add "Rest your eyes" "0 */20 * * * *" "Take a 20-second break to rest your eyes" --sound # Sound is off by default

# List all scheduled jobs
poke_me list

# Show job details
poke_me detail "morning_coffee"

# Search for jobs
poke_me search "morning"

# Remove a job
poke_me remove "morning_coffee"

# Start background service
poke_me service

# Stop running service
poke_me stop
```

### Cron Expression Format

The service uses 6-field cron expressions:
```
┌───────────── second (0-59)
│ ┌─────────── minute (0-59)
│ │ ┌───────── hour (0-23)
│ │ │ ┌─────── day of month (1-31)
│ │ │ │ ┌───── month (1-12)
│ │ │ │ │ ┌─── day of week (0-7, Sunday = 0 or 7)
│ │ │ │ │ │
* * * * * *
```

**Examples:**
- `0 0 8 * * *` - Daily at 8:00 AM
- `0 30 12 * * 1-5` - Weekdays at 12:30 PM
- `0 0 0 1 * *` - First day of every month at midnight
- `0 */20 * * * *` - Every 20 minutes

