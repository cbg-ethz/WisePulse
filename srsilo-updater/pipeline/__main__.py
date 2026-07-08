"""srsilo-updater pipeline entry point.

Usage:
    python -m pipeline --config /opt/srsilo/pipeline.yml [--virus covid]
"""

import argparse
import logging
import sys
from pathlib import Path

from pipeline.config import PipelineConfig
from pipeline.phases import api, check_new_data, cleanup, fetch, finalize, preprocessing, sort_and_merge


def setup_logging() -> None:
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s: %(message)s",
        datefmt="%Y-%m-%dT%H:%M:%S",
        stream=sys.stdout,
    )


def run_virus(virus_name: str, config: PipelineConfig) -> bool:
    """Run the full pipeline for one virus. Returns True on success."""
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

        # Write preprocessing marker before we touch the output dir
        paths.preprocessing_marker.write_text(
            str(int(__import__("time").time()))
        )

        sort_and_merge.run(config, virus, paths)
        preprocessing.run(config, virus, paths)
        finalize.run(virus_name, config, virus, paths)

    except Exception as exc:
        log.error("Pipeline failed for %s: %s", virus_name, exc)
        try:
            finalize.rollback(virus_name, config, virus, paths)
        except Exception as rb_exc:
            log.error("Rollback also failed: %s", rb_exc)
        return False

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

    failures = []
    for virus_name in viruses:
        if virus_name not in config.viruses:
            log.error("Unknown virus: %s", virus_name)
            failures.append(virus_name)
            continue
        ok = run_virus(virus_name, config)
        if not ok:
            failures.append(virus_name)

    if failures:
        log.error("Pipeline finished with failures: %s", ", ".join(failures))
        sys.exit(1)

    log.info("Pipeline complete")


if __name__ == "__main__":
    main()
