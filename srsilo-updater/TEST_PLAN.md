# srsilo-updater Test Plan

## Unit tests

### config.py
- [ ] Valid `pipeline.yml` loads correctly into typed dataclasses
- [ ] Missing required fields raise a descriptive error
- [ ] `VirusPaths` properties return correct paths relative to `base_path`
- [ ] `PipelineConfig.binaries()` returns `tools_path/target/release`

### phases/check_new_data.py
- [ ] Returns `True` when binary exits 0
- [ ] Returns `False` when binary exits 1
- [ ] Raises `RuntimeError` when binary exits 2
- [ ] Correct CLI arguments are passed to the binary

### phases/cleanup.py
- [ ] Retention: deletes only indexes older than `retention_days` AND beyond `min_keep` count
- [ ] Retention: never deletes if total count <= `min_keep`
- [ ] Retention: never deletes if all indexes are within `retention_days`
- [ ] Orphan cleanup: reads marker, deletes the matching directory, removes marker
- [ ] Orphan cleanup: no-ops if marker does not exist
- [ ] Working dir reset: `input`, `sorted_chunks`, `tmp` are deleted and recreated

### phases/fetch.py
- [ ] Correct CLI arguments passed to `fetch_silo_data`
- [ ] Raises `RuntimeError` if no `.ndjson.zst` files present after run
- [ ] `start_date` is today in YYYY-MM-DD format

### phases/sort_and_merge.py
- [ ] Each input file is piped through `zstdcat | split_into_sorted_chunks`
- [ ] Chunk paths are written to `chunks.list`
- [ ] `merge_sorted_chunks` reads from `chunks.list` and pipes output through `zstd`
- [ ] Raises `RuntimeError` if no input files present

### phases/api.py
- [ ] `stop` calls `docker compose -p <virus> down`
- [ ] `start` calls `docker compose -p <virus> up -d` then polls health endpoint
- [ ] Health poll retries up to `_HEALTH_RETRIES` times with `_HEALTH_DELAY` sleep
- [ ] Health poll logs warning (not error) if API does not become ready — does not raise

### phases/finalize.py (success path)
- [ ] Identifies the newest index directory by mtime
- [ ] Removes preprocessing marker
- [ ] Calls `api.stop` then `api.start`
- [ ] Copies `.next_timestamp` to `.last_update` and deletes `.next_timestamp`
- [ ] Raises `RuntimeError` if no index directory exists after preprocessing

### phases/finalize.py (rollback path)
- [ ] Reads timestamp from preprocessing marker
- [ ] Deletes the failed index directory
- [ ] Removes marker and `.next_timestamp`
- [ ] Calls `api.start` with the remaining newest index
- [ ] Logs warning (not error) if no previous index to roll back to

### `__main__.py`
- [ ] Loops over `enabled_viruses` when `--virus` not specified
- [ ] Processes only the named virus when `--virus` is given
- [ ] Exits 0 when all viruses succeed
- [ ] Exits 1 when any virus fails, and still processes remaining viruses
- [ ] Unknown virus name in `--virus` is logged and counted as failure

---

## Integration tests (using test_data/)

The `tests/data/` directory contains real `.ndjson.zst` sample files:

- [ ] Full sort-and-merge round trip: feed test files through `split_into_sorted_chunks` + `merge_sorted_chunks`, verify output is a valid `.ndjson.zst` with expected record count
- [ ] `fetch_silo_data` + sort-and-merge end-to-end using a small live API fetch (covid, 1 day, low max_reads)

---

## System / smoke tests (requires `/opt/srsilo` and Docker)

- [ ] `pipeline-test.yml` loads without error: `python -m pipeline --config pipeline-test.yml --help`
- [ ] Ansible `setup.yml` creates the venv and installs the package correctly
- [ ] `python -m pipeline --config /opt/srsilo/pipeline.yml --virus covid` runs to completion
- [ ] After a successful run, `.last_update` is updated and the SILO API responds on the configured port
- [ ] Simulated failure: delete `sorted.ndjson.zst` mid-run → rollback fires, previous index stays running
- [ ] Systemd timer: `systemctl start srsilo-update.service` completes without error

---

## Regression checks (things that broke under the old Ansible pipeline)

- [ ] `fetch_days` window correctly covers the actual data age (regression: 45-day default missed 92-day-old data)
- [ ] `fetch_max_reads` is above the maximum single-day read count (regression: 5M limit < single-day 5.06M, yielding 0 downloads)
- [ ] Binary stdout is visible in logs and errors are caught (regression: `systemd-cat` pipe hid fetch failures)
