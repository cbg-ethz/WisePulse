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

- [ ] **PR 1: Rust Tools - Add `--organism` Parameter** (Small, 2-4h)
  - Add `--organism` CLI argument to `fetch_silo_data` and `check_new_data`
  - Use organism in API URL instead of hardcoded `covid`
  - Dependencies: None

- [ ] **PR 2: Restructure Defaults and Variables** (Small, 2-3h)
  - Add `srsilo_virus` variable and `srsilo_viruses` registry
  - Add derived path variables (`srsilo_virus_path`, etc.)
  - Dependencies: None

- [ ] **PR 3: Reorganize Configuration Files** (Small, 2-3h)
  - Create `files/viruses/` directory structure
  - Move existing COVID configs to `files/viruses/covid/`
  - Dependencies: PR 2

- [ ] **PR 4: Parameterize Templates** (Medium, 3-4h)
  - Replace hardcoded paths/ports in docker-compose and systemd templates
  - Add virus identifier to service/timer names
  - Dependencies: PR 2, PR 3

- [ ] **PR 5: Update All Task Files** (Medium, 4-6h)
  - Update path references to use `srsilo_virus_*` variables
  - Pass `--organism` to Rust tool invocations
  - Dependencies: PR 2, PR 3, PR 4

- [ ] **PR 6: Add RSV-A Configuration Files** (Medium, 4-6h)
  - Create RSV-A database/preprocessing configs
  - Add RSV-A reference genome from [sr2silo repo](https://github.com/cbg-ethz/sr2silo/tree/dev/resources/references/rsva)
  - Dependencies: PR 3

- [ ] **PR 7: Multi-Virus Playbook Support** (Medium, 4-6h)
  - Create wrapper playbook to run all enabled viruses
  - Parameterize `update-pipeline.yml` to accept virus parameter
  - Dependencies: PR 4, PR 5

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

## Success Criteria

- [ ] COVID pipeline continues to work (backward compatible)
- [ ] RSV-A pipeline runs end-to-end
- [ ] All enabled viruses run sequentially without interference
- [ ] Adding a new virus requires only config files (no code changes)
- [ ] Single command updates all enabled viruses

## Timeline

**Estimated: 3-4 weeks**

| Week | Milestones |
|------|------------|
| 1 | Foundation (PR 1-3): COVID works with new structure |
| 2 | Infrastructure (PR 4-5): Multi-virus infrastructure complete |
| 2-3 | RSV-A (PR 6): RSV-A pipeline works end-to-end |
| 3-4 | Integration (PR 7-8): Epic complete |

## References

- [Planning Document](MULTI_VIRUS_PLAN.md)
- [LAPIS API](https://api.db.wasap.genspectrum.org/)
- [RSV-A Reference Genome](https://github.com/cbg-ethz/sr2silo/tree/dev/resources/references/rsva)
