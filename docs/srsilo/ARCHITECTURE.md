# srSILO Pipeline - System Architecture

**Last Updated**: October 31, 2025  
**Status**: Production Ready (7-Phase Pipeline Complete)

---

## Executive Summary

The srSILO pipeline is a fully automated genomic data processing system that:
- Monitors LAPIS API for new SARS-CoV-2 genomic sequences
- Downloads, processes, and indexes data for efficient querying
- Maintains a local SILO database with automatic updates
- Provides self-healing capabilities with automatic rollback on failures

**Key Metrics** (Test Environment - 6GB RAM, 30GB Disk):
- Full pipeline runtime: ~2.5 minutes for 2.25M reads
- Memory-optimized chunk processing: 30K reads/chunk
- Docker preprocessing: ~82s with 6GB memory limit
- Zero-downtime updates with automatic API management

---

## System Components

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Ansible Automation                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Playbooks:                     Roles:                                  │
│  ├─ setup.yml                   └─ srsilo/                              │
│  ├─ update-pipeline.yml             ├─ tasks/                           │
│  └─ [future: rollback.yml]          │   ├─ prerequisites.yml   ✓        │
│                                     │   ├─ build_tools.yml      ✓       │
│                                     │   ├─ check_new_data.yml   ✓       │
│                                     │   ├─ fetch_data.yml        ✓      │
│                                     │   ├─ cleanup_indexes.yml   ✓      │
│                                     │   ├─ sort_and_merge.yml    ✓      │
│                                     │   ├─ silo_preprocessing.yml ✓     │
│                                     │   ├─ manage_api.yml        ✓      │
│                                     │   └─ finalize_processing.yml ✓    │
│                                     ├─ defaults/main.yml                │
│                                     ├─ handlers/main.yml                │
│                                     └─ templates/                       │
│                                          ├─ docker-compose.yml.j2       │
│                                          └─ docker-compose-preprocessing.yml.j2 │
└─────────────────────────────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      Data Processing Tools                               │
├─────────────────────────────────────────────────────────────────────────┤
│  Rust Binaries:                                                         │
│  ├─ check_new_data           Query LAPIS for updates                   │
│  ├─ fetch_silo_data          Download genomic data                     │
│  ├─ split_into_sorted_chunks Memory-efficient data splitting           │
│  ├─ merge_sorted_chunks      Merge and sort genomic reads              │
│  └─ add_offset               Adjust sequence offsets                   │
│                                                                          │
│  Docker Containers:                                                     │
│  ├─ SILO Preprocessing       genspectrum/lapis-silo:0.8.5              │
│  └─ LAPIS API                genspectrum/lapis:0.5.17                  │
└─────────────────────────────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      External Dependencies                               │
├─────────────────────────────────────────────────────────────────────────┤
│  • LAPIS API: https://api.db.wasap.genspectrum.org                     │
│  • Docker Engine + Docker Compose                                       │
│  • ZStandard compression (zstd)                                         │
│  • Rust toolchain (for building binaries)                              │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 7-Phase Update Pipeline

The `update-pipeline.yml` playbook implements a robust state machine with error recovery:

