# WisePulse

<div align="center">

<img src="ansible/roles/loculus/files/images/wasap-logo.png" alt="WisePulse Logo" width="150"/>

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
# Build tools
make build

# Fetch data and run pipeline
make fetch-and-process

# Start API
LAPIS_PORT=8083 docker compose up -d
```

API available at: http://localhost:8083/swagger-ui/index.html

### Prerequisites
- Rust/Cargo
- Docker Compose
- Linux platform

## Automated Pipeline & Deployment

Use Ansible for:
- **Automated data pipeline**: Nightly data fetching and processing
- **Loculus deployment**: Deploy W-ASAP to Kubernetes
- **Monitoring**: Prometheus + Grafana metrics

See `ansible/README.md` for complete documentation.

## Manual Pipeline Usage

See `make help` for all available commands.

**Key commands:**
```bash
make build                    # Build all Rust tools
make fetch-and-process        # Fetch data and run full pipeline
make smart-fetch-and-process  # Smart run with API lifecycle management
make clean-all                # Clean everything including Docker
```