# Ansible Setup Documentation

## Directory Structure

This Ansible setup follows best practices with proper separation of concerns:

```
ansible/
├── ansible.cfg                     # Ansible configuration
├── inventory.ini                   # Inventory definition
├── .vault_pass                    # Vault password file (not in git)
├── group_vars/
│   ├── all/
│   │   ├── main.yml              # Non-sensitive configuration
│   │   └── vault.yml             # Encrypted secrets (vault)
│   └── monitoring/
│       ├── main.yml              # Monitoring-specific configuration
│       └── vault.yml             # Monitoring secrets (Grafana password)
├── host_vars/
│   └── localhost/
│       └── main.yml              # Host-specific configuration
├── roles/
│   ├── wisepulse_pipeline/       # Data pipeline automation
│   ├── grafana/                  # Grafana visualization
│   ├── prometheus/               # Prometheus metrics server
│   ├── node_exporter/            # Node Exporter metrics
│   └── monitoring/               # Monitoring role meta
├── templates/
│   └── values.yaml.j2            # Jinja2 template for Kubernetes values
└── playbooks/
    ├── deploy.yml                # Kubernetes deployment playbook
    ├── setup-pipeline.yml        # Data pipeline setup playbook
    └── monitoring/
        ├── full.yml              # Deploy all monitoring (Prometheus + Grafana + Node Exporter)
        ├── core.yml              # Deploy Prometheus + Grafana only
        └── exporters.yml         # Deploy Node Exporter only
```

## Usage

### Deploy Loculus to Kubernetes
```bash
cd ansible
ansible-playbook playbooks/deploy.yml
```

### Setup Automated Data Pipeline
Configure automated nightly data fetching and processing with systemd timers:

```bash
cd ansible

# 1. (Optional) Edit configuration
vim host_vars/localhost/main.yml

# 2. Run the setup playbook
ansible-playbook playbooks/setup-pipeline.yml
```

This will:
- Create a `wisepulse` user for running the pipeline
- Build all Rust utilities
- Install systemd service and timer for automated runs
- Schedule nightly checks for new data (default: 2 AM)
- Only fetch and process data when new sequences are available

**Prerequisites** (must be installed manually):
- Rust/Cargo: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Docker and Docker Compose
- git
- Repository cloned to `/opt/wisepulse`

**Configuration**: 

All settings are in `host_vars/localhost/main.yml`. Edit this file to customize:
- Schedule time (`wisepulse_timer_oncalendar`)
- Fetch parameters (`wisepulse_fetch_days`, `wisepulse_fetch_max_reads`)
- Repository path (`wisepulse_repo_path`)
- User/group names
- API URL

**Runtime overrides** (optional):
```bash
ansible-playbook playbooks/setup-pipeline.yml \
  -e "wisepulse_timer_oncalendar='*-*-* 03:00:00'" \
  -e "wisepulse_fetch_days=120"
```

**Monitoring**:
```bash
# View timer status
sudo systemctl status wisepulse-pipeline.timer

# View next scheduled run
sudo systemctl list-timers wisepulse-pipeline.timer

# View logs
sudo journalctl -u wisepulse-pipeline.service -f

# Run manually
sudo systemctl start wisepulse-pipeline.service
```

### Deploy Monitoring Stack: Grafana + Prometheus:

```bash
# Deploy monitoring stack
ansible-playbook playbooks/monitoring/full.yml

# Access Grafana (localhost only)
# http://localhost:3000 (admin password in vault)
# Access Prometheus (localhost only)
# http://localhost:9090
# All services bind to 127.0.0.1 (not publicly exposed)
# Use SSH tunnel for remote access:
# ssh -L 3000:localhost:3000 -L 9090:localhost:9090 user@server
```

### Edit Secrets
```bash
cd ansible
ansible-vault edit group_vars/all/vault.yml
```

### View Encrypted Secrets (without editing)
```bash
cd ansible
ansible-vault view group_vars/all/vault.yml
```

## Adding New Secrets

1. Edit the vault file:
   ```bash
   ansible-vault edit group_vars/all/vault.yml
   ```

2. Add the secret under the appropriate section (e.g., `vault_secrets.new_service.password`)

3. Reference it in the template: `{{ vault_secrets.new_service.password }}`
