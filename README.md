## srSILO

To build the Rust helper scripts, run `cargo build --release`.

To process a set of `.ndjson.zst` files, put these in the directory `silo_input`. Then run `make all`. 
This will generate SILO Indexes which can be readily used to run a LAPIS/SILO API.

To automatically fetch genomic data files from the LAPIS API:
```bash
make fetch-data                              # Fetch with default settings
make fetch-data FETCH_DAYS=14 FETCH_MAX_READS=50000000  # Custom parameters
make fresh-data                              # Fetch new data and run full pipeline
```

To start the API you can run `LAPIS_PORT=80 docker compose up`.
Note that you can replace the `LAPIS_PORT` with another port that the api should listen on.

A swagger UI to the API can then be accessed at:
`http://localhost:80/swagger-ui/index.html`

Prerequisites:
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