```
┌──────────────────────────────────────────────────────────────────────────┐
│ PHASE 1: Pre-flight Checks                                              │
├──────────────────────────────────────────────────────────────────────────┤
│ • Verify prerequisites (user, directories, Docker)                      │
│ • Build/verify Rust tools                                               │
│ • Display configuration                                                 │
│ Exit: Never (always runs)                                               │
└──────────────────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────────────────┐
│ PHASE 2: Check for New Data                                             │
├──────────────────────────────────────────────────────────────────────────┤
│ • Query LAPIS API for new submissions/revocations                       │
│ • Compare with .last_update timestamp                                   │
│ • Write .next_timestamp for potential update                            │
│                                                                          │
│ Decision Point:                                                          │
│   NO NEW DATA → End playbook gracefully (cost optimization)             │
│   NEW DATA    → Continue to Phase 3                                     │
└──────────────────────────────────────────────────────────────────────────┘
                            ↓ (new data detected)
┌──────────────────────────────────────────────────────────────────────────┐
│ PHASE 3: Pre-processing Cleanup                                         │
├──────────────────────────────────────────────────────────────────────────┤
│ • Apply retention policy (delete old indexes)                           │
│ • Detect orphaned indexes from previous failed runs                     │
│ • Clean working directories (sorted_chunks/, tmp/)                      │
│ Exit: On cleanup failure                                                │
└──────────────────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────────────────┐
│ PHASE 4: Fetch Data                                                     │
├──────────────────────────────────────────────────────────────────────────┤
│ • Download NDJSON files from LAPIS API                                  │
│ • Compress with ZStandard                                               │
│ • Store in input/ directory                                             │
│ Exit: On download failure                                               │
└──────────────────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────────────────┐
│ PHASE 5: Prepare for Processing                                         │
├──────────────────────────────────────────────────────────────────────────┤
│ • Create .preprocessing_in_progress marker (timestamp)                  │
│ • Stop SILO API to free memory                                          │
│ Exit: On API stop failure                                               │
└──────────────────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────────────────┐
│ PHASE 6: Process Data (CRITICAL SECTION)                                │
├──────────────────────────────────────────────────────────────────────────┤
│ Part A: Sort and Merge                                                  │
│   • Split input files into sorted chunks (configurable size)            │
│   • Merge chunks into single sorted file                                │
│                                                                          │
│ Part B: SILO Preprocessing                                              │
│   • Run Docker container with memory limits                             │
│   • Create timestamped index directory                                  │
│   • Generate SILO database files                                        │
│                                                                          │
│ Error Handling: Block/Rescue pattern                                    │
│   SUCCESS → Continue to Phase 7a                                        │
│   FAILURE → Continue to Phase 7b (rollback)                             │
└──────────────────────────────────────────────────────────────────────────┘
                   ↓                              ↓
        ┌──────────────────┐         ┌──────────────────────┐
        │ PHASE 7a: SUCCESS│         │ PHASE 7b: ROLLBACK   │
        └──────────────────┘         └──────────────────────┘
                   ↓                              ↓
┌────────────────────────────┐  ┌──────────────────────────────────┐
│ • Verify new index exists  │  │ • Read failed index from marker  │
│ • Remove preprocessing     │  │ • Delete failed/partial index    │
│   marker                   │  │ • Remove preprocessing marker    │
│ • Cleanup Docker resources │  │ • Restart API with PREVIOUS      │
│ • Start API with NEW index │  │   good index                     │
│ • Wait for health check    │  │ • Clean .next_timestamp          │
│ • Update .last_update      │  │ • Display failure summary        │
│   from .next_timestamp     │  │ • FAIL playbook (but API runs)   │
│ • Remove .next_timestamp   │  │                                  │
│ • SUCCESS ✓                │  │ • FAILED ✗                       │
└────────────────────────────┘  └──────────────────────────────────┘
```

---

## Data Flow

```
External API
    │
    │ check_new_data (Phase 2)
    │ fetch_data (Phase 4)
    ▼
┌─────────────────────────────────────────┐
│ input/                                  │  Raw downloaded data
│ └─ *.ndjson.zst                         │  (~24 MB compressed)
└────────────┬────────────────────────────┘
             │ split_into_sorted_chunks (Phase 6)
             │ chunk_size: 30000 (6GB RAM)
             ▼
┌─────────────────────────────────────────┐
│ sorted_chunks/<filename>/              │  Sorted chunks
│ ├─ chunks.list                          │  (~75 chunks)
│ └─ chunk_*.ndjson                       │
└────────────┬────────────────────────────┘
             │ merge_sorted_chunks (Phase 6)
             ▼
┌─────────────────────────────────────────┐
│ sorted.ndjson.zst                       │  Merged sorted data
│                                         │  (~23 MB compressed)
└────────────┬────────────────────────────┘
             │ Docker SILO preprocessing (Phase 6)
             │ Memory limit: 6g
             ▼
┌─────────────────────────────────────────┐
│ output/<timestamp>/                     │  SILO Index
│ ├─ data_version.silo                    │  e.g., 1761903365/
│ ├─ database_schema.silo                 │
│ └─ default/                             │
│    └─ P0.silo                           │
└────────────┬────────────────────────────┘
             │ Start SILO API (Phase 7a)
             ▼
┌─────────────────────────────────────────┐
│ LAPIS API (Docker)                      │  Query interface
│ Port: 8083                              │  http://localhost:8083
│ Status: healthy                         │
└─────────────────────────────────────────┘
```

