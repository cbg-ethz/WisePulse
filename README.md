## srSILO

WisePulse genomic data pipeline for processing COVID-19 sequencing data into SILO indexes.

### Quick Start

```bash
make help    # See all available targets and usage
make build   # Build required Rust tools
make all     # Process data in silo_input/ directory
```

### Prerequisites
- Rust/Cargo
- Docker Compose (optional for final SILO step)
- Linux platform (for preprocessing scripts)

## W-ASAP Loculus

Deploy to Kubernetes using Ansible:

```bash
cd ansible
echo "your-vault-password" > .vault_pass    # Configure vault password
ansible-vault edit secrets/my-values.yaml  # Configure your values
ansible-playbook apply-values.yml         # Deploy
```