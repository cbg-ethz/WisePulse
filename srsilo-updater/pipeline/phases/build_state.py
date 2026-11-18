"""Phase 6b: build the shared RhyDB state (Python bindings).

Runs once after all per-virus pipelines, if at least one virus got new data:
creates an empty database from rhydb/schema.rhyql, streams every enabled
virus' sorted.ndjson.zst into its table and saves a new state version. A virus
without a sorted file yet gets an empty table.
"""
from __future__ import annotations

import logging
import os
import subprocess
import sys
from pathlib import Path

from pipeline.config import PipelineConfig
from pipeline import schema

log = logging.getLogger(__name__)

# The bindings don't decompress, and append_data_from_file blocks on a FIFO
# without releasing the GIL, so decompress in a separate process.
_DECOMPRESS = (
    "import sys, zstandard\n"
    "with open(sys.argv[1], 'rb') as src, open(sys.argv[2], 'wb') as dst:\n"
    "    zstandard.ZstdDecompressor().copy_stream(src, dst)\n"
)


def run(config: PipelineConfig) -> None:
    rhydb = config.rhydb_paths()
    log.info("PHASE 6b: Building a fresh RhyDB state from %s", rhydb.schema)
    db = schema.create_database(schema.load(rhydb.schema))
    for virus_name in config.enabled_viruses:
        table = config.viruses[virus_name].table
        sorted_file = config.virus_paths(virus_name).sorted_file
        if not sorted_file.exists():
            log.warning("PHASE 6b: %s not found, table `%s` stays empty", sorted_file, table)
            continue
        append_zst(db, table, sorted_file, rhydb.tmp)
        log.info("PHASE 6b: Appended %d rows to `%s`", row_count(db, table), table)

    rhydb.output.mkdir(parents=True, exist_ok=True)
    before = set(rhydb.output.iterdir())
    db.save_checkpoint(str(rhydb.output))
    new = set(rhydb.output.iterdir()) - before
    if not new:
        raise RuntimeError("save_checkpoint did not produce a new state version")
    log.info("PHASE 6b: Wrote state %s", max(new).name)


def append_zst(db, table: str, sorted_file: Path, tmp: Path) -> None:
    tmp.mkdir(parents=True, exist_ok=True)
    fifo = tmp / f"{table}.ndjson.fifo"
    fifo.unlink(missing_ok=True)
    os.mkfifo(fifo)
    writer = subprocess.Popen(
        [sys.executable, "-c", _DECOMPRESS, str(sorted_file), str(fifo)], stderr=subprocess.PIPE
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
        raise RuntimeError(f"Decompressing {sorted_file} failed: {writer.stderr.read().decode()[-500:]}")


def row_count(db, table: str) -> int:
    return db.query(f"{table}.group(by := {{}}, aggs := {{n := count()}})").column("n")[0].as_py()
