## srSILO

WisePulse genomic data pipeline for processing COVID-19 sequencing data into SILO indexes.

### Quick Start

```bash
make help    # See all available targets and usage
make build   # Build required Rust tools
make all     # Process data in silo_input/ directory
```

Configure desired data to fetch directly in the Makefile.

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