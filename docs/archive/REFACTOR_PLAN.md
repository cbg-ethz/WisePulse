# srSILO Ansible Refactor - Implementation Plan

## Overview
Converting the srSILO pipeline from a Make-based system to an Ansible-based orchestration, with playbooks serving as entry points and role tasks handling the actual work.

## Architecture

### Directory Structure
```
playbooks/srsilo/
├── setup.yml                 # Initial setup (implemented ✓)
├── update-pipeline.yml        # Full automated pipeline (TODO)
├── fetch.yml                 # Manual fetch-only operation (TODO)
├── process.yml               # Manual process-only operation (TODO)
└── cleanup.yml               # Maintenance/retention (TODO)

roles/srsilo/
├── tasks/
│   ├── prerequisites.yml     # User/dirs/permissions (basic)
│   ├── build_tools.yml       # Compile Rust binaries (implemented ✓)
│   ├── check_new_data.yml    # Query LAPIS API (implemented ✓)
│   ├── fetch_data.yml        # Fetch from API (TODO)
│   ├── process_data.yml      # Split & merge chunks (TODO)
│   ├── silo_preprocessing.yml # Docker preprocessing (TODO)
│   ├── manage_api.yml        # Start/stop SILO API (TODO)
│   └── cleanup_indexes.yml   # Retention policy (TODO)
```

## Makefile → Ansible Mapping

### Phase 1: Data Operations (Current Focus)
| Make Target | Type | Mapped To | Status |
|-------------|------|-----------|--------|
| `build` | Build | `build_tools.yml` | ✓ Done |
| `fetch-data` | Fetch | `fetch_data.yml` | 🔄 TODO |
| `all` | Process | `process_data.yml` + `silo_preprocessing.yml` | 🔄 TODO |
| `cleanup-old-indexes` | Maintenance | `cleanup_indexes.yml` | 🔄 TODO |

### Phase 2: Playbook Orchestration
| Playbook | Replaces | Purpose | Status |
|----------|----------|---------|--------|
| `setup.yml` | Manual setup | Initial infrastructure | ✓ Done |
| `fetch.yml` | `make fetch-and-process` (partial) | Fetch without processing | 🔄 TODO |
| `process.yml` | `make all` | Process existing data | 🔄 TODO |
| `update-pipeline.yml` | `make update-fetch-and-process` | Full update pipeline | 🔄 TODO |
| `cleanup.yml` | `make cleanup-old-indexes` | Maintenance only | 🔄 TODO |

## Detailed Task Design

### 1. fetch_data.yml (Role Task)
**Replaces**: `make fetch-data`

**Purpose**: Download genomic data from LAPIS API

**Key Features**:
- Runs as `srsilo_user` (unprivileged)
- Configurable date range and batch size
- Fetches NDJSON data and compresses with ZStandard
- Output goes to `srsilo_data_input` directory

**Variables (from defaults/main.yml)**:
```yaml
srsilo_fetch_days: 90              # How many days back to fetch
srsilo_fetch_max_reads: 125000000  # Max reads per batch
srsilo_api_base_url: ...           # LAPIS API endpoint
srsilo_data_input: /opt/srsilo/input
```

**Implementation**:
```yaml
- name: Fetch data from LAPIS API
  command: >
    {{ srsilo_tools_path }}/target/release/fetch_silo_data
    --start-date "{{ fetch_start_date | default(ansible_date_time.iso8601_date) }}"
    --days {{ srsilo_fetch_days }}
    --max-reads {{ srsilo_fetch_max_reads }}
    --output-dir "{{ srsilo_data_input }}"
    --api-base-url "{{ srsilo_api_base_url }}"
  become: yes
  become_user: "{{ srsilo_user }}"
  register: fetch_result
```

**Exit Behavior**:
- Task fails if fetch returns non-zero
- Sets fact with fetch summary (files downloaded, size)
- Displays progress via debug output

---

### 2. process_data.yml (Role Task)
**Replaces**: `make all` / `$(SORTED_FILE)` + `$(SILO_OUTPUT_FLAG)`

**Purpose**: Process input data through the pipeline (split → merge → preprocess)

**Key Steps** (in sequence):
1. Create working directories (sorted_chunks, tmp, silo_output)
2. Split input files into sorted chunks
3. Merge sorted chunks
4. Run SILO preprocessing via Docker

**Variables**:
```yaml
srsilo_data_sorted_chunks: /opt/srsilo/sorted_chunks
srsilo_data_tmp: /opt/srsilo/tmp
srsilo_data_output: /opt/srsilo/output
```

**Implementation Strategy**:
- Create separate subtasks for each step
- Use handlers to clean up on failure
- Check Docker availability before preprocessing
- Register key artifacts (chunk count, merged file size)

---

### 3. manage_api.yml (Role Task)
**Replaces**: `docker compose up/down` commands

**Purpose**: Control SILO API container lifecycle

