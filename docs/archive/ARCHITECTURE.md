# srSILO Ansible Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Ansible Playbooks                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐  │
│  │  setup.yml       │    │  fetch.yml       │    │  update-pipeline  │  │
│  │  (init setup)    │    │  (manual fetch)  │    │  (full auto)     │  │
│  └────────┬─────────┘    └────────┬─────────┘    └────────┬─────────┘  │
│           │                      │                        │             │
└───────────┼──────────────────────┼────────────────────────┼─────────────┘
            │                      │                        │
            ▼                      ▼                        ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      srSILO Role Tasks                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ✓ prerequisites.yml      - User/group/directories                     │
│  ✓ build_tools.yml         - Compile Rust binaries                     │
│  ✓ check_new_data.yml      - Query LAPIS API                          │
│  ✓ fetch_data.yml          - Download data                            │
│  🔄 process_data.yml       - Split/merge/preprocess (TODO)            │
│  🔄 manage_api.yml         - Start/stop SILO API (TODO)               │
│  🔄 cleanup_indexes.yml    - Retention policy (TODO)                  │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      External Components                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  • LAPIS API (api.db.wasap.genspectrum.org)                            │
│  • Rust binaries (split, merge, fetch, check, add_offset)              │
│  • Docker/Docker Compose (SILO preprocessing)                          │
│  • Local filesystem (input/output/tmp/sorted_chunks)                   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Update Pipeline - 7 Phase State Machine

```
START
  │
  ├─ PHASE 1: Pre-flight Checks ─────────────────────────────────────────
  │  ├─ Verify prerequisites (user, dirs, permissions)
  │  ├─ Build tools if needed
  │  └─ Display configuration
  │
  ├─ PHASE 2: Check for New Data ────────────────────────────────────────
  │  ├─ Query LAPIS API
  │  │
  │  ├─ NO NEW DATA ─────────┐
  │  │  └─ Exit gracefully   │  (saves money, skip expensive ops)
  │  │                       │
  │  └─ NEW DATA ───────┐    │
  │     Continue ◄──────┴────┘
  │
  ├─ PHASE 3: Pre-processing Cleanup ────────────────────────────────────
  │  ├─ Run retention policy (cleanup old indexes)
  │  ├─ Detect/clean orphaned indexes from previous failures
  │  ├─ Remove old downloaded data
  │  └─ Reset working directories
  │
  ├─ PHASE 4: Fetch Data ────────────────────────────────────────────────
  │  ├─ Download from LAPIS API
  │  └─ Verify files downloaded
  │
  ├─ PHASE 5: Prepare for Processing ────────────────────────────────────
  │  ├─ Create state file: .preprocessing_in_progress
  │  └─ Stop SILO API (free resources during heavy processing)
  │
  ├─ PHASE 6: Process Data (CRITICAL SECTION) ──────────────────────────
  │  ├─ Run: process_data task
  │  │  ├─ Split files into sorted chunks
  │  │  ├─ Merge sorted chunks
  │  │  └─ SILO preprocessing via Docker
  │  │
  │  ├─ SUCCESS ─────────────┐
  │  │  Continue to Phase 7a  │
  │  │                        │
  │  └─ FAILURE ──────────┐   │
  │     Rescue Error  │   │   │
  │        Continue ◄──┼───────┘
  │     to Phase 7b  │
  │
  ├─ PHASE 7a: SUCCESS - Restart API ────────────────────────────────────
  │  ├─ Detect new index directory
  │  ├─ Remove .preprocessing_in_progress
  │  ├─ Clean Docker networks/volumes
  │  ├─ Start SILO API with new index
  │  ├─ Update checkpoint: .next_timestamp → .last_update
  │  └─ DONE ✓
  │
  └─ PHASE 7b: FAILURE - Rollback ──────────────────────────────────────
     ├─ Identify failed index directory
     ├─ Delete failed index
     ├─ Remove .preprocessing_in_progress
     ├─ Restart SILO API with PREVIOUS GOOD INDEX
     ├─ Clean up .next_timestamp
     └─ FAIL ✗ (but API still running with old data)

END
```

---

## Data Flow Diagram

```
┌──────────────────┐
│  LAPIS API       │  (External)
│  db.wasap.org    │
└────────┬─────────┘
         │ check_new_data
         │ fetch_data (Phase 4)
         │
         ▼
    ┌─────────────────────────────────────┐
    │  {{ srsilo_data_input }}            │ (Phase 4)
    │  /opt/srsilo/input                  │
    │  └─ *.ndjson.zst files              │
    └────────┬────────────────────────────┘
             │ split_into_sorted_chunks
             │ (Phase 6)
             │
             ▼
    ┌─────────────────────────────────────┐
    │  {{ srsilo_data_sorted_chunks }}    │ (Phase 6)
    │  /opt/srsilo/sorted_chunks          │
    │  └─ chunks.list                     │
    │  └─ chunk_*.ndjson                  │
    └────────┬────────────────────────────┘
             │ merge_sorted_chunks
             │ (Phase 6)
             │
             ▼
    ┌─────────────────────────────────────┐
    │  sorted.ndjson.zst                  │ (Phase 6)
    │  /opt/srsilo/sorted.ndjson.zst      │
    └────────┬────────────────────────────┘
             │ Docker preprocessing
             │ (Phase 6)
             │
             ▼
    ┌─────────────────────────────────────┐
    │  {{ srsilo_data_output }}           │ (Phase 7a)
    │  /opt/srsilo/silo_output            │
    │  └─ <timestamp>/                    │ (new index)
    │     ├─ indexes/                     │
    │     ├─ schema/                      │
    │     └─ ...                          │
    └────────┬────────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────────┐
    │  SILO API Running                   │ (Phase 7a)
    │  Port 8083                          │
    │  ✓ Ready to serve queries           │
    └─────────────────────────────────────┘
```

