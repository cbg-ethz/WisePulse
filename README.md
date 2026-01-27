# WisePulse

<div align="center">

<img src="roles/loculus/files/images/wasap-logo.png" alt="WisePulse Logo" width="150"/>

### WISE Loculus with V-Pipe – Infrastructure for start-to-end viral wastewater analysis

![Status: Public Beta](https://img.shields.io/badge/status-public%20beta-blue)
![Platform: Linux](https://img.shields.io/badge/platform-linux-lightgrey)
![License: MIT](https://img.shields.io/badge/license-MIT-green)

</div>

---

## About

**WisePulse** is an end-to-end infrastructure for viral wastewater surveillance, combining [Loculus](https://loculus.org) for sequence data management with [LAPIS-SILO](https://github.com/GenSpectrum/LAPIS-SILO), a high-performance genomic database – here deployed for the first time to **S**hort-**R**ead / amplicon sequences. It enables real-time querying and visualization of viral genomic data from wastewater samples.

**Demo**: [Watch on YouTube](https://www.youtube.com/watch?v=kCUd-o1FbXg&t=420) (starts at 7:00)

#### Major Goals

- **Cluster to Browser**: Brought downstream exploratory analysis from exclusive, high barrier-to-entry cluster-based workflows to user-friendly, simple browser-based workflows

- **Days to Minutes**: Simplified exploratory analysis beyond the regular processing reports produced by the WISE Consortium from days waiting for requests served by few with data access to self-serving

- **Expert-only to Community Access**: Direct access to data allows for community access—not just those with data access and expert bioinformatics knowledge—to explore, catering for virologists and phylogeneticists

- **Isolated to Integrated**: First time wastewater data can be analyzed on the fly with the latest clinical variant definitions without manual work

### Current Features
- Loculus instance for easy data sharing and collaboration
- Multi-virus srSILO API for querying amplicon sequence data (SARS-CoV-2, RSV-A)

#### Live Instances
- 🔬 **Mutational Analysis Dashboards**: [genspectrum.org/swiss-wastewater](https://genspectrum.org/swiss-wastewater)
- 🗄️ **Swiss Wastewater Viral Alignment Database**: [db.wasap.genspectrum.org](https://db.wasap.genspectrum.org/)

#### Data Processing Pipeline (external)
Upstream data processing is handled separately:
- NGS processing via [V-pipe](https://github.com/cbg-ethz/V-pipe)
- Database ingestion with [sr2silo](https://github.com/cbg-ethz/sr2silo) from V-pipe alignment outputs

### Roadmap
- Expand to more viruses
- Raw data submission & V-pipe pipeline integration
- On-demand exploratory analysis tools

## Quick Start

```bash
# One-time setup (user, directories, build tools)
ansible-playbook playbooks/srsilo/setup.yml -i inventory.ini --become --ask-become-pass

# Run automated update pipeline for all enabled viruses
ansible-playbook playbooks/srsilo/update-all-viruses.yml -i inventory.ini --become --ask-become-pass

# Or update a single virus
ansible-playbook playbooks/srsilo/update-pipeline.yml -i inventory.ini -e "srsilo_virus=rsva"

# Test with reduced resources (8GB RAM)
ansible-playbook playbooks/srsilo/update-all-viruses.yml -i inventory.ini \
  -e "@playbooks/srsilo/vars/test_vars.yml"

# Setup daily automated runs at 2 AM
ansible-playbook playbooks/srsilo/setup-timer.yml -i inventory.ini

# Check API status
curl http://localhost:8083/sample/info  # COVID
curl http://localhost:8084/sample/info  # RSV-A
```

API Swagger UI:
- COVID: http://localhost:8083/swagger-ui/index.html
- RSV-A: http://localhost:8084/swagger-ui/index.html

## Architecture

### srSILO Pipeline

Multi-virus genomic data pipeline **fully managed by Ansible** with:
- **Multi-virus support** (SARS-CoV-2, RSV-A; RSV-B and Influenza planned)
- **Low Downtime** (API managed automatically)
- **Self-healing** (automatic rollback on failures)
- **Smart execution** (exits early if no new data)
- **Retention policy** (automatic cleanup of old indexes)

**Full documentation**: See [`docs/srsilo/ARCHITECTURE.md`](docs/srsilo/ARCHITECTURE.md)

**Playbooks**:
- `playbooks/srsilo/setup.yml` - One-time server setup
- `playbooks/srsilo/update-all-viruses.yml` - Update all enabled viruses (production)
- `playbooks/srsilo/update-pipeline.yml` - Update single virus (debug/testing)
- `playbooks/srsilo/setup-timer.yml` - Configure daily automated runs

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

### Nginx Reverse Proxy

Deploy Nginx as a reverse proxy with SSL termination for all services:
```bash
ansible-playbook playbooks/setup_nginx.yml -i inventory.ini --ask-become-pass
```
See [`roles/nginx/README.md`](roles/nginx/README.md) for details.

## Configuration

Enable viruses in `roles/srsilo/defaults/main.yml`:
```yaml
srsilo_enabled_viruses:
  - covid
  - rsva
```

Per-virus configuration in `group_vars/srsilo/main.yml`:
```yaml
srsilo_virus_config:
  covid:
    fetch_days: 90
    fetch_max_reads: 172500000
    chunk_size: 1000000
    docker_memory_limit: 340g
  rsva:
    fetch_days: 90
    fetch_max_reads: 50000000
    chunk_size: 500000
    docker_memory_limit: 340g
```

See [`docs/srsilo/ARCHITECTURE.md`](docs/srsilo/ARCHITECTURE.md) for complete configuration reference and adding new viruses.