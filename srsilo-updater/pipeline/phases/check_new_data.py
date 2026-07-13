import logging
import subprocess
from pathlib import Path

from pipeline.config import PipelineConfig, VirusConfig, VirusPaths

log = logging.getLogger(__name__)


def run(config: PipelineConfig, virus: VirusConfig, paths: VirusPaths) -> bool:
    """Return True if new data is available, False if nothing to do."""
    log.info("PHASE 2: Checking for new data")
    result = subprocess.run(
        [
            config.binaries() / "check_new_data",
            "--organism", virus.organism,
            "--api-base-url", config.api_base_url,
            "--timestamp-file", str(paths.last_update),
            "--days-back", str(virus.fetch_days),
            "--output-timestamp-file", str(paths.next_timestamp),
        ],
        cwd=paths.base,
    )

    if result.returncode == 0:
        log.info("PHASE 2: New data available — pipeline will run")
        return True
    elif result.returncode == 1:
        log.info("PHASE 2: No new data — skipping pipeline")
        return False
    else:
        raise RuntimeError(f"check_new_data exited with code {result.returncode}")
