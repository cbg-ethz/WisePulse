# Epic: Multi-Virus Support for srSILO Role

## Summary

Generalize the `srsilo` Ansible role to support multiple viruses beyond SARS-CoV-2. Each virus will have its own configuration, data directories, Docker containers, and API endpoints.

## Target Viruses

| ID | Organism | Port | Priority |
|----|----------|------|----------|
| `covid` | covid | 8083 | P0 (current) |
| `rsva` | rsva | 8084 | P1 (next) |
| `rsvb` | rsvb | 8085 | P2 |
| `flu_h1` | flu-h1 | 8087 | P3 |
| `flu_n1` | flu-n1 | 8089 | P3 |
| `flu_h3` | flu-h3 | 8091 | P3 |
| `flu_n2` | flu-n2 | 8093 | P3 |

## Sub-Issues / PRs

- [x] **PR 1: Rust Tools - Add `--organism` Parameter** (Small, 2-4h) ✅ Merged
  - Add `--organism` CLI argument to `fetch_silo_data` and `check_new_data`
  - Use organism in API URL instead of hardcoded `covid`
  - Dependencies: None
  - **Merged:** #176

- [x] **PR 2: Restructure Defaults and Variables** (Small, 2-3h) ✅ Merged
  - Add `srsilo_virus` variable and `srsilo_viruses` registry (covid + rsva)
  - Add `srsilo_enabled_viruses` list as single source of truth
  - Add derived path variables (`srsilo_virus_path`, etc.)
  - Add convenience lookup variables (`srsilo_current_organism`, etc.)
  - Dependencies: None
  - **Merged:** #184

- [x] **PR 3: Reorganize Configuration Files** (Small, 2-3h) ✅ Complete
  - Create `files/viruses/` directory structure
  - Move existing COVID configs to `files/viruses/covid/`
  - Update `deploy_configs.yml` to use virus-parameterized paths
  - Add backward compatibility symlinks
  - Dependencies: PR 2
  - **Branch:** `pr-3-reorganize-configuration-files` (ready for PR)

- [x] **PR 4: Parameterize Templates** (Medium, 3-4h) ✅ Complete
  - Parameterized docker-compose templates with virus-specific ports, paths, container names
  - Updated task files to deploy to virus-specific config directories
  - Systemd changes deferred to PR 7 (playbook orchestration)
  - Dependencies: PR 2, PR 3
  - **Branch:** `168-pr-4-parameterize-templates` (ready for merge)
  - **Testing:** ✅ Template rendering validated for COVID and RSV-A
  - **Testing:** ✅ Deployed to staging server successfully
  - **Testing:** ✅ Production deployment verified (zero downtime)
  - **Note**: Memory limits (`srsilo_docker_memory_limit`) currently global in group_vars
    - Future enhancement: Add per-virus memory limits to `srsilo_viruses` registry
    - COVID ports unchanged: 8083 (LAPIS), 8081 (SILO)

- [ ] **PR 5: Update All Task Files** (Medium, 4-6h)
  - Update path references to use `srsilo_virus_*` variables
  - Pass `--organism` to Rust tool invocations
  - Dependencies: PR 2, PR 3, PR 4