---

## State Management & Checkpoints

### Timestamp Files

```
/opt/srsilo/.last_update
├─ Format: Unix timestamp (seconds)
├─ Purpose: Track last successful pipeline completion
├─ Updated: Phase 7a (after successful API start)
├─ Used by: Phase 2 (check_new_data as baseline)
├─ Lifecycle: Persistent across runs
└─ Example: 1729000000 (Oct 15, 2024 13:46:40 UTC)

/opt/srsilo/.next_timestamp
├─ Format: Unix timestamp (seconds)
├─ Purpose: Temp storage for next update timestamp
├─ Created: Phase 2 (when new data detected)
├─ Updated: Never (single write)
├─ Deleted: Phase 7a success OR Phase 7b rollback
├─ Lifecycle: Ephemeral (one pipeline run)
└─ Example: 1761646172 (Oct 28, 2025 10:09:32 UTC)

/opt/srsilo/output/.preprocessing_in_progress
├─ Format: Unix timestamp (directory name for failed index)
├─ Purpose: Mark in-progress preprocessing (orphan detection)
├─ Created: Phase 5 (before processing starts)
├─ Deleted: Phase 7a success OR Phase 7b rollback
├─ Used by: Phase 3 cleanup (detect orphaned indexes)
├─ Lifecycle: Should be ephemeral (persistence = failure)
└─ Example: 1761902389
```

### State Transitions

```
IDLE (No Processing)
  └─ .last_update exists
  └─ .next_timestamp absent
  └─ .preprocessing_in_progress absent
  └─ API running with latest index

CHECKING (Phase 2)
  └─ .last_update exists (baseline)
  └─ .next_timestamp created (if new data)
  └─ .preprocessing_in_progress absent
  └─ API still running

PROCESSING (Phase 5-6)
  └─ .last_update unchanged
  └─ .next_timestamp exists
  └─ .preprocessing_in_progress exists
  └─ API stopped (memory freed)

SUCCESS (Phase 7a)
  └─ .last_update = .next_timestamp (atomic update)
  └─ .next_timestamp deleted
  └─ .preprocessing_in_progress deleted
  └─ API restarted with new index

FAILED (Phase 7b)
  └─ .last_update unchanged (old timestamp preserved)
  └─ .next_timestamp deleted
  └─ .preprocessing_in_progress deleted
  └─ Failed index deleted
  └─ API restarted with PREVIOUS index
```

---

## Error Recovery Scenarios

### Scenario 1: No New Data
**Trigger**: Phase 2 finds no updates  
**Behavior**: Playbook ends gracefully via `meta: end_play`  
**Impact**: No state changes, API keeps running  
**Cost**: Minimal (API query only, ~0.5s)  

### Scenario 2: Download Fails
**Trigger**: Phase 4 fetch_data fails  
**Behavior**: Playbook fails immediately  
**State**: 
- .preprocessing_in_progress absent (never created)
- .next_timestamp exists (orphaned)
- API still running with old data  
**Recovery**: Next run proceeds normally, overwrites .next_timestamp  

### Scenario 3: Processing Fails (OOM, Corruption)
**Trigger**: Phase 6 split/merge/preprocessing fails  
**Behavior**: Rescue block catches error → Phase 7b rollback  
**Actions**:
1. Read timestamp from .preprocessing_in_progress marker
2. Delete failed index directory
3. Remove marker file
4. Restart API with previous good index
5. Clean .next_timestamp
6. Fail playbook with error details  
**State After**: API running with old data, system stable  

