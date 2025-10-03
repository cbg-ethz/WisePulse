## srSILO

WisePulse genomic data pipeline for processing COVID-19 sequencing data into SILO indexes.

## Usage

To fetch data from the W-ASAP Loculus LAPIS and pre-process the set of `.ndjson.zst` files 
use the functionality in the `make`, see `make help`

 This will generate SILO Indexes which can be readily used to run a LAPIS/SILO API.

Configure the desired data to fetch and directories directly in the Makefile.

To start the API you can run `LAPIS_PORT=8083 docker compose up`. Note that you can replace the `LAPIS_PORT` with another port that the api should listen on.

A swagger UI to the API can then be accessed at: http://localhost:80/swagger-ui/index.html

### Quick Start

```bash
    make build                           # Build all Rust tools
    make clean-all                       # Clean everything including Docker
    make fetch-and-process               # Fetch data and run full pipeline (no API mgmt)
    make smart-fetch-and-process         # Smart: checks new data, stops/restarts API (for automation)
    LAPIS_PORT=8083 docker compose up -d # Run srSILO on port 8083
```

**Note**: `smart-fetch-and-process` manages the API lifecycle (stops before preprocessing, restarts after). Use `fetch-and-process` for manual runs when you want to control the API yourself.

### Prerequisites
- Rust/Cargo
- Docker Compose (optional for final SILO step)
- Linux platform (for preprocessing scripts)

## Automated Data Pipeline

Set up automated nightly data fetching and processing using Ansible:

```bash
cd ansible

# 1. (Optional) Customize settings
vim host_vars/localhost/main.yml

# 2. Run the setup playbook
ansible-playbook playbooks/setup-pipeline.yml
```

This configures a systemd timer that:
- Runs nightly (default: 2 AM)
- Checks if new data is available via LAPIS API
- Only downloads and processes data when new sequences exist
- **Manages API lifecycle**: Stops SILO API before preprocessing (1-2h), restarts after completion
- Logs all activity to journald

**Prerequisites for automation** (must be installed before running playbook):
- Rust/Cargo: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Docker and Docker Compose
- git
- Repository at `/opt/wisepulse` (or customize via `wisepulse_repo_path`)

**Monitor the pipeline**:
```bash
sudo systemctl status wisepulse-pipeline.timer   # Check timer status
sudo journalctl -u wisepulse-pipeline.service -f # View logs
sudo systemctl start wisepulse-pipeline.service  # Run manually
```

See `ansible/README.md` for more details.

## W-ASAP Loculus

Deploy to Kubernetes using Ansible:

```bash
cd ansible
ansible-playbook playbooks/deploy.yml
```

for more detail see `ansible/README.md`