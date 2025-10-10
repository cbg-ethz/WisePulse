# WisePulse Pipeline Ansible Role

Automates the setup of WisePulse data pipeline with systemd timers for scheduled data fetching and processing.

## Description

This role configures automated data pipeline execution using systemd timers (similar to cron, but better integrated with systemd). The pipeline:

1. Checks the LAPIS API for new genomic sequences
2. Only downloads and processes data if new sequences are available
3. Runs on a configurable schedule (default: nightly at 2 AM)
4. Logs all activity to journald for easy monitoring

## Requirements

The following must be installed on the target system **before** running this role:

- **Docker and Docker Compose**: Follow [official Docker installation](https://docs.docker.com/engine/install/)
- **git**: Usually pre-installed on most systems
- **WisePulse repository**: Cloned to the target location (default: `/opt/WisePulse`)

**Note**: Rust/Cargo will be automatically installed for the `wisepulse` user by this role.

## Role Variables

Configuration is managed through Ansible's standard variable hierarchy:

1. **Role defaults** (`roles/wisepulse_pipeline/defaults/main.yml`) - sensible defaults
2. **Host variables** (`host_vars/localhost/main.yml`) - **customize here** 
3. **Playbook variables** - inline overrides
4. **Command-line** (`-e` flag) - runtime overrides

### Main Variables

```yaml
# Deployment
wisepulse_repo_path: /opt/wisepulse
wisepulse_user: wisepulse
wisepulse_group: wisepulse
wisepulse_log_dir: /var/log/wisepulse

# Schedule
wisepulse_timer_oncalendar: "*-*-* 02:00:00"  # Daily at 2 AM

# API
wisepulse_api_base_url: https://api.db.wasap.genspectrum.org

# Fetch configuration
wisepulse_fetch_days: 90
wisepulse_fetch_max_reads: 172500000  # 172.5 million reads
```

Note: The Docker memory limit for preprocessing should be adjusted according to the value of `wisepulse_fetch_max_reads`.

**To customize permanently**: Edit `host_vars/localhost/main.yml`  
**To override at runtime**: Use `-e "variable_name=value"`

## Example Playbook

Standard usage (reads from `host_vars/localhost/main.yml`):

```yaml
---
- hosts: localhost
  roles:
    - wisepulse_pipeline
```

With inline overrides:

```yaml
---
- hosts: localhost
  roles:
    - wisepulse_pipeline
  vars:
    wisepulse_timer_oncalendar: "*-*-* 03:00:00"  # Run at 3 AM
    wisepulse_fetch_days: 120
```

## Usage

Run the playbook with

```bash
# run playbook requires sudo password for config
cd ansible
sudo ansible-playbook playbooks/setup-pipeline.yml 
```

then use these commands to manage the pipeline:

```bash
# View timer status
sudo systemctl status wisepulse-pipeline.timer

# View next scheduled run
sudo systemctl list-timers wisepulse-pipeline.timer

# View service logs (live)
sudo journalctl -u wisepulse-pipeline.service -f

# View recent logs
sudo journalctl -u wisepulse-pipeline.service -n 100

# View logs between dates
sudo journalctl -u wisepulse-pipeline.service --since "2025-10-03 00:00" --until "2025-10-04 00:00"

# Inspect the currently deployed config
cat /etc/systemd/system/wisepulse-pipeline.service

# Run pipeline manually (for testing)
sudo systemctl start wisepulse-pipeline.service

# Stop the timer
sudo systemctl stop wisepulse-pipeline.timer

# Disable automatic execution
sudo systemctl disable wisepulse-pipeline.timer
```

## How It Works

1. **Timer Activation**: The systemd timer triggers at the scheduled time
2. **Service Execution**: The timer starts the systemd service
3. **Data Check**: The service runs `make smart-fetch-and-process` which:
   - Executes `check_new_data` to query the LAPIS API
   - Filters by `submittedAtTimestampFrom` (last update) AND `samplingDateFrom` (rolling window)
   - Writes max `submittedAtTimestamp` to `.next_timestamp` if data found
   - If new data exists: cleans old data, fetches fresh data, processes pipeline
   - If no new data: skips execution and logs
4. **Timestamp Update**: On successful completion, copies `.next_timestamp` to `.last_update`
5. **Logging**: All output goes to journald for easy monitoring

## Smart Data Checking

The pipeline uses the `check_new_data` Rust utility to efficiently check if new sequences have been submitted to the LAPIS API.

**How it works:**
- Queries API with both `submittedAtTimestampFrom` (last pipeline run) and `samplingDateFrom` (rolling window)
- Only processes data within the configured time window (default: last 90 days)
- Tracks the **maximum `submittedAtTimestamp`** from all matching submissions
- On success, updates `.last_update` with this max timestamp (not current time)

**First run behavior:**
- When no `.last_update` exists, uses `today - 90 days` as initial timestamp
- Queries API to find actual data in rolling window
- Saves the max `submittedAtTimestamp` from results

**Why this approach:**
- Handles out-of-order submissions correctly (e.g., data submitted on Oct 2 after data from Oct 1 was already processed)
- Only triggers pipeline for relevant data (within rolling window)
- Prevents missing retroactively submitted or revised data

**Exit codes:**
- `0`: New data available → Pipeline runs
- `1`: No new data → Pipeline skips
- `2`: Error → Pipeline skips and logs error


## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Systemd Timer                             │
│  (wisepulse-pipeline.timer)                                     │
│  Triggers: Daily at 02:00                                      │
└────────────────────┬────────────────────────────────────────────┘
                     │ activates
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Systemd Service                              │
│  (wisepulse-pipeline.service)                                   │
│  User: wisepulse                                                 │
│  WorkDir: /opt/wisepulse                                        │
└────────────────────┬────────────────────────────────────────────┘
                     │ executes
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                  make smart-fetch-and-process                    │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│              check_new_data (Rust binary)                        │
│  • Queries: submittedAtTimestampFrom={last} & samplingDateFrom  │
│  • Filters by rolling window (default: 90 days)                 │
│  • Writes max submittedAtTimestamp to .next_timestamp           │
│  • Exit 0 = new data, Exit 1 = no new data                     │
└────────────────────┬────────────────────────────────────────────┘
                     │
         ┌───────────┴───────────┐
         │                       │
    Exit 0                  Exit 1
    (new data)           (no new data)
         │                       │
         ▼                       ▼
┌─────────────────┐    ┌──────────────────┐
│ make clean-data │    │  Skip & Log      │
└────────┬────────┘    │  "No new data"   │
         │             └──────────────────┘
         ▼
┌─────────────────┐
│ make fetch-data │
│ (fetch_silo_data)│
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   make all      │
│ • split chunks  │
│ • merge chunks  │
│ • SILO preproc  │
└────────┬────────┘
         │
         ▼
┌──────────────────────┐
│ Update .last_update  │
│ (from .next_timestamp)│
└──────────────────────┘
         │
         ▼
    Logs to journald
```