- [x] **PR 6: Add RSV-A Configuration Files** (Medium, 4-6h) ✅ Deployed
  - Create RSV-A database/preprocessing configs
  - Add RSV-A reference genome from [sr2silo repo](https://github.com/cbg-ethz/sr2silo/tree/dev/resources/references/rsva)
  - Dependencies: PR 3
  - **Branch:** Multiple branches (config files + bugfixes)
  - **Testing:** ✅ Full pipeline tested on production
  - **Testing:** ✅ 171K sequences indexed successfully
  - **Testing:** ✅ SILO API responding (port 8082)
  - **Testing:** ✅ LAPIS API responding with RSV-A data (port 8084)
  - **Production deployment:** ✅ Complete (Jan 7, 2026)
  - **Bugfixes:** Critical Docker Compose fixes deployed (see below)

- [x] **PR 7: Multi-Virus Playbook Support** (Medium, 4-6h) ✅ Complete
  - Create wrapper playbook to run all enabled viruses
  - Parameterize `update-pipeline.yml` to accept virus parameter
  - Dependencies: PR 4, PR 5
  - **Branch:** `172-pr-7-multi-virus-playbook-support`
  - **Status:** ✅ Implemented and tested (Jan 9, 2026)
  - **Key Fix:** Replaced broken `import_playbook` with `include_tasks` loop pattern
  - **Testing:** ✅ Verified both COVID and RSV-A process correctly with proper organism parameters

- [ ] **PR 8: Documentation Update** (Medium, 3-4h)
  - Update ARCHITECTURE.md and deployment docs
  - Add guide for adding new viruses
  - Dependencies: All PRs

## Dependency Graph

```
PR 1 (Rust) ────────────────────────────────────────┐
                                                    │
PR 2 (Variables) ───┬─── PR 4 (Templates) ───┐     │
                    │                         │     │
PR 3 (Configs) ─────┴─── PR 5 (Tasks) ───────┼─── PR 7 (Playbooks) ─── PR 8 (Docs)
                    │                         │
                    └─── PR 6 (RSV-A) ────────┘
```

**Critical path**: PR 2 → PR 4 → PR 5 → PR 7

## Key Design Decisions

1. **Timer Strategy**: Sequential processing (Option A) initially; migrate to per-virus timers (Option B) if runtime exceeds acceptable window
2. **Playbook Strategy**: Wrapper + parameterized core (Option C) - single core playbook with loop wrapper for all viruses
3. **Directory Structure**: `/opt/srsilo/{virus}/` with per-virus config, input, output directories
4. **Shared Binaries**: Rust tools compiled once in `/opt/srsilo/tools/`, used by all viruses
5. **Single Source of Truth**: `srsilo_enabled_viruses` list controls which viruses are active (no redundant `enabled` field in registry)
6. **Explicit Organism Field**: Registry uses explicit `organism` field for API endpoint naming (handles hyphens vs underscores)
7. **Incremental Virus Support**: Start with covid + rsva only; add more viruses to registry as needed

## Success Criteria

- [x] COVID pipeline continues to work (backward compatible) ✅
- [x] RSV-A pipeline runs end-to-end ✅
- [x] All enabled viruses run sequentially without interference ✅
- [x] Adding a new virus requires only config files (no code changes) ✅
- [x] Single command updates all enabled viruses ✅

## Timeline

**Estimated: 3-4 weeks**

| Week | Milestones |
|------|------------|
| 1 | Foundation (PR 1-3): COVID works with new structure ✅ |
| 2 | Infrastructure (PR 4-5): Multi-virus infrastructure complete (PR 4 ✅) |
| 2-3 | RSV-A (PR 6): RSV-A pipeline works end-to-end |
| 3-4 | Integration (PR 7-8): Epic complete |

## Deployment Notes

### Current Environment: Staging Server

PR 3 has been implemented and tested on the **staging server**. All changes are backward compatible with the existing COVID pipeline.

### Production Deployment Strategy

When deploying PRs to production, follow this process:

1. **Pre-Deployment Verification** (on staging):
   - Verify COVID pipeline still works with new structure
   - Run syntax checks: `ansible-playbook playbooks/srsilo/setup.yml --syntax-check`
   - Perform dry run: `ansible-playbook playbooks/srsilo/setup.yml --check`

2. **Production Deployment** (after PR merge):
   ```bash
   # On production server (or from control node targeting production)
   cd /path/to/WisePulse
   git fetch origin
   git checkout main  # or target branch
   git pull origin main

   # Run Ansible playbook to deploy changes
   ansible-playbook -i inventory/production playbooks/srsilo/setup.yml
   ```

3. **Post-Deployment Verification**:
   - Verify COVID pipeline continues to function
   - Check configuration files deployed correctly
   - Monitor first pipeline run after deployment

4. **Rollback Plan** (if issues arise):
   - Revert to previous git commit
   - Re-run Ansible playbook
   - Existing COVID indexes remain intact (atomic swap)

### PR 3 Specific Deployment Notes

- **No service downtime required**: Changes are file organization only
- **Backward compatible**: Symlinks maintain old paths
- **No data migration needed**: File moves handled by git/Ansible
- **Safe to deploy**: Syntax validated, dry-run tested on staging

### PR 6 Production Deployment (Jan 7, 2026) ✅

**RSV-A deployment completed successfully:**
- ✅ 171,592 sequences indexed
- ✅ SILO API healthy on port 8082
- ✅ LAPIS API healthy on port 8084
- ✅ Data version: 1767793267 (Jan 7, 2026)
- ✅ Multi-virus infrastructure validated (COVID + RSV-A running simultaneously)

**Critical Bugfixes Deployed (Branch: `fix-multi-virus-docker-compose-bugs`):**

**Bug 1: SILO Internal Port Mapping**
- **Issue:** Template used `{{ srsilo_current_silo_port }}` for both external and internal ports
- **Impact:** RSV-A LAPIS tried connecting to `wise-rsva-silo:8082`, but SILO listens on `8081` internally
- **Fix:** Changed docker-compose.yml.j2 line 11 to hardcoded `:8081` and line 29 to `{{ srsilo_current_silo_port }}:8081`
- **Why COVID worked:** External port 8081 matched internal port 8081 by accident
- **Why RSV-A failed:** External port 8082 ≠ internal port 8081

**Bug 2: Docker Compose Project Isolation**
- **Issue:** `docker compose` commands lacked `-p {{ srsilo_virus }}` flag
- **Impact:** Both viruses shared `config_default` network; starting one recreated the other's containers
- **Fix:** Added `-p {{ srsilo_virus }}` to all docker compose commands in manage_api.yml (lines 17, 26, 52)
- **Result:** Each virus now has isolated project (`covid_default`, `rsva_default` networks)

**Deployment Challenges:**
1. Initial deployment created orphaned containers without project names
2. Subsequent deployments failed with "container name already in use" errors
3. Required manual cleanup: `docker rm -f $(docker ps -a --filter "name=wise-rsva" -q)`
4. After cleanup, both viruses started successfully with proper isolation

**Current Status:**
```
COVID:  169M sequences | Ports 8081 (SILO), 8083 (LAPIS) | Healthy ✅
RSV-A:  171K sequences | Ports 8082 (SILO), 8084 (LAPIS) | Healthy ✅
```

### PR 7 Implementation Details (Jan 9, 2026) ✅

**Critical Bug Fixed:**
The initial implementation of `update-all-viruses.yml` used `import_playbook` with `vars:`, which is not supported by Ansible. This caused both iterations to process COVID instead of COVID then RSV-A.

**Evidence of Bug:**
```yaml
# Original (broken):
- import_playbook: update-pipeline.yml
  vars:
    srsilo_virus: rsva  # This doesn't actually set the variable!
  when: "'rsva' in hostvars[groups['srsilo'][0]].srsilo_enabled_viruses"
```

Output showed: `'Organism: covid'` in both pipeline runs, confirming the variable wasn't being passed.

**Solution Implemented:**
Rewrote to use `include_tasks` with a loop pattern (Option C from MULTI_VIRUS_PLAN.md):

```yaml
# New (working):
tasks:
  - name: Process each enabled virus sequentially
    include_tasks: _tasks/run-single-virus-pipeline.yml
    loop: "{{ srsilo_enabled_viruses }}"
    loop_control:
      loop_var: current_virus
```

**Files Changed:**
1. `playbooks/srsilo/update-all-viruses.yml` - Main entry point using include_tasks loop
2. `playbooks/srsilo/_tasks/run-single-virus-pipeline.yml` - Complete pipeline for single virus (341 lines)
3. `roles/srsilo/defaults/main.yml` - Enabled rsva in srsilo_enabled_viruses

**Testing Results:**
- ✅ Both COVID and RSV-A validated in enabled viruses list
- ✅ Correct sequential iteration over each virus
- ✅ Each virus receives proper organism-specific variables
- ✅ Rust tools get correct `--organism` parameters (covid/rsva)
- ✅ Each virus queries correct API endpoints

**Usage:**
```bash
# Update all enabled viruses
ansible-playbook playbooks/srsilo/update-all-viruses.yml -i inventory.ini

# Update single virus (still works)
ansible-playbook playbooks/srsilo/update-pipeline.yml -i inventory.ini -e "srsilo_virus=rsva"
```

### Future PR Deployment Considerations

- **PR 4**: ✅ Zero downtime deployment - templates deploy to new paths, old containers keep running
- **PR 5**: ✅ Deployed successfully - full pipeline migration to virus-specific paths complete
- **PR 6**: ✅ RSV-A deployed - bugfixes ensure proper multi-virus isolation
- **PR 7**: ✅ Multi-virus wrapper playbook - working correctly with proper variable passing

## Future Enhancements (Post-Epic)

### Per-Virus Resource Configuration

**Current State (PR 4):**
- Memory limits: Global `srsilo_docker_memory_limit: 340g` in group_vars
- Chunk size: Global `srsilo_chunk_size: 1000000` in group_vars
- All viruses share same resource settings

**Future Enhancement:**
Add per-virus resource settings to `srsilo_viruses` registry in `defaults/main.yml`:

```yaml
srsilo_viruses:
  covid:
    organism: covid
    instance_name: wise-sarsCoV2
    lapis_port: 8083
    silo_port: 8081
    docker_memory_limit: 340g      # High volume virus
    chunk_size: 1000000
  rsva:
    organism: rsva
    instance_name: wise-rsva
    lapis_port: 8084
    silo_port: 8082
    docker_memory_limit: 340g      # Similar volume
    chunk_size: 500000
  flu_h1:
    organism: flu-h1
    instance_name: wise-flu-h1
    lapis_port: 8085
    silo_port: 8086
    docker_memory_limit: 100g      # Lower volume per segment
    chunk_size: 200000
```

Then update lookup variables:
```yaml
srsilo_docker_memory_limit: "{{ srsilo_current_virus.docker_memory_limit | default('340g') }}"
srsilo_chunk_size: "{{ srsilo_current_virus.chunk_size | default(1000000) }}"
```

**Benefits:**
- Optimize resource usage per virus
- Support viruses with different data volumes
- Influenza segments can use less memory

**When to implement:** After PR 6 when RSV-A is added, or when adding influenza segments

## References

- [Planning Document](MULTI_VIRUS_PLAN.md)
- [LAPIS API](https://api.db.wasap.genspectrum.org/)
- [RSV-A Reference Genome](https://github.com/cbg-ethz/sr2silo/tree/dev/resources/references/rsva)
