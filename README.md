# WisePulse

<div align="center">

<img src="roles/loculus/files/images/wasap-logo.png" alt="WisePulse Logo" width="150"/>

### WISE Loculus with V-Pipe – Infrastructure for start-to-end viral wastewater analysis

![Status: Public Beta](https://img.shields.io/badge/status-public%20beta-blue)
![Work in Progress](https://img.shields.io/badge/work%20in%20progress-orange)
![Platform: Linux](https://img.shields.io/badge/platform-linux-lightgrey)
![License: MIT](https://img.shields.io/badge/license-MIT-green)

</div>

---

## About

**WisePulse** is an end-to-end infrastructure for viral wastewater surveillance, combining [Loculus](https://loculus.org) for sequence data management with [LAPIS-SILO](https://github.com/GenSpectrum/LAPIS-SILO), a high-performance genomic database – here deployed for the first time to **S**hort-**R**ead / amplicon sequences. It enables real-time querying and visualization of viral genomic data from wastewater samples.

### Current Features
- Loculus instance for easy data sharing and collaboration
- srSILO API for querying amplicon sequence data

#### Data Processing Pipeline (external)
Upstream data processing is handled separately:
- NGS processing via [V-pipe](https://github.com/cbg-ethz/V-pipe)
- Database ingestion with [sr2silo](https://github.com/cbg-ethz/sr2silo) from V-pipe alignment outputs

### Roadmap
- Raw data submission & V-pipe pipeline integration
- On-demand exploratory analysis tools

## Quick Start

```bash
# One-time setup (user, directories, build tools)
ansible-playbook playbooks/srsilo/setup.yml -i inventory.ini --become  --ask-become-pass

# Run automated update pipeline (checks for new data, processes, updates API)
ansible-playbook playbooks/srsilo/update-pipeline.yml -i inventory.ini \
  --become --ask-become-pass \
  -e "@playbooks/srsilo/vars/test_vars.yml"

# Setup daily automated runs at 2 AM
ansible-playbook playbooks/srsilo/setup-timer.yml -i inventory.ini

# Check API status
curl http://localhost:8083/sample/info
```

API available at: http://localhost:8083/swagger-ui/index.html

## Development

### Pre-commit Hooks

This repository uses [pre-commit](https://pre-commit.com/) hooks to ensure code quality and prevent accidental commits of sensitive data.

**Setup:**
```bash
# Install pre-commit
pip install pre-commit

# Install the git hooks
pre-commit install
```

**What the hooks do:**
- Check YAML syntax and formatting
- Detect large files before committing
- Run Ansible linting with production profile
- Format and lint Python code with ruff
- Type-check Python code with pyright
- Scan for secrets with detect-secrets
- Verify Ansible vault files are encrypted (prevents unencrypted secrets)

**Manual execution:**
```bash
# Run all hooks on all files
pre-commit run --all-files

# Run specific hook
pre-commit run check-ansible-vault --all-files
```

**Note:** All Ansible vault files (e.g., `vault.yml`, `secret*.yml`) must be encrypted with `ansible-vault` before committing.

## Architecture

### srSILO Pipeline

The srSILO pipeline is now **fully managed by Ansible** with:
-  **Low Downtime** (API managed automatically)
-  **Self-healing** (automatic rollback on failures)
-  **Smart execution** (exits early if no new data)
-  **Retention policy** (automatic cleanup of old indexes)

**Full documentation**: See [`docs/srsilo/ARCHITECTURE.md`](docs/srsilo/ARCHITECTURE.md)

**Playbooks**:
- `playbooks/srsilo/setup.yml` - One-time server setup
- `playbooks/srsilo/update-pipeline.yml` - Full automated update (7 phases)
- Future: `playbooks/srsilo/rollback.yml` - Manual rollback to previous index

**Example Usage**:
```bash
# Full automated update (recommended)
ansible-playbook playbooks/srsilo/update-pipeline.yml -i inventory.ini

# With custom configuration
ansible-playbook playbooks/srsilo/update-pipeline.yml \
  -i inventory.ini \
  -e "srsilo_retention_days=7" \
  -e "srsilo_fetch_days=30"

# Test specific phase (e.g., check for new data only)
ansible-playbook playbooks/srsilo/update-pipeline.yml \
  -i inventory.ini \
  --tags phase2
```

### Loculus Deployment

Deploy W-ASAP Loculus to Kubernetes:
```bash
ansible-playbook playbooks/loculus/deploy-loculus.yml --ask-become-pass
```

### Monitoring Stack

Deploy Prometheus + Grafana:
```bash
ansible-playbook playbooks/monitoring/full.yml
```

## Configuration

Key configuration in `group_vars/srsilo/main.yml`:

```yaml
# Data retention
srsilo_retention_days: 7               # Delete indexes older than 7 days
srsilo_retention_min_keep: 2           # Always keep at least 2 indexes

# Fetch configuration  
srsilo_fetch_days: 90                  # Fetch last 90 days of data
srsilo_fetch_max_reads: 172500000      # 172.5M reads for production

# Processing (Production: 377GB RAM server)
srsilo_chunk_size: 1000000             # Large chunks for high-memory environment
srsilo_docker_memory_limit: 340g       # 90% of 377GB RAM

# For testing with constrained resources (8GB RAM):
# Use: -e "@playbooks/srsilo/vars/test_vars.yml"
```

See [`docs/srsilo/ARCHITECTURE.md`](docs/srsilo/ARCHITECTURE.md) for complete configuration reference.