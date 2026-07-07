import logging
import shutil
import time
from pathlib import Path

from pipeline.config import PipelineConfig, VirusConfig, VirusPaths

log = logging.getLogger(__name__)


def run(config: PipelineConfig, virus: VirusConfig, paths: VirusPaths) -> None:
    """Phase 3: retention policy, orphan cleanup, reset working dirs."""
    log.info("PHASE 3: Cleanup")
    _apply_retention(paths, config.retention_days, config.retention_min_keep)
    _cleanup_orphan(paths)
    _reset_working_dirs(paths)
    log.info("PHASE 3: Cleanup complete")


def _apply_retention(paths: VirusPaths, retention_days: int, min_keep: int) -> None:
    index_dirs = sorted(
        [d for d in paths.output.iterdir() if d.is_dir() and not d.name.startswith(".")],
        key=lambda d: d.stat().st_mtime,
    )
    total = len(index_dirs)
    log.info("Retention: found %d index(es)", total)

    now = time.time()
    cutoff = now - retention_days * 86400

    old = [d for d in index_dirs if d.stat().st_mtime < cutoff]
    # Never delete below min_keep
    deletable = old[: max(0, total - min_keep)]

    for d in deletable:
        log.info("Retention: deleting old index %s", d.name)
        shutil.rmtree(d)

    if not deletable:
        log.info("Retention: nothing to delete")


def _cleanup_orphan(paths: VirusPaths) -> None:
    marker = paths.preprocessing_marker
    if not marker.exists():
        return

    orphan_ts = marker.read_text().strip()
    log.warning("Found orphaned preprocessing marker (ts=%s) — cleaning up", orphan_ts)

    orphan_dir = paths.output / orphan_ts
    if orphan_dir.exists():
        shutil.rmtree(orphan_dir)
        log.info("Deleted orphaned index %s", orphan_ts)

    marker.unlink()


def _reset_working_dirs(paths: VirusPaths) -> None:
    for d in (paths.input, paths.sorted_chunks, paths.tmp):
        if d.exists():
            shutil.rmtree(d)
        d.mkdir(parents=True, exist_ok=True)
        log.info("Reset directory: %s", d)
