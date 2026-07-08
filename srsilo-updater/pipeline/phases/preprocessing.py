import logging
import subprocess

from pipeline.config import PipelineConfig, VirusConfig, VirusPaths

log = logging.getLogger(__name__)


def run(config: PipelineConfig, virus: VirusConfig, paths: VirusPaths) -> None:
    """Phase 6b: run SILO preprocessing container.

    SILO reads sorted.ndjson.zst plus the virus schema and reference genome
    files and builds a binary index directory under output/<timestamp>/. This
    is the longest phase — SILO processes every record and computes its
    internal data structures upfront. The payoff is that loading the finished
    index into the running SILO instance (phase 7) is near-instant.
    """
    if not paths.sorted_file.exists():
        raise RuntimeError(f"Sorted file not found: {paths.sorted_file}")

    compose_file = paths.config / "docker-compose-preprocessing.yml"

    log.info("PHASE 6b: Starting SILO preprocessing container")

    # Clean up any leftover container from a previous run
    subprocess.run(
        ["docker", "compose", "-f", str(compose_file), "down", "-v"],
        cwd=paths.config,
        capture_output=True,
    )

    subprocess.run(
        [
            "docker", "compose",
            "-f", str(compose_file),
            "up",
            "--exit-code-from", "siloPreprocessing",
        ],
        cwd=paths.config,
        check=True,
    )

    log.info("PHASE 6b: SILO preprocessing complete")
