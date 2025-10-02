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
wisepulse_fetch_max_reads: 125000000  # 125 million reads
```

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

After running the playbook, use these commands to manage the pipeline:

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
   - Uses `submittedAtTimestampFrom` to check for submissions after `.last_update` timestamp
   - Single API call with `limit=1` for efficiency
   - If new data exists: cleans old data, fetches fresh data, processes pipeline
   - If no new data: skips execution and logs
4. **Timestamp Update**: On successful completion, updates `.last_update` file
5. **Logging**: All output goes to journald for easy monitoring

## Smart Data Checking

The pipeline uses the `check_new_data` Rust utility to efficiently check if new sequences have been submitted to the LAPIS API using a single API query with the `submittedAtTimestampFrom` parameter.

Exit codes:
- `0`: New data available → Pipeline runs
- `1`: No new data → Pipeline skips
- `2`: Error → Pipeline skips and logs error

