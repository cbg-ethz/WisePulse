# WisePulse Deployment Guide

Detailed deployment instructions for all WisePulse components.

## Prerequisites

- **Ansible**: 2.16+
- **Docker & Docker Compose**: For srSILO and Loculus
- **Kubernetes cluster**: For Loculus deployment (kubectl configured)
- **Linux**: Ubuntu 20.04+, Debian 11+ recommended

## srSILO Pipeline Deployment

### Initial Setup

```bash
# Clone repository
git clone https://github.com/cbg-ethz/WisePulse.git
cd WisePulse

# Configure inventory
cp inventory.ini.example inventory.ini
# Edit inventory.ini to match your environment

# Configure srSILO settings
vim group_vars/srsilo/main.yml

# Run one-time setup
ansible-playbook playbooks/srsilo/setup.yml -i inventory.ini
```

**What setup.yml does:**
- Creates `srsilo` user and group
- Sets up directory structure in `/opt/srsilo/`
- Builds Rust tools (check_new_data, fetch_silo_data, split/merge utilities)
- Deploys Docker Compose configurations
- Verifies prerequisites

### Regular Operations

```bash
# Run automated update pipeline (7 phases)
ansible-playbook playbooks/srsilo/update-pipeline.yml -i inventory.ini

# Setup daily automated timer (runs at 2 AM)
ansible-playbook playbooks/srsilo/setup-timer.yml -i inventory.ini

# Check for new data only (Phase 2)
ansible-playbook playbooks/srsilo/update-pipeline.yml \
  -i inventory.ini \
  --tags phase2

# Run pipeline up to (but NOT including) preprocessing
# Phases 1-5: Prerequisites, Check, Cleanup, Fetch, Prepare
ansible-playbook playbooks/srsilo/update-pipeline.yml \
  -i inventory.ini \
  --tags phase1,phase2,phase3,phase4,phase5

# Run complete pipeline including preprocessing
# All phases 1-7
ansible-playbook playbooks/srsilo/update-pipeline.yml -i inventory.ini

# Custom configuration
ansible-playbook playbooks/srsilo/update-pipeline.yml \
  -i inventory.ini \
  -e "srsilo_retention_days=14" \
  -e "srsilo_fetch_days=30"
```

**Common Use Cases:**
- **Check for updates**: `--tags phase2` (exits immediately if no new data)
- **Dry run to preprocessing**: `--tags phase1,phase2,phase3,phase4,phase5` (downloads data, stops before SILO indexing)
- **Full update**: No tags (runs all 7 phases)

### 7-Phase Pipeline

1. **Pre-flight checks** - Verify system ready
2. **Check new data** - Query LAPIS API (exits early if no updates)
3. **Cleanup** - Apply retention policy, remove orphaned indexes
4. **Fetch data** - Download from LAPIS API
5. **Prepare** - Stop API, create processing marker
6. **Process** - Split → Merge → SILO preprocessing (with rollback on failure)
7. **Finalize** - Start API with new index, update timestamps

### Automated Daily Updates

Setup systemd timer for automatic daily runs at 2 AM:

```bash
# Deploy and enable timer
ansible-playbook playbooks/srsilo/setup-timer.yml -i inventory.ini

# Check timer status
sudo systemctl status srsilo-update.timer
sudo systemctl list-timers srsilo-update.timer

# View timer logs
sudo journalctl -u srsilo-update.service --since today

# Manually trigger (for testing)
sudo systemctl start srsilo-update.service

# Disable timer (if needed)
sudo systemctl stop srsilo-update.timer
sudo systemctl disable srsilo-update.timer
```

### Monitoring

```bash
# Check API status
curl http://localhost:8083/sample/info

# View logs (journald)
sudo journalctl -t srsilo-pipeline --since today
sudo journalctl -t srsilo-phase2 --since "1 hour ago"

# Check specific phase logs
sudo journalctl -t srsilo-check-data -n 50    # Phase 2: Check for new data
sudo journalctl -t srsilo-fetch -n 100        # Phase 4: Fetch data
sudo journalctl -t srsilo-preprocessing       # Phase 6: SILO preprocessing

# See LOGGING.md for complete journalctl usage guide

# List indexes
ls -lt /opt/srsilo/output/

# View Docker logs
docker logs srsilo-lapis-1

# Check processing status
cat /opt/srsilo/.last_update
cat /opt/srsilo/.next_timestamp
```