### Scenario 4: Orphaned Index (Previous Run Crashed)
**Trigger**: Phase 3 detects .preprocessing_in_progress marker  
**Behavior**: Cleanup task automatically removes orphan  
**Actions**:
1. Read timestamp from marker
2. Delete directory `output/<timestamp>/`
3. Remove marker file  
**Impact**: Automatic cleanup, no manual intervention needed  

### Scenario 5: API Start Fails
**Trigger**: Phase 7a docker-compose fails  
**Behavior**: Playbook fails (no rescue at Phase 7a level)  
**State**: 
- New index exists but unused
- Marker removed
- API not running  
**Recovery**: Manual intervention required, restart API with any valid index  

---

## Performance Characteristics

### Resource Requirements

**Minimum** (Test Environment):
- RAM: 6GB (4GB for Docker, 2GB for system/Rust tools)
- Disk: 30GB (includes logs, intermediate files, multiple indexes)
- CPU: 2 cores (ARM64 or x86_64)
- Network: Stable connection to LAPIS API

**Recommended** (Production):
- RAM: 350GB (for full preprocessing without memory constraints)
- Disk: 500GB (long retention, many indexes)
- CPU: 8+ cores (parallel processing)
- Network: 1Gbps+ for large dataset downloads

### Tuning Parameters

```yaml
# Memory Management
srsilo_chunk_size: 30000              # Reads per chunk (lower = less RAM)
srsilo_docker_memory_limit: 6g        # Docker container memory

# Guidelines:
#   4GB RAM  → chunk_size: 20000, docker_memory: 3g
#   6GB RAM  → chunk_size: 30000, docker_memory: 6g
#   8GB RAM  → chunk_size: 50000, docker_memory: 7g
#   16GB+ RAM → chunk_size: 100000+, docker_memory: 350g (recommended)

# Data Retention
srsilo_retention_days: 3              # Delete indexes older than N days
srsilo_retention_min_keep: 2          # Always keep at least M indexes

# Fetch Configuration
srsilo_fetch_days: 30                 # Rolling window for data fetch
srsilo_fetch_max_reads: 5000000       # Max reads per batch
```

### Typical Runtimes (6GB RAM, 2.25M reads)

| Phase | Task | Duration | Notes |
|-------|------|----------|-------|
| 1 | Prerequisites | ~4s | User/dirs/Docker check |
| 2 | Check data | ~0.3s | API query |
| 3 | Cleanup | ~8s | Retention + orphan detection |
| 4 | Fetch | ~4s | Download 24MB (cached) |
| 5 | Prepare | ~0.3s | Stop API |
| 6a | Split | ~20s | 75 chunks @ 30K reads/chunk |
| 6b | Merge | ~24s | Sort and compress |
| 6c | Preprocess | ~82s | Docker SILO indexing |
| 7a | Finalize | ~5s | Start API, health check |
| **Total** | | **~2.5 min** | End-to-end |

---

## Directory Structure

```
/opt/srsilo/                              # Base directory (srsilo_base_path)
├── input/                                # Downloaded data (srsilo_data_input)
│   └── sampleId-*.ndjson.zst            # Raw genomic data files
│
├── sorted_chunks/                        # Temporary sorted chunks
│   └── <filename>/                       # Per-file chunk directory
│       ├── chunks.list                   # Chunk manifest
│       └── chunk_*.ndjson                # Individual sorted chunks
│
├── tmp/                                  # Temporary processing files
│   └── (cleaned between runs)
│
├── output/                               # SILO indexes (srsilo_data_output)
│   ├── <timestamp1>/                     # e.g., 1761903365/
│   │   ├── data_version.silo
│   │   ├── database_schema.silo
│   │   └── default/
│   │       └── P0.silo
│   ├── <timestamp2>/
│   ├── .preprocessing_in_progress        # Orphan detection marker
│   └── ...
│
├── tools/                                # Rust binaries & Docker configs
│   ├── Cargo.toml
│   ├── docker-compose.yml               # LAPIS API configuration
│   ├── docker-compose-preprocessing.yml # SILO preprocessing
│   ├── database_config.yaml
│   ├── preprocessing_config.yaml
│   ├── reference_genomes.json
│   ├── src/                             # Rust source code
│   │   ├── check_new_data/
│   │   ├── fetch_silo_data/
│   │   ├── split_into_sorted_chunks/
│   │   ├── merge_sorted_chunks/
│   │   └── add_offset/
│   └── target/release/                  # Compiled binaries
│       ├── check_new_data
│       ├── fetch_silo_data
│       ├── split_into_sorted_chunks
│       ├── merge_sorted_chunks
│       └── add_offset
│
├── sorted.ndjson.zst                    # Merged sorted file (intermediate)
├── .last_update                         # Last successful run timestamp
└── .next_timestamp                      # Pending update timestamp (temp)
```

