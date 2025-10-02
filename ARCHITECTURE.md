# WisePulse Pipeline Architecture

## Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                        Systemd Timer                             │
│  (wisepulse-pipeline.timer)                                     │
│  Triggers: Daily at 02:00 + random 0-30min                      │
└────────────────────┬────────────────────────────────────────────┘
                     │ activates
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Systemd Service                              │
│  (wisepulse-pipeline.service)                                   │
│  User: wisepulse                                                 │
│  WorkDir: /opt/wisepulse                                        │
└────────────────────┬────────────────────────────────────────────┘
                     │ executes
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                  make smart-fetch-and-process                    │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│              check_new_data (Rust binary)                        │
│  • Queries LAPIS API for latest sequence dates                  │
│  • Compares with .last_update timestamp                         │
│  • Exit 0 = new data, Exit 1 = no new data                     │
└────────────────────┬────────────────────────────────────────────┘
                     │
         ┌───────────┴───────────┐
         │                       │
    Exit 0                  Exit 1
    (new data)           (no new data)
         │                       │
         ▼                       ▼
┌─────────────────┐    ┌──────────────────┐
│ make clean-data │    │  Skip & Log      │
└────────┬────────┘    │  "No new data"   │
         │             └──────────────────┘
         ▼
┌─────────────────┐
│ make fetch-data │
│ (fetch_silo_data)│
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   make all      │
│ • split chunks  │
│ • merge chunks  │
│ • SILO preproc  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│update_timestamp │
│(.last_update)   │
└─────────────────┘
         │
         ▼
    Logs to journald
```

## Component Diagram

```
┌────────────────────────────────────────────────────────────────┐
│                     LAPIS API                                   │
│              (api.db.wasap.genspectrum.org)                    │
└────────────────────┬───────────────────────────────────────────┘
                     │ queries
                     │
     ┌───────────────┴────────────────────┐
     │                                    │
     │ check                         │ fetch
     ▼                                    ▼
┌──────────────┐              ┌────────────────────┐
│check_new_data│              │ fetch_silo_data    │
└──────────────┘              └────────────────────┘
     │                                    │
     │ writes                        │ writes
     ▼                                    ▼
┌──────────────┐              ┌────────────────────┐
│ .last_update │              │   silo_input/      │
│  (timestamp) │              │   *.ndjson.zst     │
└──────────────┘              └──────────┬─────────┘
                                         │
                                         ▼
                              ┌────────────────────┐
                              │split_into_sorted   │
                              │     _chunks        │
                              └──────────┬─────────┘
                                         │
                                         ▼
                              ┌────────────────────┐
                              │  sorted_chunks/    │
                              └──────────┬─────────┘
                                         │
                                         ▼
                              ┌────────────────────┐
                              │ merge_sorted       │
                              │     _chunks        │
                              └──────────┬─────────┘
                                         │
                                         ▼
                              ┌────────────────────┐
                              │sorted.ndjson.zst   │
                              └──────────┬─────────┘
                                         │
                                         ▼
                              ┌────────────────────┐
                              │ SILO preprocessing │
                              │  (docker-compose)  │
                              └──────────┬─────────┘
                                         │
                                         ▼
                              ┌────────────────────┐
                              │   silo_output/     │
                              │  (SILO indexes)    │
                              └────────────────────┘
                                         │
                                         ▼
                              ┌────────────────────┐
                              │    LAPIS API       │
                              │  (docker-compose)  │
                              │   Port: 8083       │
                              └────────────────────┘
```

## File Structure

```
/opt/wisepulse/                      # Repository location
├── .last_update                     # Timestamp file (managed by pipeline)
├── Makefile                         # Build and pipeline orchestration
├── Cargo.toml                       # Rust workspace
├── check_new_data/                  # New: Smart data checking
│   └── src/main.rs
├── update_timestamp/                # New: Timestamp updater
│   └── src/main.rs
├── fetch_silo_data/                 # Fetch data from LAPIS
│   └── src/main.rs
├── split_into_sorted_chunks/        # Split and sort data
│   └── src/main.rs
├── merge_sorted_chunks/             # Merge sorted chunks
│   └── src/main.rs
├── silo_input/                      # Downloaded .ndjson.zst files
├── sorted_chunks/                   # Intermediate sorted chunks
├── silo_output/                     # Final SILO indexes
├── ansible/
│   ├── playbooks/
│   │   ├── deploy.yml               # Kubernetes deployment
│   │   └── setup-pipeline.yml       # New: Pipeline automation
│   └── roles/
│       └── wisepulse_pipeline/      # New: Automation role
│           ├── defaults/main.yml
│           ├── tasks/main.yml
│           ├── templates/
│           │   ├── wisepulse-pipeline.service.j2
│           │   └── wisepulse-pipeline.timer.j2
│           └── handlers/main.yml

/etc/systemd/system/                 # System configuration
├── wisepulse-pipeline.service       # Pipeline service definition
└── wisepulse-pipeline.timer         # Timer schedule

/var/log/wisepulse/                  # Log directory (created by role)
```

## Ansible Deployment Flow

```
ansible-playbook setup-pipeline.yml
         │
         ▼
┌────────────────────┐
│ Check prereqs      │
│ - Rust/Cargo       │
│ - Docker           │
│ - Repository       │
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│ Create user/group  │
│ - wisepulse:       │
│   wisepulse        │
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│ Build binaries     │
│ - cargo build      │
│   --release        │
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│ Deploy systemd     │
│ - .service         │
│ - .timer           │
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│ Enable & start     │
│ - systemctl enable │
│ - systemctl start  │
└────────┬───────────┘
         │
         ▼
    ✅ Complete
```
