import logging
import shutil

from pipeline.config import PipelineConfig, VirusConfig, VirusPaths

log = logging.getLogger(__name__)


def run(virus_name: str, config: PipelineConfig, virus: VirusConfig, paths: VirusPaths) -> None:
    """Phase 7: promote timestamp.

    SILO continuously scans its output directory and loads new complete indexes
    automatically, so no API restart is needed.
    """
    log.info("PHASE 7: Finalizing")

    if paths.next_timestamp.exists():
        shutil.copy2(paths.next_timestamp, paths.last_update)
        paths.next_timestamp.unlink()

    log.info("PHASE 7: Done")
