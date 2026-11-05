"""srsilo-updater pipeline entry point.

Usage:
    python -m pipeline --config /opt/srsilo/pipeline.yml [--virus covid]

All viruses are served by one RhyDB instance, one table each. Every virus with
new data is fetched and sorted on its own; if at least one was, a fresh state
is built from all viruses' sorted data at the end.
"""

import argparse
import logging
import shutil
import sys
from pathlib import Path
from typing import List

from pipeline.config import PipelineConfig
from pipeline.phases import build_state, check_new_data, cleanup, fetch, sort_and_merge


def setup_logging() -> None:
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s: %(message)s",
        datefmt="%Y-%m-%dT%H:%M:%S",
        stream=sys.stdout,
    )


def run_virus(virus_name: str, config: PipelineConfig, staged: List[str]) -> bool:
    """Run the pipeline for one virus, adding it to `staged` if it has new data. Returns True on success."""
    log = logging.getLogger(f"pipeline.{virus_name}")
    virus = config.viruses[virus_name]
    paths = config.virus_paths(virus_name)

    log.info("======== %s ========", virus_name.upper())

    has_new_data = check_new_data.run(config, virus, paths)
    if not has_new_data:
        log.info("Nothing to do for %s", virus_name)
        return True

    try:
        cleanup.run(config, virus, paths)
        fetch.run(config, virus, paths)
        sort_and_merge.run(config, virus, paths)
    except Exception as exc:
        log.error("Pipeline failed for %s: %s", virus_name, exc)
        paths.next_timestamp.unlink(missing_ok=True)
        return False

    staged.append(virus_name)
    return True


def build(config: PipelineConfig, staged: List[str]) -> bool:
    """Build a fresh state from all sorted data, then promote the staged viruses' timestamps."""
    log = logging.getLogger("pipeline.rhydb")
    log.info("======== RHYDB ========")
    try:
        build_state.run(config)
    except Exception as exc:
        log.error("Building the RhyDB state failed: %s", exc)
        for virus_name in staged:
            config.virus_paths(virus_name).next_timestamp.unlink(missing_ok=True)
        return False

    for virus_name in staged:
        paths = config.virus_paths(virus_name)
        if paths.next_timestamp.exists():
            shutil.copy2(paths.next_timestamp, paths.last_update)
            paths.next_timestamp.unlink()
    return True


def main() -> None:
    setup_logging()
    log = logging.getLogger("pipeline")

    parser = argparse.ArgumentParser(description="srSILO update pipeline")
    parser.add_argument("--config", type=Path, required=True, help="Path to pipeline.yml")
    parser.add_argument("--virus", help="Process a single virus (default: all enabled viruses)")
    args = parser.parse_args()

    config = PipelineConfig.load(args.config)

    viruses = [args.virus] if args.virus else config.enabled_viruses
    log.info("Processing %d virus(es): %s", len(viruses), ", ".join(viruses))

    failures, staged = [], []
    for virus_name in viruses:
        if virus_name not in config.viruses:
            log.error("Unknown virus: %s", virus_name)
            failures.append(virus_name)
            continue
        ok = run_virus(virus_name, config, staged)
        if not ok:
            failures.append(virus_name)

    if staged and not build(config, staged):
        failures.append("rhydb")
    if failures:
        log.error("Pipeline finished with failures: %s", ", ".join(failures))
        sys.exit(1)

    log.info("Pipeline complete")


if __name__ == "__main__":
    main()
