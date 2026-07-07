import logging
import subprocess

from pipeline.config import PipelineConfig, VirusConfig, VirusPaths

log = logging.getLogger(__name__)


def run(config: PipelineConfig, virus: VirusConfig, paths: VirusPaths) -> None:
    """Phase 6b: run SILO preprocessing container."""
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
