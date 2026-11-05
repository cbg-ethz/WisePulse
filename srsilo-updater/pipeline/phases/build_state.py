"""Phase 6b: build the RhyDB state with the RhyDB Python bindings.

Runs schema.rhyql on an empty database, streams sorted.ndjson.zst into the
`data` table and saves a new state version to output/.
"""
from __future__ import annotations

import logging
import os
import subprocess
import sys

from pipeline.config import PipelineConfig, VirusConfig, VirusPaths
from pipeline import schema
from pipeline.schema import DEFAULT_TABLE

log = logging.getLogger(__name__)

# The bindings don't decompress, and append_data_from_file blocks on a FIFO
# without releasing the GIL, so decompress in a separate process.
_DECOMPRESS = (
    "import sys, zstandard\n"
    "with open(sys.argv[1], 'rb') as src, open(sys.argv[2], 'wb') as dst:\n"
    "    zstandard.ZstdDecompressor().copy_stream(src, dst)\n"
)


def run(config: PipelineConfig, virus: VirusConfig, paths: VirusPaths) -> None:
    if not paths.sorted_file.exists():
        raise RuntimeError(f"Sorted file not found: {paths.sorted_file}")

    statements = schema.load(paths.config / "schema.rhyql")
    log.info("PHASE 6b: Building RhyDB state from %s", paths.sorted_file.name)
    db = schema.create_database(statements)
    append_zst(db, DEFAULT_TABLE, paths)
    rows = row_count(db, DEFAULT_TABLE)
    log.info("PHASE 6b: Appended %d rows", rows)

    paths.output.mkdir(parents=True, exist_ok=True)
    before = set(paths.output.iterdir())
    db.save_checkpoint(str(paths.output))
    new = set(paths.output.iterdir()) - before
    if not new:
        raise RuntimeError("save_checkpoint did not produce a new state version")
    log.info("PHASE 6b: Wrote state %s", max(new).name)


def append_zst(db, table: str, paths: VirusPaths) -> None:
    paths.tmp.mkdir(parents=True, exist_ok=True)
    fifo = paths.tmp / "sorted.ndjson.fifo"
    fifo.unlink(missing_ok=True)
    os.mkfifo(fifo)
    writer = subprocess.Popen(
        [sys.executable, "-c", _DECOMPRESS, str(paths.sorted_file), str(fifo)], stderr=subprocess.PIPE
    )
    try:
        db.append_data_from_file(table, str(fifo))
    finally:
        if writer.poll() is None:
            # The append failed before reading everything: unblock the writer
            try:
                os.close(os.open(fifo, os.O_RDONLY | os.O_NONBLOCK))
            except OSError:
                pass
        try:
            writer.wait(timeout=30)
        except subprocess.TimeoutExpired:
            writer.kill()
            writer.wait()
        fifo.unlink(missing_ok=True)
    if writer.returncode != 0:
        raise RuntimeError(f"Decompressing {paths.sorted_file} failed: {writer.stderr.read().decode()[-500:]}")


def row_count(db, table: str) -> int:
    return db.query(f"{table}.group(by := {{}}, aggs := {{n := count()}})").column("n")[0].as_py()
