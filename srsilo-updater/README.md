# srSILO Updater

Rust-based data pipeline tools for the srSILO multi-virus genomic database.

## Overview

This workspace contains 5 binaries that power the srSILO data pipeline:

- `check_new_data` - Queries LAPIS API to detect new sequence data
- `fetch_silo_data` - Downloads NDJSON data from LAPIS API
- `split_into_sorted_chunks` - Splits large datasets for parallel processing
- `merge_sorted_chunks` - Merges sorted chunks into final dataset
- `add_offset` - Adjusts timestamps for incremental updates

## Building

```bash
# Development build
cargo build

# Release build (used in deployment)
cargo build --release
```

## Testing

```bash
cargo test --workspace
```

## Code Quality

```bash
# Format check
cargo fmt --all -- --check

# Linting
cargo clippy --workspace --all-targets -- -D warnings
```

## Deployment

These tools are deployed to `/opt/srsilo/tools/` on srSILO hosts via Ansible. See the main WisePulse documentation for deployment instructions.

## CI/CD

Quality checks run automatically on all PRs via GitHub Actions (`.github/workflows/srsilo-ci.yml`).
