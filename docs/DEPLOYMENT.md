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

# Custom configuration
ansible-playbook playbooks/srsilo/update-pipeline.yml \
  -i inventory.ini \
  -e "srsilo_retention_days=14" \
  -e "srsilo_fetch_days=30"

# Test specific phase
ansible-playbook playbooks/srsilo/update-pipeline.yml \
  -i inventory.ini \
  --tags phase2  # Check for new data only
```

### 7-Phase Pipeline

1. **Pre-flight checks** - Verify system ready
2. **Check new data** - Query LAPIS API (exits early if no updates)
3. **Cleanup** - Apply retention policy, remove orphaned indexes
4. **Fetch data** - Download from LAPIS API
5. **Prepare** - Stop API, create processing marker
6. **Process** - Split → Merge → SILO preprocessing (with rollback on failure)
7. **Finalize** - Start API with new index, update timestamps

### Monitoring

```bash
# Check API status
curl http://localhost:8083/sample/info

# List indexes
ls -lt /opt/srsilo/output/

# View Docker logs
docker logs srsilo-lapis-1

# Check processing status
cat /opt/srsilo/.last_update
cat /opt/srsilo/.next_timestamp
```

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
srsilo_fetch_max_reads: 125000000      # Max reads per batch

# Processing
srsilo_chunk_size: 100000              # Reads per chunk (adjust for RAM)
srsilo_docker_memory_limit: 350g       # Docker memory limit

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
