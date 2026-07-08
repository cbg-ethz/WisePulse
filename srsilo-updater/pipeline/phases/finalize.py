import logging
import shutil

from pipeline.config import PipelineConfig, VirusConfig, VirusPaths
from pipeline.phases import api

log = logging.getLogger(__name__)


def run(virus_name: str, config: PipelineConfig, virus: VirusConfig, paths: VirusPaths) -> None:
    """Phase 7: promote timestamp and ensure API is running.

    SILO continuously scans its output directory and loads new indexes
    automatically when it detects a complete one, so no restart is needed.
    api.start() is a no-op if the containers are already running; it only
    matters on the very first pipeline run.
    """
    log.info("PHASE 7: Finalizing")

    if paths.next_timestamp.exists():
        shutil.copy2(paths.next_timestamp, paths.last_update)
        paths.next_timestamp.unlink()

    api.start(virus_name, virus, paths)

    log.info("PHASE 7: Done")