### Available Tags

Use `--tags` to run specific phases:

| Tag | Description | Use Case |
|-----|-------------|----------|
| `phase1` | Prerequisites & build | First-time setup |
| `phase2` | Check for new data | Quick check if update needed |
| `phase3` | Cleanup old indexes | Maintenance only |
| `phase4` | Fetch data from API | Download without processing |
| `phase5` | Prepare (stop API) | Pre-processing setup |
| `phase6` | Process data (split/merge/SILO) | Heavy computation |
| `phase7` | Finalize (start API) | Post-processing |
| `check` | Same as phase2 | Alias for checking |
| `fetch` | Same as phase4 | Alias for fetching |
| `process` | Same as phase6 | Alias for processing |
| `silo` | SILO preprocessing only | Within phase6 |

**Examples:**
```bash
# Everything up to preprocessing
--tags phase1,phase2,phase3,phase4,phase5

# Just preprocessing and finalization
--tags phase6,phase7

# Skip preprocessing, just fetch
--tags phase1,phase2,phase3,phase4
```

### Manual Recovery from Bad Index

SILO automatically uses the **newest index** in `/opt/srsilo/output/`. If something goes wrong:

```bash
# 1. List all indexes
ls -lth /opt/srsilo/output/

# 2. Identify and delete the problematic index(es)
sudo rm -rf /opt/srsilo/output/1234567890

# 3. Update the .last_update marker to match the newest remaining index
echo "1234567890" | sudo tee /opt/srsilo/.last_update

# 4. Restart the API (it will automatically pick up the newest remaining index)
cd /opt/WisePulse
ansible-playbook playbooks/srsilo/update-pipeline.yml -i inventory.ini \
  --tags silo -e api_action=start

# 5. Verify the API is serving the correct index
curl http://localhost:8083/sample/info | jq .dataVersion
```

**When to use manual recovery:**
- After a failed preprocessing run that created a corrupted index
- To remove damaged or incomplete indexes
- Emergency recovery when automated pipeline fails

## Loculus Deployment

### Setup

```bash
# Configure Loculus settings
vim group_vars/loculus/main.yml

# Configure secrets (encrypted)
ansible-vault edit group_vars/loculus/vault.yml

# Configure kubectl context
vim host_vars/localhost/main.yml

# Deploy to Kubernetes
ansible-playbook playbooks/loculus/deploy-loculus.yml --ask-become-pass
```

**Note:** Run without `sudo`. Use `--ask-become-pass` for privilege escalation when needed.

### Configuration Files

- `group_vars/loculus/main.yml` - Loculus settings (organisms, URLs, etc.)
- `group_vars/loculus/vault.yml` - Encrypted secrets (database, Keycloak, S3)
- `host_vars/localhost/main.yml` - kubectl context, Helm chart path

### Managing Secrets

```bash
# Edit secrets
ansible-vault edit group_vars/loculus/vault.yml

# View secrets (read-only)
ansible-vault view group_vars/loculus/vault.yml

# Change vault password
ansible-vault rekey group_vars/loculus/vault.yml
```

## Monitoring Stack Deployment

### Full Stack

```bash
# Deploy Prometheus + Grafana + Node Exporter
ansible-playbook playbooks/monitoring/full.yml

# Deploy only core (Prometheus + Grafana)
ansible-playbook playbooks/monitoring/core.yml

# Deploy only exporters (Node Exporter)
ansible-playbook playbooks/monitoring/exporters.yml
```

### Access

- **Grafana**: http://localhost:3000
  - Username: `admin`
  - Password: See `group_vars/monitoring/vault.yml`
- **Prometheus**: http://localhost:9090
- **Node Exporter**: http://localhost:9100/metrics

All services bind to localhost. For remote access, use SSH tunnel:

