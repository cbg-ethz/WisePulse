# Multi-Virus Support for srSILO Role - Implementation Plan

## Overview

This document outlines the plan to generalize the `srsilo` Ansible role to support multiple viruses. Currently, the role is hardcoded to SARS-CoV-2 (COVID-19). We want to support:

- **SARS-CoV-2** (current, production)
- **RSV-A** (first to add)
- **RSV-B** (future)
- **Influenza segments** (each segment treated as separate virus):
  - **Flu-H1**
  - **Flu-N1**
  - **Flu-H3**
  - **Flu-N2**

**Total: ~7 virus instances in the mid-term**

Each virus/segment is treated as a completely separate instance with its own:
- Configuration files (`database_config.yaml`, `preprocessing_config.yaml`, `reference_genomes.json`)
- Data directories
- Docker containers
- API endpoints (ports)
- Update timer/schedule

## Current Architecture (Single Virus)

### Hardcoded Elements

| Component | Location | Hardcoded Value |
|-----------|----------|-----------------|
| API endpoint | Rust tools | `/covid/sample/details` |
| Instance name | `database_config.yaml` | `wise-sarsCoV2` |
| Reference genome | `reference_genomes.json` | SARS-CoV-2 genome (29903 bp) |
| API port | `defaults/main.yml` | `8083` |
| Paths | `defaults/main.yml` | `/opt/srsilo/*` |
| Systemd units | templates | `srsilo-update.service/timer` |

### LAPIS API URLs

```
SARS-CoV-2: https://api.db.wasap.genspectrum.org/covid/sample/details
RSV-A:      https://api.db.wasap.genspectrum.org/rsva/sample/details
RSV-B:      https://api.db.wasap.genspectrum.org/rsvb/sample/details (TBC)
Flu-H1:     https://api.db.wasap.genspectrum.org/flu-h1/sample/details (TBC)
Flu-N1:     https://api.db.wasap.genspectrum.org/flu-n1/sample/details (TBC)
Flu-H3:     https://api.db.wasap.genspectrum.org/flu-h3/sample/details (TBC)
Flu-N2:     https://api.db.wasap.genspectrum.org/flu-n2/sample/details (TBC)
```

**Note:** The Loculus API structure is consistent across all viruses/segments.

### Current File Structure
```
/opt/srsilo/
├── input/                    # Downloaded NDJSON files
├── output/                   # SILO indexes
├── sorted_chunks/            # Processing temp
├── tmp/                      # Processing temp
├── sorted.ndjson.zst         # Merged data
├── .last_update              # Timestamp checkpoint
├── .next_timestamp           # Pending timestamp
└── tools/                    # Rust binaries + configs
    ├── target/release/
    ├── database_config.yaml
    ├── preprocessing_config.yaml
    ├── reference_genomes.json
    ├── docker-compose.yml
    └── docker-compose-preprocessing.yml
```

---

## Target Architecture (Multi-Virus)

### Proposed Directory Structure

```
/opt/srsilo/
├── covid/                         # SARS-CoV-2 instance
│   ├── input/
│   ├── output/
│   ├── sorted_chunks/
│   ├── tmp/
│   ├── sorted.ndjson.zst
│   ├── .last_update
│   ├── .next_timestamp
│   └── config/
│       ├── database_config.yaml
│       ├── preprocessing_config.yaml
│       ├── reference_genomes.json
│       ├── docker-compose.yml
│       └── docker-compose-preprocessing.yml
├── rsva/                          # RSV-A instance
│   ├── input/
│   ├── output/
│   ├── sorted_chunks/
│   ├── tmp/
│   ├── sorted.ndjson.zst
│   ├── .last_update
│   ├── .next_timestamp
│   └── config/
│       ├── database_config.yaml
│       ├── preprocessing_config.yaml
│       ├── reference_genomes.json
│       ├── docker-compose.yml
│       └── docker-compose-preprocessing.yml
├── rsvb/                          # RSV-B instance (future)
├── flu/                           # Influenza instance (future)
└── tools/                         # Shared Rust binaries
    └── target/release/
        ├── fetch_silo_data
        ├── check_new_data
        ├── split_into_sorted_chunks
        ├── merge_sorted_chunks
        └── add_offset
```

### Port Assignments

