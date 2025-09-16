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
    make fetch-and-process               # Fetch data and run full pipeline
    LAPIS_PORT=8083 docker compose up -d   # Run srSILO on port 8083
````

### Prerequisites
- Rust/Cargo
- Docker Compose (optional for final SILO step)
- Linux platform (for preprocessing scripts)

## W-ASAP Loculus

Deploy to Kubernetes using Ansible:

```bash
cd ansible
ansible-playbook playbooks/deploy.yml
```

for more detail see `ansible/README.md`