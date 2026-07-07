import logging
import shutil
from pathlib import Path

from pipeline.config import PipelineConfig, VirusConfig, VirusPaths
from pipeline.phases import api

log = logging.getLogger(__name__)


def run(virus_name: str, config: PipelineConfig, virus: VirusConfig, paths: VirusPaths) -> None:
    """Phase 7 success path: swap indexes, update timestamp, start API."""
    log.info("PHASE 7: Finalizing")

    new_index = _newest_index(paths)
    if new_index is None:
        raise RuntimeError("No index found in output directory after preprocessing")

    log.info("PHASE 7: New index: %s", new_index.name)

    # Remove preprocessing marker
    paths.preprocessing_marker.unlink(missing_ok=True)

    # Clean up and start API with new index
    api.stop(virus_name, paths)
    api.start(virus_name, virus, paths)

    # Promote next_timestamp -> last_update
    if paths.next_timestamp.exists():
        shutil.copy2(paths.next_timestamp, paths.last_update)
        paths.next_timestamp.unlink()

    log.info("PHASE 7: Done — API running with index %s", new_index.name)


def rollback(virus_name: str, config: PipelineConfig, virus: VirusConfig, paths: VirusPaths) -> None:
    """Rollback after a failed processing run."""
    log.error("ROLLBACK: Starting rollback for %s", virus_name)

    # Delete the failed partial index
    marker = paths.preprocessing_marker
    if marker.exists():
        failed_ts = marker.read_text().strip()
        failed_dir = paths.output / failed_ts
        if failed_dir.exists():
            shutil.rmtree(failed_dir)
            log.info("ROLLBACK: Deleted failed index %s", failed_ts)
        marker.unlink()

    # Clean up next_timestamp
    paths.next_timestamp.unlink(missing_ok=True)

    # Restart API with whatever good index remains
    previous = _newest_index(paths)
    if previous:
        log.info("ROLLBACK: Restarting API with previous index %s", previous.name)
        api.stop(virus_name, paths)
        api.start(virus_name, virus, paths)
    else:
        log.warning("ROLLBACK: No previous index available — API will not be started")

    log.info("ROLLBACK: Complete")


def _newest_index(paths: VirusPaths):
    dirs = [d for d in paths.output.iterdir() if d.is_dir() and not d.name.startswith(".")]
    if not dirs:
        return None
    return max(dirs, key=lambda d: d.stat().st_mtime)
