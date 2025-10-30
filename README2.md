# Ansible Automation

Ansible playbooks for WisePulse automation and deployment.

## Quick Reference

```bash
# Deploy Loculus to Kubernetes
ansible-playbook playbooks/deploy-loculus.yml --ask-become-pass

# Setup automated data pipeline
ansible-playbook playbooks/setup-pipeline.yml

# Deploy monitoring stack
ansible-playbook playbooks/monitoring/full.yml
```

## Playbooks

### 1. Loculus Kubernetes Deployment

Deploy W-ASAP Loculus to Kubernetes cluster:

```bash
ansible-playbook playbooks/deploy-loculus.yml --ask-become-pass
```

**Configuration:**
- `group_vars/loculus/main.yml` - Loculus settings (organisms, URLs, etc.)
- `group_vars/loculus/vault.yml` - Encrypted secrets
- `host_vars/localhost/main.yml` - kubectl context, Helm chart path

**Note:** Run without `sudo`. Use `--ask-become-pass` for privilege escalation when needed.

### 2. Automated Data Pipeline

Setup systemd timer for nightly data processing:

```bash
ansible-playbook playbooks/setup-pipeline.yml
```

**What it does:**
- Creates `wisepulse` user for pipeline
- Builds all Rust utilities
- Installs systemd service + timer
- Schedules nightly data checks (default: 2 AM)
- Only processes when new data available
- Manages SILO API lifecycle

**Prerequisites:**
- Rust/Cargo
- Docker and Docker Compose
- git

**Configuration:** Edit `host_vars/localhost/main.yml` to customize schedule, fetch parameters, paths.

**Monitor:**
```bash
sudo systemctl status wisepulse-pipeline.timer
sudo journalctl -u wisepulse-pipeline.service -f
sudo systemctl start wisepulse-pipeline.service  # Manual run
```

### 3. Monitoring Stack

Deploy Prometheus + Grafana + Node Exporter:

```bash
ansible-playbook playbooks/monitoring/full.yml      # Full stack
ansible-playbook playbooks/monitoring/core.yml      # Prometheus + Grafana only
ansible-playbook playbooks/monitoring/exporters.yml # Node Exporter only
```

**Access:**
- Grafana: http://localhost:3000 (admin password in `group_vars/monitoring/vault.yml`)
- Prometheus: http://localhost:9090

All services bind to localhost. Use SSH tunnel for remote access:
```bash
ssh -L 3000:localhost:3000 -L 9090:localhost:9090 user@server
```

## Managing Secrets

```bash
# Edit secrets
ansible-vault edit group_vars/loculus/vault.yml
ansible-vault edit group_vars/monitoring/vault.yml

# View secrets (read-only)
ansible-vault view group_vars/loculus/vault.yml
```

## Directory Structure

```
ansible/
├── playbooks/
│   ├── deploy-loculus.yml    # Loculus deployment
│   ├── setup-pipeline.yml    # Pipeline automation
│   └── monitoring/           # Monitoring playbooks
├── roles/                    # Ansible roles
│   ├── loculus/
│   ├── wisepulse_pipeline/
│   ├── prometheus/
│   ├── grafana/
│   └── node_exporter/
├── group_vars/
│   ├── loculus/              # Loculus configuration
│   └── monitoring/           # Monitoring configuration
└── host_vars/
    └── localhost/            # Host-specific settings
```
