# srSILO Pipeline - Architecture

Fully automated genomic data processing: monitors LAPIS API for new SARS-CoV-2 sequences, downloads, processes, and indexes data with self-healing rollback on failures.

## Components

**Playbooks**: `setup.yml`, `update-pipeline.yml`, `setup-timer.yml`  
**Rust Tools**: check_new_data, fetch_silo_data, split_into_sorted_chunks, merge_sorted_chunks, add_offset  
**Docker**: SILO preprocessing (genspectrum/lapis-silo:0.8.5), LAPIS API (genspectrum/lapis:0.5.17)

## 7-Phase Pipeline

1. **Prerequisites**: Verify user, directories, Docker; build Rust tools
2. **Check New Data**: Query LAPIS API; exit early if no updates (0.3s)
3. **Cleanup**: Apply retention policy, detect orphans, clean temp dirs
4. **Fetch**: Download NDJSON from LAPIS, compress with zstd
5. **Prepare**: Create `.preprocessing_in_progress` marker, stop API (free memory)
6. **Process** (block/rescue):
   - Split input into sorted chunks (configurable size)
   - Merge chunks into sorted file
   - SILO preprocessing (Docker with memory limits)
7. **Finalize**:
   - **Success**: Start API with new index, update `.last_update`, cleanup markers
   - **Rollback**: Delete failed index, restart API with previous index, fail playbook

## State Files

**`.last_update`**: Unix timestamp of last successful run (persistent)  
**`.next_timestamp`**: Temp timestamp for current update (ephemeral, deleted on success/rollback)  
**`output/.preprocessing_in_progress`**: Orphan detection marker (contains failed index timestamp)

## Configuration

```yaml
srsilo_chunk_size: 30000              # Reads/chunk (lower = less RAM)
srsilo_docker_memory_limit: 6g        # Docker memory (6g for 8GB, 340g for 377GB)
srsilo_retention_days: 3              # Delete indexes older than N days
srsilo_retention_min_keep: 2          # Always keep at least M indexes
srsilo_fetch_max_reads: 5000000       # Max reads per batch
```

See `group_vars/srsilo/main.yml` (production) and `playbooks/srsilo/vars/test_vars.yml` (8GB override).