```bash
ssh -L 3000:localhost:3000 -L 9090:localhost:9090 user@server
```

### Dashboards

The setup automatically downloads the **Node Exporter Full** dashboard (ID: 1860) from Grafana.com, providing:
- CPU usage and load
- RAM and SWAP usage
- Disk I/O and space
- Network traffic
- System uptime

## Configuration Reference

### srSILO Variables

Edit `group_vars/srsilo/main.yml`:

```yaml
# User and paths
srsilo_user: srsilo
srsilo_group: srsilo
srsilo_base_dir: /opt/srsilo

# Data retention
srsilo_retention_days: 7               # Delete indexes older than 7 days
srsilo_retention_min_keep: 2           # Always keep at least 2 indexes

# Fetch configuration
srsilo_api_base_url: https://api.db.wasap.genspectrum.org
srsilo_fetch_days: 90                  # Fetch last 90 days of data
srsilo_fetch_max_reads: 172500000      # 172.5M reads for production

# Processing (Production: 377GB RAM)
srsilo_chunk_size: 1000000             # Large chunks for 377GB RAM
srsilo_docker_memory_limit: 340g       # 90% of 377GB RAM

# API
srsilo_lapis_port: 8083
```

### Loculus Variables

Edit `group_vars/loculus/main.yml`:

```yaml
# Deployment
name: "W-ASAP"
host: "your-domain.com"
loculus_environment: "local"

# Features
runDevelopmentMainDatabase: true
runDevelopmentKeycloakDatabase: true
runDevelopmentS3: true

# Organisms configuration
organisms:
  sarscov2:
    schema:
      organismName: "SARS-CoV-2"
      # ... metadata fields
```

### Monitoring Variables

Edit `group_vars/monitoring/main.yml`:

```yaml
# Prometheus
prometheus_version: "2.45.0"
prometheus_port: 9090
prometheus_retention_time: "15d"

# Grafana
grafana_port: 3000
grafana_admin_user: admin
# grafana_admin_password in vault.yml

# Node Exporter
node_exporter_version: "1.6.1"
node_exporter_port: 9100
```

## Troubleshooting

### srSILO Pipeline

**API not responding:**
```bash
# Check Docker containers
docker ps -a

# Restart API
cd /opt/srsilo
docker compose up -d

# Check logs
docker logs srsilo-lapis-1
```

**Processing failures:**
- Check `/opt/srsilo/.processing_marker` exists (indicates processing in progress)
- Check `/opt/srsilo/.last_update` for last successful run
- View recent indexes: `ls -lt /opt/srsilo/output/`
- Automatic rollback activates on preprocessing failures

**Out of memory:**
- Reduce `srsilo_chunk_size` in `group_vars/srsilo/main.yml`
- Reduce `srsilo_docker_memory_limit` to match available RAM
- Rule of thumb: 6GB RAM = chunk_size 30000, 16GB+ = chunk_size 100000

### Loculus

**Helm deployment fails:**
- Verify kubectl context: `kubectl config current-context`
- Check namespace exists: `kubectl get namespace`
- Review Helm chart path in `host_vars/localhost/main.yml`

**Database connection issues:**
- Verify database credentials in `group_vars/loculus/vault.yml`
- Check database pods: `kubectl get pods -n <namespace>`

### Monitoring

**Grafana login fails:**
- Verify password in `group_vars/monitoring/vault.yml`
- Reset: Edit vault file and redeploy

**Prometheus not scraping:**
- Check Prometheus targets: http://localhost:9090/targets
- Verify Node Exporter running: `systemctl status node_exporter`
- Check firewall rules (services bind to localhost)

**Dashboard not loading:**
- Dashboard auto-downloads on first deployment
- Manual import: Download ID 1860 from grafana.com, import via UI

## Next Steps

- **Automation**: Set up systemd timer for scheduled srSILO runs (see [Implementation Plan](srsilo/IMPLEMENTATION_PLAN.md))
- **Logging**: Configure centralized logging (planned)
- **Rollback**: Implement manual rollback playbook (planned)
- **Alerts**: Configure alerting for failures