---

## Logging & Monitoring

### Current Status
**Implemented**: Basic debug output via Ansible's `debug` module  
**Log Location**: Terminal output / Ansible execution logs  
**Retention**: Session-based (lost after terminal closes)  

### Planned Improvements (Next Phase)
```
/var/log/srsilo/
├── update-pipeline.log          # Main pipeline log
├── setup.log                    # Initial setup log
├── rollback.log                 # Manual rollback operations
└── archived/                    # Rotated logs (30 days)
    └── update-pipeline-2025-10-*.log.gz
```

**Features**:
- Timestamped entries (ISO 8601 format)
- Structured logging (phase markers, timing data)
- Log rotation via logrotate (30-day retention)
- Integration with journald for systemd timer runs
- Grafana-ready format for visualization

**Implementation Notes**:
- Add `log_path` callback plugin to ansible.cfg
- Implement per-playbook log files
- Add hooks for Slack/email notifications (future)

---

## Security Considerations

### User Privilege Model
```
Root (become: yes)
├─ Docker operations (container management)
├─ Directory ownership changes
├─ Log file creation in /var/log
└─ Systemd service management

srsilo User (unprivileged)
├─ Run Rust binaries
├─ Read/write data files
├─ Query LAPIS API
└─ Manage SILO data directories
```

### File Permissions
```
/opt/srsilo/              → srsilo:srsilo (755)
/opt/srsilo/input/        → srsilo:srsilo (755)
/opt/srsilo/output/       → root:root (755) - created by Docker
/var/log/srsilo/          → root:adm (750)
```

### Network Security
- LAPIS API: HTTPS only (api.db.wasap.genspectrum.org)
- Local SILO API: HTTP on localhost:8083 (not exposed externally)
- Docker networks: Bridge mode (isolated)

---

## Future Enhancements

### Phase 1: Operations & Testing (In Progress)
- [ ] Implement centralized logging
- [ ] Create rollback.yml playbook
- [ ] Add --tags for testing individual phases
- [ ] Documentation for manual operations

### Phase 2: Automation (Planned)
- [ ] Systemd timer (srsilo-update.timer)
- [ ] Schedule: Daily at 2 AM
- [ ] Failure notifications (journald → future: Slack/email)

### Phase 3: Monitoring (Future)
- [ ] Grafana dashboards for pipeline metrics
- [ ] Prometheus exporter for SILO API stats
- [ ] Alert rules for failures
- [ ] Historical trend analysis

### Phase 4: Scalability (Future)
- [ ] Multi-host deployment support
- [ ] Distributed processing for large datasets
- [ ] S3 backup integration
- [ ] Geographic replication

---

## Related Documentation

- **Implementation Plan**: See `docs/srsilo/IMPLEMENTATION_PLAN.md`
- **Role README**: See `roles/srsilo/README.md`
- **Variable Reference**: See `roles/srsilo/defaults/main.yml`
- **Playbook Usage**: See individual playbook headers in `playbooks/srsilo/`

---

## Maintenance & Support

**Primary Maintainer**: WisePulse Team  
**Repository**: https://github.com/cbg-ethz/WisePulse  
**Pull Request**: #95 (refactor/srSILO branch)  

**Change Log**:
- 2025-10-31: 7-phase pipeline complete, retention policy fixed
- 2025-10-30: Initial Ansible conversion from Makefile
- 2024-10-15: Original Makefile-based implementation