**Operations**:
- Start API: `docker-compose up -d` with `LAPIS_PORT`
- Stop API: `docker-compose down`
- Health check: curl API endpoint
- Cleanup orphaned networks/volumes

**Variables**:
```yaml
srsilo_lapis_port: 8083
```

---

### 4. cleanup_indexes.yml (Role Task)
**Replaces**: `make cleanup-old-indexes`

**Purpose**: Implement retention policy

**Logic**:
- Find indexes older than N days
- Keep at least M newest indexes (never delete all)
- Use Ansible's `find` module with sorting

**Variables**:
```yaml
srsilo_retention_days: 7      # Delete if older than 7 days
srsilo_retention_min_keep: 2  # But always keep 2 newest
```

---

## Playbook Design

### setup.yml (✓ Done)
```yaml
hosts: srsilo
tasks:
  - prerequisites     # User + dirs + perms
  - build_tools       # Compile binaries
  - deploy_configs    # Config files
```

### fetch.yml (TODO)
```yaml
hosts: srsilo
tasks:
  - check_new_data (optional)
  - fetch_data
  - display summary
```

### process.yml (TODO)
```yaml
hosts: srsilo
tasks:
  - check if input data exists
  - cleanup old artifacts
  - process_data       # Split → Merge → Preprocess
  - display summary
```

### update-pipeline.yml (TODO)
```yaml
hosts: srsilo
tasks:
  - check_new_data     # Exit if no new data
  - cleanup_old_indexes
  - manage_api (stop)
  - fetch_data
  - process_data
  - manage_api (start)
  - manage_api (health check)
  - update checkpoint file
```

### cleanup.yml (TODO)
```yaml
hosts: srsilo
tasks:
  - cleanup_indexes
  - optional: clean-data
  - optional: clean-all
```

---

## Implementation Priority

### Phase 1: Core Data Tasks (This Sprint)
1. **fetch_data.yml** ← START HERE
   - Wraps `fetch_silo_data` binary
   - Minimal complexity
   - Can test immediately
   
2. **process_data.yml**
   - Orchestrates split → merge → preprocess
   - More complex, multi-step
   - Needs Docker integration

3. **manage_api.yml**
   - Control Docker compose
   - Health checks
   - Required by update pipeline

### Phase 2: Playbook Orchestration
4. **fetch.yml** playbook
5. **process.yml** playbook
6. **update-pipeline.yml** playbook
7. **cleanup.yml** playbook

### Phase 3: CI/CD & Testing
8. Update `.github/workflows/ci.yml` end-to-end tests
9. Add integration tests for full pipeline
10. Documentation & usage examples

---

## Testing Strategy

### Unit Tests (Per Task)
```bash
# Test fetch_data task
ansible localhost -m include_role -a "name=srsilo tasks_from=fetch_data"

# Test process_data task
ansible localhost -m include_role -a "name=srsilo tasks_from=process_data"
```

### Integration Tests (Per Playbook)
```bash
# Test fetch playbook
ansible-playbook playbooks/srsilo/fetch.yml -i inventory.ini

# Test update pipeline
ansible-playbook playbooks/srsilo/update-pipeline.yml -i inventory.ini
```

### End-to-End Tests
- Update `ci.yml` workflow to test full pipeline
- Validate all binaries are created
- Verify data flow through each stage

---

## Variable Hierarchy

**Precedence** (highest to lowest):
1. Playbook variables (`playbooks/srsilo/vars/`)
2. Inventory host variables (`host_vars/`)
3. Inventory group variables (`group_vars/srsilo/`)
4. Role defaults (`roles/srsilo/defaults/main.yml`)

**Runtime Overrides**:
```bash
# Override via CLI
ansible-playbook ... -e "srsilo_fetch_days=30" -e "srsilo_fetch_max_reads=50000000"

# Override via playbook
tasks:
  - include_role:
      name: srsilo
      tasks_from: fetch_data
    vars:
      srsilo_fetch_days: 30
```

---

## Known Issues & Considerations

1. **Privilege Escalation**
   - Using `allow_world_readable_tmpfiles = True` in ansible.cfg
   - ✓ Tested and working with `become: yes` + `become_user`

2. **Docker Compose**
   - Runs as root (via become: yes)
   - Monitor for permission issues with volumes

3. **Timestamp Management**
   - `.last_update` file tracks last successful run
   - `.next_timestamp` is temp file during processing
   - Must handle cleanup on failure

4. **Error Handling**
   - Failed tasks should not leave system in inconsistent state
   - Implement rollback handlers for cleanup
   - Log all operations for debugging

---

## Next Steps

1. ✅ Implement `fetch_data.yml` task
2. Test locally with sample data
3. Implement `process_data.yml` task
4. Implement `manage_api.yml` task
5. Create playbooks (fetch.yml, process.yml, update-pipeline.yml, cleanup.yml)
6. Update CI/CD workflow
7. Write documentation

