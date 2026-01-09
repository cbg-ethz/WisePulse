# srSILO Pipeline - Architecture

Automated multi-virus genomic data processing: monitors LAPIS API for new sequences, downloads, processes, and indexes data with self-healing rollback on failures.

## Supported Viruses

| Virus | Organism | LAPIS Port | SILO Port | Status |
|-------|----------|------------|-----------|--------|
| COVID (SARS-CoV-2) | `covid` | 8083 | 8081 | Production |
| RSV-A | `rsva` | 8084 | 8082 | Production |
| RSV-B | `rsvb` | 8085 | 8086 | Planned |
| Influenza (H1, N1, H3, N2) | `flu-*` | 8087+ | 8088+ | Planned |

## Directory Structure

```
/opt/srsilo/
├── covid/                    # SARS-CoV-2 instance
│   ├── input/               # Downloaded NDJSON files
│   ├── output/              # SILO indexes (timestamped)
│   ├── sorted_chunks/       # Processing temp
│   ├── tmp/                 # Processing temp
│   ├── config/              # docker-compose, configs
│   ├── .last_update         # Timestamp checkpoint
│   └── sorted.ndjson.zst    # Merged data
├── rsva/                     # RSV-A instance (same structure)
└── tools/                    # Shared Rust binaries
    └── target/release/
```

## Components

**Playbooks**:
- `update-all-viruses.yml` - Run pipeline for all enabled viruses (production)
- `update-pipeline.yml` - Run pipeline for single virus (debug/testing)
- `setup.yml` - Initial setup
- `setup-timer.yml` - Configure systemd timer

**Rust Tools**: `check_new_data`, `fetch_silo_data`, `split_into_sorted_chunks`, `merge_sorted_chunks`, `add_offset`

**Docker**: SILO (genspectrum/lapis-silo), LAPIS API (genspectrum/lapis)

## 7-Phase Pipeline

1. **Prerequisites**: Verify user, directories, Docker; build Rust tools
2. **Check New Data**: Query LAPIS API; exit early if no updates
3. **Cleanup**: Apply retention policy, detect orphans, clean temp dirs
4. **Fetch**: Download NDJSON from LAPIS (`--organism` parameter), compress with zstd
5. **Prepare**: Create `.preprocessing_in_progress` marker, stop API (free memory)
6. **Process** (block/rescue): Split, merge, SILO preprocessing
7. **Finalize**: Start API with new index, update `.last_update`, cleanup markers

## Usage

```bash
# Update all enabled viruses (production)
ansible-playbook playbooks/srsilo/update-all-viruses.yml -i inventory.ini

# Update single virus (debug)
ansible-playbook playbooks/srsilo/update-pipeline.yml -i inventory.ini -e "srsilo_virus=rsva"

# Test with reduced resources (8GB RAM)
ansible-playbook playbooks/srsilo/update-all-viruses.yml -i inventory.ini \
  -e "@playbooks/srsilo/vars/test_vars.yml"

# Setup systemd timer (daily at 2 AM)
ansible-playbook playbooks/srsilo/setup-timer.yml -i inventory.ini
```

## Configuration

### Enable Viruses

In `roles/srsilo/defaults/main.yml`:
```yaml
srsilo_enabled_viruses:
  - covid
  - rsva
```

### Per-Virus Configuration

In `group_vars/srsilo/main.yml`:
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

### Global Settings

```yaml
srsilo_retention_days: 3        # Delete indexes older than N days
srsilo_retention_min_keep: 2    # Always keep at least M indexes
```

## Adding a New Virus

1. Create config files in `roles/srsilo/files/viruses/<virus_id>/`:
   - `database_config.yaml`
   - `preprocessing_config.yaml`
   - `reference_genomes.json`

2. Register in `roles/srsilo/defaults/main.yml`:
```yaml
srsilo_viruses:
  new_virus:
    organism: new-virus      # LAPIS API path segment
    instance_name: wise-new-virus
    lapis_port: 8086
    silo_port: 8087
```

3. Add per-virus config in `group_vars/srsilo/main.yml`

4. Enable: add `new_virus` to `srsilo_enabled_viruses`

## State Files

- **`.last_update`**: Unix timestamp of last successful run (persistent)
- **`.next_timestamp`**: Temp timestamp for current update (ephemeral)
- **`output/.preprocessing_in_progress`**: Orphan detection marker

## Monitoring

```bash
# View pipeline logs
journalctl -t srsilo-pipeline -f

# Check timer status
systemctl status srsilo-update.timer

# API health
curl http://localhost:8083/sample/info  # COVID
curl http://localhost:8084/sample/info  # RSV-A
```
