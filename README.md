## srSILO

### Quick Start

```bash
# See all available targets
make help

# Build all Rust tools (required first step)
make build

# Option 1: Use existing data files
# Put .ndjson.zst files in silo_input/ directory, then:
make all

# Option 2: Fetch data automatically and process
make fetch-and-process

# Option 3: Just fetch data (without processing)
make fetch-data
```

### Build Process

To build the Rust helper scripts:
```bash
make build              # Build all Rust tools
# OR
cargo build --release   # Direct cargo build
```

### Data Processing

To process existing `.ndjson.zst` files:
1. Put files in the `silo_input/` directory
2. Run `make all` - this generates SILO indexes for the LAPIS/SILO API

### Data Fetching

To automatically fetch genomic data from the LAPIS API:
```bash
make fetch-data                                          # Fetch with default settings
make fetch-data FETCH_DAYS=30 FETCH_MAX_READS=5000000  # Custom parameters
make fetch-and-process                                   # Fetch + process in one command
```

Available fetch configuration variables:
- `FETCH_START_DATE` (default: today's date)
- `FETCH_DAYS` (default: 60) 
- `FETCH_MAX_READS` (default: 1000000)
- `FETCH_OUTPUT_DIR` (default: silo_input)
- `FETCH_API_BASE_URL` (default: https://api.db.wasap.genspectrum.org)

### Cleanup

```bash
make clean       # Remove intermediate processing files
make clean-data  # Remove downloaded data files
make clean-all   # Clean everything (files + Docker + Rust builds)
```

### API Deployment

To start the API you can run `LAPIS_PORT=80 docker compose up`.
Note that you can replace the `LAPIS_PORT` with another port that the api should listen on.

A swagger UI to the API can then be accessed at:
`http://localhost:80/swagger-ui/index.html`

### Prerequisites
- installed cargo
- installed Docker Compose
- platform: Linux (w.r.t. pre-processing scripts invoked by `make`)

## W-ASAP Loculus

Deploy to Kubernetes using Ansible:

```bash
cd ansible
echo "your-vault-password" > .vault_pass    # Configure vault password
ansible-vault edit secrets/my-values.yaml  # Configure your values
ansible-playbook apply-values.yml         # Deploy
```