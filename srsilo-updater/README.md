# srsilo-updater

Python pipeline that keeps the srSILO genomic database up to date. It fetches new sequence data from the upstream Loculus/LAPIS API, preprocesses it through SILO, and hot-swaps the running index with zero downtime.

## Structure

```
srsilo-updater/
  pipeline/              # Python package
    __main__.py          # Entry point
    config.py            # Config loading (pipeline.yml → typed dataclasses)
    phases/
      check_new_data.py  # Phase 2: query API for new submissions
      cleanup.py         # Phase 3: retention policy + reset working dirs
      fetch.py           # Phase 4: download .ndjson.zst files
      sort_and_merge.py  # Phase 6a: chunk, sort, merge reads
      preprocessing.py   # Phase 6b: run SILO preprocessing container
      finalize.py        # Phase 7: swap index, update timestamp, start API
      api.py             # Docker compose start/stop + health check
  rust/                  # Rust binaries (built by Ansible, called by pipeline)
    src/
      check_new_data/
      fetch_silo_data/
      split_into_sorted_chunks/
      merge_sorted_chunks/
  tests/
    data/                # Sample .ndjson.zst files for integration tests
  pipeline.yml.example   # Annotated production config template
  pipeline-test.yml      # Reduced-resource config for local dev runs
  TEST_PLAN.md           # Test checklist (not yet implemented)
  pyproject.toml
```

## Pipeline phases

| Phase | What happens |
|---|---|
| 1 | Ansible setup (prerequisites, build tools, deploy configs) |
| 2 | `check_new_data` binary queries the LAPIS API; exits early if nothing new |
| 3 | Retention cleanup of old indexes; reset working directories |
| 4 | `fetch_silo_data` binary downloads `.ndjson.zst` files from the API |
| 6a | `split_into_sorted_chunks` + `merge_sorted_chunks` produce `sorted.ndjson.zst` |
| 6b | SILO preprocessing Docker container builds the index |
| 7 | New index swapped in, API restarted, timestamp updated |

On any failure in phases 6–7 the rollback path in `finalize.py` deletes the partial index and restarts the API with the previous good index.

## Running locally

```bash
# Install into a venv
python3 -m venv .venv
.venv/bin/pip install -e .

# Run against the test config (reduced fetch window and memory limits)
.venv/bin/python -m pipeline --config pipeline-test.yml

# Run a single virus
.venv/bin/python -m pipeline --config pipeline-test.yml --virus covid
```

Requires the Rust binaries to already be built and available at the path configured in `tools_path` (default: `/opt/srsilo/tools/target/release/`). Run the Ansible setup playbook first, or point `tools_path` at a local build.

## Config file

The pipeline reads a single YAML file. In production this is written by Ansible to `/opt/srsilo/pipeline.yml` from the `pipeline.yml.j2` template. See `pipeline.yml.example` for all fields with comments.

Key settings:

```yaml
base_path: /opt/srsilo          # Root of all virus data directories
tools_path: /opt/srsilo/tools   # Where Rust binaries are built
api_base_url: https://api.db.wasap.genspectrum.org
enabled_viruses: [covid, rsva]

viruses:
  covid:
    fetch_days: 120           # How far back to look for new sampling dates
    fetch_max_reads: 20000000 # Must exceed the largest single-day read count
    chunk_size: 30000
    docker_memory_limit: 7g
```

**Common gotcha:** `fetch_days` must be large enough to cover the actual age of the newest data in the API, and `fetch_max_reads` must exceed the total read count for the busiest single sampling day — otherwise the fetch binary exits with zero downloads.

## In production

The pipeline is invoked daily by a systemd timer (`srsilo-update.timer`) which runs `srsilo-update.service`. The service calls:

```
/opt/srsilo/venv/bin/python -m pipeline --config /opt/srsilo/pipeline.yml
```

All output goes to the systemd journal (`journalctl -u srsilo-update`).

To set up the timer:

```bash
ansible-playbook playbooks/srsilo/setup-timer.yml -i inventory.ini
```

## Rust binaries

The binaries in `rust/` are built by Ansible (`roles/srsilo/tasks/build_tools.yml`) and are not built as part of the Python package. They are standalone Cargo workspaces. To build manually:

```bash
cd rust/
cargo build --release
# Binaries in rust/target/release/
```