---

## State Files & Checkpoint Management

```
{{ srsilo_base_path }}/.last_update
├─ Format: Unix timestamp (seconds)
├─ Updated: After successful pipeline completion
├─ Used by: check_new_data (as starting point)
├─ Persistent: YES (across runs)
└─ On failure: NOT modified (keeps last known good state)

{{ srsilo_base_path }}/.next_timestamp
├─ Format: Unix timestamp (seconds)
├─ Created by: check_new_data task
├─ Used by: To update .last_update on success
├─ Persistent: NO (deleted after use)
└─ On failure: Cleaned up automatically

{{ srsilo_data_output }}/.preprocessing_in_progress
├─ Format: ISO8601 timestamp (creation time)
├─ Created: Phase 5 (marks processing started)
├─ Used by: Orphan detection in next run's Phase 3
├─ Persistent: NO (deleted on Phase 7 success)
└─ On failure: Preserved for cleanup next run
```

---

## Error Scenarios & Recovery

### Scenario 1: No New Data
```
Phase 2 detects no changes
  ↓
Meta: end_play executed
  ↓
Playbook exits cleanly (no state change)
  ↓
API keeps running with old index
  ↓
Next run will check again
```

### Scenario 2: API Query Fails
```
Phase 2 check_new_data fails
  ↓
Task fails immediately
  ↓
API still running (Phase 5 not reached)
  ↓
Manual intervention required
```

### Scenario 3: Fetch Fails
```
Phase 4 fetch_data fails
  ↓
Task fails immediately
  ↓
API still running (not stopped yet)
  ↓
Next run will detect orphaned .preprocessing_in_progress
  ↓
Phase 3 cleanup will delete the orphan
```

### Scenario 4: Processing Fails
```
Phase 6 process_data fails (inside rescue block)
  ↓
preprocessing_failed flag set
  ↓
Phase 7b rollback triggered
  ↓
Failed index deleted
  ↓
API restarted with PREVIOUS GOOD INDEX
  ↓
Old data still available (safe state)
  ↓
Manual debugging needed
```

### Scenario 5: API Restart Fails
```
Phase 7b restart_api fails
  ↓
notify: restart api on failure triggered
  ↓
Manual intervention + handler retry
  ↓
System in uncertain state
  ↓
Requires manual debugging & recovery
```

---

## Variable Precedence

```
CLI Arguments (-e flags)
     ▲
     │ Overrides
     │
Playbook vars section
     ▲
     │ Overrides
     │
inventory/group_vars/srsilo/main.yml
     ▲
     │ Overrides
     │
roles/srsilo/defaults/main.yml
     │
     └─ Lowest priority (fallback)
```

### Critical Variables

**From CLI (highest priority)**:
```bash
-e "fetch_start_date=2025-08-01"
-e "srsilo_fetch_days=30"
-e "srsilo_retention_days=7"
```

**From group_vars (persistent config)**:
```yaml
srsilo_fetch_days: 90
srsilo_fetch_max_reads: 125000000
srsilo_api_base_url: https://api.db.wasap.genspectrum.org
srsilo_retention_days: 7
srsilo_retention_min_keep: 2
```

**From role defaults (fallback)**:
```yaml
srsilo_base_path: /opt/srsilo
srsilo_user: srsilo
srsilo_group: srsilo
```

---

## Playbook Invocation Examples

### Manual Fetch Only
```bash
ansible-playbook playbooks/srsilo/fetch.yml \
  -e "fetch_start_date=2025-08-01" \
  -i inventory.ini
```

### Full update Pipeline
```bash
ansible-playbook playbooks/srsilo/update-pipeline.yml \
  -i inventory.ini
```

### With Override Configuration
```bash
ansible-playbook playbooks/srsilo/update-pipeline.yml \
  -e "srsilo_retention_days=7" \
  -e "srsilo_retention_min_keep=3" \
  -e "srsilo_lapis_port=8083" \
  -i inventory.ini
```

### Dry-Run (No Changes)
```bash
ansible-playbook playbooks/srsilo/update-pipeline.yml \
  -i inventory.ini \
  --check
```

### Verbose Debugging
```bash
ansible-playbook playbooks/srsilo/update-pipeline.yml \
  -i inventory.ini \
  -vvv
```

### Run Specific Phase Only
```bash
ansible-playbook playbooks/srsilo/update-pipeline.yml \
  -i inventory.ini \
  --tags phase2
```

---

## Tags Available

```
phase1       - Pre-flight checks
phase2       - Check for new data
phase3       - Pre-processing cleanup
phase4       - Fetch data
phase5       - Prepare for processing
phase6       - Process data
phase7a      - Success path
phase7b      - Failure path

prerequisites  - User/group/dirs
build          - Compile tools
check          - Query API
fetch          - Download data
process        - Process pipeline
cleanup        - Retention policy
manage_api     - API lifecycle
state_mgmt     - Checkpoint files
```

Example usage:
```bash
ansible-playbook ... --tags "phase2,phase3"
ansible-playbook ... --skip-tags "cleanup"
```