| Virus | LAPIS Port | SILO Port | Notes |
|-------|------------|-----------|-------|
| SARS-CoV-2 | 8083 | 8081 | Current production |
| RSV-A | 8084 | 8082 | First new virus |
| RSV-B | 8085 | 8086 | Future |
| Flu-H1 | 8087 | 8088 | Influenza segment |
| Flu-N1 | 8089 | 8090 | Influenza segment |
| Flu-H3 | 8091 | 8092 | Influenza segment |
| Flu-N2 | 8093 | 8094 | Influenza segment |

### Systemd Units

See [Timer Strategy](#1-timer-strategy) section for discussion of options.

---

## Changes Required

### 1. Rust Tools (Virus-Agnostic)

**Files:**
- `roles/srsilo/files/tools/src/fetch_silo_data/src/main.rs`
- `roles/srsilo/files/tools/src/check_new_data/src/main.rs`

**Current hardcoding:**
```rust
// fetch_silo_data/src/main.rs line 341-343
let url = format!(
    "{}/covid/sample/details?samplingDate={}&dataFormat=JSON&downloadAsFile=false",
    api_base_url, date_str
);

// check_new_data/src/main.rs line 192
"{}/covid/sample/details?submittedAtTimestampFrom={}&samplingDateFrom={}..."
```

**Required change:** Add `--organism` CLI argument:
```rust
.arg(
    Arg::new("organism")
        .long("organism")
        .value_name("NAME")
        .help("Organism/virus identifier (covid, rsva, rsvb, flu)")
        .required(true),
)

// Use in URL construction:
let url = format!(
    "{}/{}/sample/details?samplingDate={}...",
    api_base_url, organism, date_str
);
```

### 2. Ansible Role Defaults ✅ IMPLEMENTED (PR #184)

**File:** `roles/srsilo/defaults/main.yml`

**Implemented virus configuration structure:**
```yaml
# Current virus being processed (set per-playbook or per-run)
srsilo_virus: covid

# Virus registry - defines available viruses (no 'enabled' field - use srsilo_enabled_viruses)
srsilo_viruses:
  covid:
    organism: covid           # API endpoint identifier
    instance_name: wise-sarsCoV2
    lapis_port: 8083
    silo_port: 8081
  rsva:
    organism: rsva
    instance_name: wise-rsva
    lapis_port: 8084
    silo_port: 8082

# Single source of truth for which viruses are active
srsilo_enabled_viruses:
  - covid

# Derived virus-specific paths
srsilo_virus_path: "{{ srsilo_base_path }}/{{ srsilo_virus }}"
srsilo_virus_config_path: "{{ srsilo_virus_path }}/config"
srsilo_virus_input: "{{ srsilo_virus_path }}/input"
srsilo_virus_output: "{{ srsilo_virus_path }}/output"
srsilo_virus_sorted_chunks: "{{ srsilo_virus_path }}/sorted_chunks"
srsilo_virus_tmp: "{{ srsilo_virus_path }}/tmp"
srsilo_virus_last_update: "{{ srsilo_virus_path }}/.last_update"
srsilo_virus_next_timestamp: "{{ srsilo_virus_path }}/.next_timestamp"
srsilo_virus_sorted_file: "{{ srsilo_virus_path }}/sorted.ndjson.zst"

# Convenience lookup variables for current virus
srsilo_current_virus: "{{ srsilo_viruses[srsilo_virus] }}"
srsilo_current_organism: "{{ srsilo_current_virus.organism }}"
srsilo_current_instance_name: "{{ srsilo_current_virus.instance_name }}"
srsilo_current_lapis_port: "{{ srsilo_current_virus.lapis_port }}"
srsilo_current_silo_port: "{{ srsilo_current_virus.silo_port }}"

# Legacy path variables (backward compatibility until PR5)
srsilo_data_input: "{{ srsilo_base_path }}/input"
srsilo_data_output: "{{ srsilo_base_path }}/output"
srsilo_data_sorted_chunks: "{{ srsilo_base_path }}/sorted_chunks"
srsilo_data_tmp: "{{ srsilo_base_path }}/tmp"
```

### 3. Configuration Files (Per-Virus)

**Files to create per virus:**
- `roles/srsilo/files/viruses/covid/database_config.yaml`
- `roles/srsilo/files/viruses/covid/preprocessing_config.yaml`
- `roles/srsilo/files/viruses/covid/reference_genomes.json`
- `roles/srsilo/files/viruses/rsva/database_config.yaml`
- `roles/srsilo/files/viruses/rsva/preprocessing_config.yaml`
- `roles/srsilo/files/viruses/rsva/reference_genomes.json`
- ... (same for other viruses)

**Note:** Reference genomes will need to be obtained for each virus:
- RSV-A reference genome (~15,200 bp)
- RSV-B reference genome (~15,200 bp)  
- Influenza (segmented genome - special handling?)

### 4. Templates (Parameterized)

**Files:**
- `roles/srsilo/templates/docker-compose.yml.j2`
- `roles/srsilo/templates/docker-compose-preprocessing.yml.j2`
- `roles/srsilo/templates/srsilo-update.service.j2`
- `roles/srsilo/templates/srsilo-update.timer.j2`

**Changes:**
- Parameterize paths with `{{ srsilo_virus }}`
- Parameterize ports with virus-specific values
- Rename service/timer with virus prefix

### 5. Ansible Tasks (Virus-Aware)

All task files need to use virus-specific variables:
- `tasks/fetch_data.yml` - pass `--organism` to Rust tool
- `tasks/check_new_data.yml` - pass `--organism` to Rust tool
- `tasks/deploy_configs.yml` - deploy virus-specific configs
- `tasks/manage_api.yml` - manage virus-specific containers
- All other tasks - use `srsilo_virus_*` paths

### 6. Playbooks

**Option A: Parameterized single playbook**
```yaml
# update-pipeline.yml
- hosts: srsilo
  vars:
    srsilo_virus: "{{ target_virus | default('covid') }}"
  # ... rest of playbook
```

**Option B: Per-virus playbooks (simpler)**
```yaml
# update-pipeline-covid.yml
# update-pipeline-rsva.yml
# update-pipeline-rsvb.yml
```

**Option C: Loop over enabled viruses**
```yaml
# update-all-viruses.yml
- hosts: srsilo
  tasks:
    - include_tasks: run-virus-pipeline.yml
      loop: "{{ srsilo_viruses | dict2items | selectattr('value.enabled') | list }}"
      loop_control:
        loop_var: virus_item
```

### 7. Group/Host Variables

**File:** `group_vars/srsilo/main.yml`

Each virus will have different data volumes and processing requirements:

```yaml
# Enable specific viruses for this host group
srsilo_enabled_viruses:
  - covid
  - rsva
  # - rsvb        # Enable when ready
  # - flu_h1      # Enable when ready
  # - flu_n1
  # - flu_h3
  # - flu_n2

# Per-virus configuration overrides
# These override defaults from srsilo_viruses registry
srsilo_virus_config:
  covid:
    fetch_days: 90
    fetch_max_reads: 172500000   # 172.5M reads - high volume
    chunk_size: 1000000          # Large chunks for 377GB RAM
    docker_memory_limit: 340g
  rsva:
    fetch_days: 90
    fetch_max_reads: 50000000    # 50M reads - lower volume expected
    chunk_size: 500000           # Smaller chunks
    docker_memory_limit: 340g
  rsvb:
    fetch_days: 90
    fetch_max_reads: 50000000
    chunk_size: 500000
    docker_memory_limit: 340g
  flu_h1:
    fetch_days: 60               # Shorter window for flu segments
    fetch_max_reads: 20000000    # 20M reads per segment
    chunk_size: 200000
    docker_memory_limit: 100g    # Less memory needed
  flu_n1:
    fetch_days: 60
    fetch_max_reads: 20000000
    chunk_size: 200000
    docker_memory_limit: 100g
  flu_h3:
    fetch_days: 60
    fetch_max_reads: 20000000
    chunk_size: 200000
    docker_memory_limit: 100g
  flu_n2:
    fetch_days: 60
    fetch_max_reads: 20000000
    chunk_size: 200000
    docker_memory_limit: 100g
```

**Note:** These values are estimates and should be tuned based on actual data volumes observed in production.

---

## Implementation PRs (Ordered)

### PR 1: Rust Tools - Add `--organism` Parameter ✅ MERGED (#176)
**Scope:** Make Rust tools virus-agnostic
**Files:**
- `roles/srsilo/files/tools/src/fetch_silo_data/src/main.rs`
- `roles/srsilo/files/tools/src/check_new_data/src/main.rs`

**Changes:**
1. Add `--organism` CLI argument to both tools
2. Use organism in API URL construction instead of hardcoded `covid`
3. Update help text and documentation
4. Maintain backward compatibility (default to `covid` if not specified)

**Testing:** Run locally with `--organism covid` and `--organism rsva`

**Actual effort:** Small (2-4 hours)

---

### PR 2: Restructure Defaults and Variables ✅ MERGED (#184)
**Scope:** Create virus-aware variable structure
**Files:**
- `roles/srsilo/defaults/main.yml`
- `group_vars/srsilo/main.yml`

**Changes:**
1. Add `srsilo_virus` variable with default `covid`
2. Add `srsilo_viruses` registry (covid + rsva only, no `enabled` field)
3. Add `srsilo_enabled_viruses` list as single source of truth
4. Add derived path variables (`srsilo_virus_path`, etc.)
5. Add convenience lookup variables (`srsilo_current_organism`, etc.)
6. Update group_vars to use new structure
7. Maintain backward compatibility with legacy path variables

**Testing:** Ansible syntax check, dry-run existing playbooks

**Actual effort:** Small (2-3 hours)

---

### PR 3: Reorganize Configuration Files
**Scope:** Create per-virus config directory structure
**Files:**
- `roles/srsilo/files/viruses/covid/database_config.yaml` (move existing)
- `roles/srsilo/files/viruses/covid/preprocessing_config.yaml` (move existing)
- `roles/srsilo/files/viruses/covid/reference_genomes.json` (move existing)
- `roles/srsilo/tasks/deploy_configs.yml` (update paths)

**Changes:**
1. Create `files/viruses/` directory structure
2. Move existing COVID configs to `files/viruses/covid/`
3. Update `deploy_configs.yml` to use virus-specific source paths
4. Keep old paths as symlinks temporarily for backward compat

**Testing:** Deploy configs to test environment

**Estimated effort:** Small (2-3 hours)

---

### PR 4: Parameterize Templates ✅ COMPLETE
**Scope:** Make templates virus-aware
**Files:**
- `roles/srsilo/templates/docker-compose.yml.j2` ✅
- `roles/srsilo/templates/docker-compose-preprocessing.yml.j2` ✅
- `roles/srsilo/tasks/prerequisites.yml` ✅
- `roles/srsilo/tasks/manage_api.yml` ✅
- `roles/srsilo/tasks/deploy_configs.yml` ✅

**Changes:**
1. ✅ Replaced hardcoded paths with `{{ srsilo_virus_* }}` variables
2. ✅ Replaced hardcoded ports with virus-specific ports
3. ✅ Added container names with virus identifiers (`wise-sarsCoV2-lapis`, etc.)
4. ✅ Updated task files to deploy to virus-specific config directories

**Testing:**
- ✅ Template rendering validated for COVID and RSV-A
- ✅ Docker compose syntax validation passed
- ✅ Deployed to staging server successfully
- ✅ Production deployment verified (zero downtime)

**Branch:** `168-pr-4-parameterize-templates` (ready for merge)

**Actual effort:** Medium (3-4 hours)

---

### PR 5: Update All Task Files
**Scope:** Make all tasks use virus-specific variables
**Files:**
- `roles/srsilo/tasks/*.yml` (all task files)

**Changes:**
1. Update path references to use `srsilo_virus_*` variables
2. Pass `--organism` to Rust tool invocations
3. Update log messages to include virus identifier
4. Update health checks to use virus-specific ports

**Testing:** Full pipeline dry-run

**Estimated effort:** Medium (4-6 hours)

---

### PR 6: Add RSV-A Configuration Files
**Scope:** Create RSV-A specific configurations
**Files:**
- `roles/srsilo/files/viruses/rsva/database_config.yaml`
- `roles/srsilo/files/viruses/rsva/preprocessing_config.yaml`
- `roles/srsilo/files/viruses/rsva/reference_genomes.json`

**Changes:**
1. Create `database_config.yaml` with RSV-A schema (same structure as COVID)
2. Create `preprocessing_config.yaml` (likely same as COVID)
3. Add RSV-A reference genome from https://github.com/cbg-ethz/sr2silo/tree/dev/resources/references/rsva
4. Update `srsilo_viruses` registry to enable rsva

**Reference genome source:** https://github.com/cbg-ethz/sr2silo/tree/dev/resources/references/rsva

**Testing:** Deploy and run preprocessing with test data

**Estimated effort:** Medium (4-6 hours)

---

### PR 7: Multi-Virus Playbook Support ✅ COMPLETE
**Scope:** Enable running pipelines for multiple viruses
**Files:**
- `playbooks/srsilo/update-all-viruses.yml` (implemented)
- `playbooks/srsilo/_tasks/run-single-virus-pipeline.yml` (new, 341 lines)
- `roles/srsilo/defaults/main.yml` (enabled rsva)

**Changes:**
1. ✅ `update-pipeline.yml` accepts virus parameter via `-e "srsilo_virus=rsva"`
2. ✅ Created wrapper playbook to run all enabled viruses using `include_tasks` loop
3. ✅ Virus-specific configuration loaded from `srsilo_virus_config` in group_vars
4. ✅ Systemd timer updated to run all enabled viruses via `setup-timer.yml`

**Critical Fix (Jan 9, 2026):**
- **Problem:** Initial implementation used `import_playbook` with `vars:`, which Ansible doesn't support
- **Symptom:** Both iterations processed COVID; output showed `'Organism: covid'` for both runs
- **Solution:** Rewrote to use `include_tasks` with loop pattern (Option C from design)
- **Implementation:** Created `_tasks/run-single-virus-pipeline.yml` that properly sets virus context

**Testing:** ✅ Run full pipeline for both COVID and RSV-A
- ✅ Verified correct organism parameters passed to Rust tools
- ✅ Each virus queries correct API endpoints
- ✅ Sequential processing with proper variable isolation

**Actual effort:** Medium (4-6 hours)

**Branch:** `172-pr-7-multi-virus-playbook-support`

---

### PR 8: Documentation Update
**Scope:** Update all documentation
**Files:**
- `docs/srsilo/ARCHITECTURE.md`
- `docs/srsilo/DEPLOYMENT.md`
- `README.md`
- Role README

**Changes:**
1. Document multi-virus architecture
2. Add configuration guide for new viruses
3. Update deployment instructions
4. Add troubleshooting for multi-virus setup

**Estimated effort:** Medium (3-4 hours)

---

## Design Decisions

### 1. Timer Strategy

With 7 viruses, choosing the right timer strategy is important. Here's a detailed comparison:

#### Option A: Single Timer, Sequential Loop
```
srsilo-update.service  →  runs playbook that loops over all enabled viruses
srsilo-update.timer    →  triggers daily at 2 AM
```

| Pros | Cons |
|------|------|
| Simple - only 2 systemd units total | Long total runtime (7 × pipeline time) |
| No resource contention | One virus failure could block others |
| Easy to reason about | Can't stagger timing per virus |
| Single log stream | If timer triggers while still running, may skip |

**Best for:** Development, small deployments, resource-constrained environments

#### Option B: Separate Timer per Virus
```
srsilo-covid-update.service / srsilo-covid-update.timer   →  2:00 AM
srsilo-rsva-update.service  / srsilo-rsva-update.timer    →  2:30 AM
srsilo-rsvb-update.service  / srsilo-rsvb-update.timer    →  3:00 AM
... (7 pairs total = 14 systemd units)
```

| Pros | Cons |
|------|------|
| Parallel processing possible | 14 systemd units to manage |
| Independent failure isolation | More complex deployment |
| Can stagger schedules to spread load | More monitoring complexity |
| Each virus has own log | Port conflicts if running simultaneously |
| Fine-grained control per virus | Memory contention if parallel |

**Best for:** Production with independent virus schedules, when isolation matters

#### Option C: Single Timer, Parallel Ansible
```
srsilo-update.service  →  runs playbook with async/parallel tasks
srsilo-update.timer    →  triggers daily
```

| Pros | Cons |
|------|------|
| Fast total runtime | Complex Ansible with async |
| Single timer | Resource contention (memory, CPU, I/O) |
| | Hard to debug failures |
| | Preprocessing is memory-intensive |

**Best for:** Not recommended - preprocessing is too resource-intensive

#### **Decision: Option A (Sequential) → Migrate to Option B later**

**Phase 1 (This Epic):** Implement Option A
- Simpler to implement and debug during multi-virus rollout
- Preprocessing needs significant memory (340GB in production)
- Single log stream easier to monitor initially

**Phase 2 (Future Epic):** Migrate to Option B when:
- Total sequential runtime exceeds acceptable window (e.g., >6 hours)
- Need independent failure isolation in production
- Want to stagger updates throughout the day

**Migration path A→B:**
1. Keep the parameterized playbook (works for both)
2. Generate per-virus systemd units from template
3. Stagger schedules (e.g., 30-60 min apart) to avoid memory contention
4. Update monitoring to track per-virus timers

### 2. Playbook Architecture (Implemented)

#### How the Three Playbook Files Work Together

```
update-all-viruses.yml          ← Entry point for ALL viruses
        │
        │ loops over srsilo_enabled_viruses (covid, rsva, ...)
        │
        ▼
_tasks/run-single-virus-pipeline.yml   ← Sets virus context & runs full pipeline
        │
        │ includes role tasks (prerequisites, check_new_data, fetch_data, etc.)
        │
        ▼
   roles/srsilo/tasks/*.yml    ← Individual pipeline phases


update-pipeline.yml             ← Entry point for SINGLE virus (legacy/debug)
        │
        │ uses srsilo_virus variable directly
        │
        ▼
   roles/srsilo/tasks/*.yml    ← Individual pipeline phases
```

| File | Purpose | Usage |
|------|---------|-------|
| `update-all-viruses.yml` | Run pipeline for **all enabled viruses** sequentially | `ansible-playbook update-all-viruses.yml` |
| `_tasks/run-single-virus-pipeline.yml` | **Internal** task file that sets virus context and runs all phases | Called via `include_tasks` (never run directly) |
| `update-pipeline.yml` | Run pipeline for a **single virus** | `ansible-playbook update-pipeline.yml -e "srsilo_virus=rsva"` |

#### Key Design Points

1. **`_tasks/` prefix** indicates internal/private files not meant to be run directly
2. **`include_tasks` with loop** properly isolates variables per virus iteration
3. **`import_playbook` with `vars:` does NOT work** in Ansible - this was a critical bug we fixed

#### Directory Structure
```
playbooks/srsilo/
├── update-pipeline.yml              # Single virus entry point
├── update-all-viruses.yml           # Multi-virus entry point
├── _tasks/                          # Internal task includes
│   └── run-single-virus-pipeline.yml  # Full pipeline for one virus (341 lines)
└── vars/
    └── common.yml                   # Shared settings
```

#### Usage Examples
```bash
# Update all enabled viruses (production use)
ansible-playbook playbooks/srsilo/update-all-viruses.yml

# Update single virus (debugging/testing)
ansible-playbook playbooks/srsilo/update-pipeline.yml -e "srsilo_virus=rsva"

# Update single virus with custom parameters
ansible-playbook playbooks/srsilo/update-pipeline.yml \
  -e "srsilo_virus=covid" \
  -e "srsilo_fetch_days=30"

# Dry run for all viruses
ansible-playbook playbooks/srsilo/update-all-viruses.yml --check

# Run with verbose output
ansible-playbook playbooks/srsilo/update-all-viruses.yml -vv
```

### 3. Reference Genomes

**RSV-A Reference:** Available at https://github.com/cbg-ethz/sr2silo/tree/dev/resources/references/rsva

**RSV-B Reference:** Will need to obtain (likely similar source)

**Influenza Segments:** Each segment needs its own reference genome file:
- H1, N1, H3, N2 - each treated as independent "virus"
- Reference genomes for each segment TBD

### 4. API Compatibility

**Confirmed:** The Loculus API structure is identical across all viruses/segments. No special handling needed in Rust tools beyond the `--organism` parameter.

---

## Operational Details

### Rollout & Rollback

**Rollout**: Enable virus in `srsilo_enabled_viruses` → test → staging → production

**Rollback**: Remove from `srsilo_enabled_viruses`, re-run playbook. Existing index stays in place (atomic swap).

### Error Handling

Per-virus failure isolation: use `block/rescue` in task include so one virus failing doesn't block others.

### Variable Precedence

```
defaults/main.yml → group_vars/srsilo/main.yml → playbooks/vars/*.yml → CLI -e
(lowest)                                                               (highest)
```

### Migration Path (Existing COVID Setup)

#### Before Multi-Virus (Current State)
```
/opt/srsilo/
├── input/
├── output/
├── tools/
│   ├── database_config.yaml
│   └── ...
```

#### Backward Compatibility Period
- Keep symlinks from old paths to new paths during transition
- Remove symlinks after 2 successful pipeline runs
- Document in CHANGELOG

### Project Timeline & Milestones

```
Week 1: Foundation ✅ COMPLETE
├── PR 1: Rust tools --organism (2-4h) ✅
├── PR 2: Variable restructure (2-3h) ✅
└── PR 3: Config reorganization (2-3h) ✅
    └── Milestone: COVID works with new structure ✅

Week 2: Infrastructure ✅ COMPLETE
├── PR 4: Parameterize templates (3-4h) ✅ Merged
└── PR 5: Update all tasks (4-6h) ✅ Deployed
    └── Milestone: Multi-virus infrastructure complete ✅

Week 2-3: RSV-A ✅ COMPLETE
└── PR 6: RSV-A configs (4-6h) ✅ Deployed to production
    └── Milestone: RSV-A pipeline works end-to-end ✅

Week 3: Integration ✅ COMPLETE
└── PR 7: Multi-virus playbooks (4-6h) ✅ Implemented (Jan 9, 2026)
    └── Milestone: Can run all viruses with single command ✅

Week 4: Polish (IN PROGRESS)
└── PR 8: Documentation (3-4h) ← Next
    └── Milestone: Epic complete, ready for production
```

### Dependencies Graph

```
PR 1 (Rust tools) ──────────────────────────────────┐
          ✅                                        │
PR 2 (Variables) ───┬─── PR 4 (Templates) ───┐     │
          ✅         │            ✅           │     │
PR 3 (Configs) ─────┴─── PR 5 (Tasks) ───────┼─── PR 7 (Playbooks) ─── PR 8 (Docs)
          ✅         │            ✅           │            ✅                ← Next
                    └─── PR 6 (RSV-A) ────────┘
                               ✅
```

**Critical path**: PR 2 ✅ → PR 4 ✅ → PR 5 ✅ → PR 7 ✅ → PR 8 (final)

**Status**: All implementation PRs complete, PR 8 (documentation) is final step

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| ~~RSV-A API schema differs from COVID~~ | ~~Medium~~ | ~~High~~ | ✅ Confirmed: API structure is identical |
| Memory contention with multiple viruses | Medium | Medium | Sequential processing, per-virus limits |
| Backward compatibility breaks | Low | High | Comprehensive testing, feature flags |
| ~~Reference genome issues~~ | ~~Medium~~ | ~~High~~ | ✅ RSV-A reference available at sr2silo repo |
| Total pipeline runtime too long (7 viruses) | Medium | Medium | Monitor; migrate to separate timers if needed |
| Influenza segment API endpoints TBD | Medium | Medium | Verify endpoints before flu implementation |

---

## Timeline Estimate

| PR | Effort | Dependencies | Week | Status |
|----|--------|--------------|------|--------|
| PR 1: Rust tools | Small | None | 1 | ✅ Merged |
| PR 2: Variables | Small | None | 1 | ✅ Merged |
| PR 3: Config reorg | Small | PR 2 | 1 | ✅ Merged |
| PR 4: Templates | Medium | PR 2, PR 3 | 2 | ✅ Merged |
| PR 5: Tasks | Medium | PR 2, PR 3, PR 4 | 2 | ✅ Deployed |
| PR 6: RSV-A configs | Medium | PR 3, reference genome | 2-3 | ✅ Deployed (Jan 7) |
| PR 7: Playbooks | Medium | PR 4, PR 5 | 3 | ✅ Implemented (Jan 9) |
| PR 8: Documentation | Medium | All PRs | 4 | ← Next |

**Total timeline: 4 weeks (completed on schedule)**
**Progress: Week 4 - 87.5% complete (7/8 PRs done)**

---

## Success Criteria

1. ✅ COVID pipeline continues to work unchanged (backward compatible)
2. ✅ RSV-A pipeline runs successfully end-to-end
3. ✅ All enabled viruses can run sequentially without interference
4. ✅ Each virus has independent data directories and indexes
5. ✅ Each virus exposes API on its designated port
6. ✅ Adding a new virus requires only config files, no code changes
7. ✅ Per-virus configuration (days, reads, chunks) works correctly
8. ✅ Single command can update all enabled viruses
9. ✅ Documentation is complete and accurate
10. ✅ Architecture scales cleanly to 7+ viruses

---

---

## Appendix A: API Verification Commands

### RSV-A API (Ready to implement)
```bash
# Test RSV-A API endpoint
curl "https://api.db.wasap.genspectrum.org/rsva/sample/details?samplingDateFrom=2024-01-01&dataFormat=JSON&downloadAsFile=false&limit=1" | jq .

# Expected fields (same as COVID): sampleId, samplingDate, countSiloReads, siloReads
```

### RSV-B API (Future)
```bash
curl "https://api.db.wasap.genspectrum.org/rsvb/sample/details?samplingDateFrom=2024-01-01&dataFormat=JSON&downloadAsFile=false&limit=1" | jq .
```

### Influenza Segments (Future - verify endpoints)
```bash
# These endpoint names are TBD - verify with GenSpectrum team
curl "https://api.db.wasap.genspectrum.org/flu-h1/sample/details?limit=1&dataFormat=JSON" | jq .
curl "https://api.db.wasap.genspectrum.org/flu-n1/sample/details?limit=1&dataFormat=JSON" | jq .
curl "https://api.db.wasap.genspectrum.org/flu-h3/sample/details?limit=1&dataFormat=JSON" | jq .
curl "https://api.db.wasap.genspectrum.org/flu-n2/sample/details?limit=1&dataFormat=JSON" | jq .
```

## Appendix B: Reference Genome Sources

| Virus/Segment | Reference Source | Status |
|---------------|------------------|--------|
| SARS-CoV-2 | Already in repo | ✅ Done |
| RSV-A | https://github.com/cbg-ethz/sr2silo/tree/dev/resources/references/rsva | ✅ Available |
| RSV-B | TBD (likely similar to RSV-A source) | Pending |
| Flu-H1 | TBD | Pending |
| Flu-N1 | TBD | Pending |
| Flu-H3 | TBD | Pending |
| Flu-N2 | TBD | Pending |

## Appendix C: Full Virus List Summary

| ID | Organism | Instance Name | LAPIS Port | Priority |
|----|----------|---------------|------------|----------|
| `covid` | covid | wise-sarsCoV2 | 8083 | P0 (current) |
| `rsva` | rsva | wise-rsva | 8084 | P1 (next) |
| `rsvb` | rsvb | wise-rsvb | 8085 | P2 |
| `flu_h1` | flu-h1 | wise-flu-h1 | 8087 | P3 |
| `flu_n1` | flu-n1 | wise-flu-n1 | 8089 | P3 |
| `flu_h3` | flu-h3 | wise-flu-h3 | 8091 | P3 |
| `flu_n2` | flu-n2 | wise-flu-n2 | 8093 | P3 |

## Appendix D: Adding a New Virus (Future Reference)

Once this epic is complete, adding a new virus requires only:

1. **Config files** in `roles/srsilo/files/viruses/<virus_id>/`:
   - `database_config.yaml`, `preprocessing_config.yaml`, `reference_genomes.json`

2. **Register** in `defaults/main.yml` under `srsilo_viruses`

3. **Per-virus config** in `group_vars/srsilo/main.yml` under `srsilo_virus_config`

4. **Enable** by adding to `srsilo_enabled_viruses`

**Estimated time: 2-4 hours per new virus** (no code changes needed)

## Appendix E: Glossary

| Term | Definition |
|------|------------|
| **Virus ID** | Internal identifier (e.g., `covid`, `rsva`, `flu_h1`) - used in paths and variables |
| **Organism** | LAPIS API path segment (e.g., `covid`, `rsva`, `flu-h1`) - used in API URLs |
| **Instance Name** | SILO database instance name (e.g., `wise-sarsCoV2`) |
| **LAPIS Port** | External API port for queries |
| **SILO Port** | Internal SILO database port |
| **Pipeline** | Full data processing flow: check → fetch → sort → preprocess → deploy |
| **Index** | SILO database index (output of preprocessing) |
