import logging
import subprocess
from pathlib import Path

from pipeline.config import PipelineConfig, VirusConfig, VirusPaths

log = logging.getLogger(__name__)

SORT_FIELD = "/main/offset"


def run(config: PipelineConfig, virus: VirusConfig, paths: VirusPaths) -> None:
    """Phase 6a: split input files into sorted chunks, then merge."""
    input_files = sorted(paths.input.glob("*.ndjson.zst"))
    if not input_files:
        raise RuntimeError(f"No input files found in {paths.input}")

    log.info("PHASE 6a: Splitting %d file(s) into sorted chunks (chunk_size=%d)",
             len(input_files), virus.chunk_size)

    chunks_list = paths.sorted_chunks / "chunks.list"
    chunks_list.unlink(missing_ok=True)

    bins = config.binaries()

    for input_file in input_files:
        chunk_output = paths.sorted_chunks / input_file.name
        chunk_output.mkdir(parents=True, exist_ok=True)

        # zstdcat <file> | split_into_sorted_chunks --output-path <dir> ...
        zstdcat = subprocess.Popen(
            ["zstdcat", str(input_file)],
            stdout=subprocess.PIPE,
        )
        subprocess.run(
            [
                str(bins / "split_into_sorted_chunks"),
                "--output-path", str(chunk_output),
                "--chunk-size", str(virus.chunk_size),
                "--sort-field-path", SORT_FIELD,
            ],
            stdin=zstdcat.stdout,
            cwd=paths.base,
            check=True,
        )
        zstdcat.wait()
        if zstdcat.returncode != 0:
            raise RuntimeError(f"zstdcat failed for {input_file.name}")

        # Append chunk paths to the list file
        chunk_files = sorted(chunk_output.glob("chunk_*.ndjson.zst"))
        with chunks_list.open("a") as f:
            for c in chunk_files:
                f.write(str(c) + "\n")

    chunk_count = sum(1 for _ in chunks_list.open())
    log.info("PHASE 6a: Created %d chunk(s)", chunk_count)

    log.info("PHASE 6a: Merging chunks -> %s", paths.sorted_file)

    # cat chunks.list | merge_sorted_chunks ... | zstd > sorted.ndjson.zst
    with chunks_list.open() as chunk_input:
        merge = subprocess.Popen(
            [
                str(bins / "merge_sorted_chunks"),
                "--tmp-directory", str(paths.tmp),
                "--sort-field-path", SORT_FIELD,
            ],
            stdin=chunk_input,
            stdout=subprocess.PIPE,
            cwd=paths.base,
        )
        with paths.sorted_file.open("wb") as out:
            subprocess.run(
                ["zstd"],
                stdin=merge.stdout,
                stdout=out,
                check=True,
            )
        merge.wait()
        if merge.returncode != 0:
            raise RuntimeError("merge_sorted_chunks failed")

    size_mb = paths.sorted_file.stat().st_size / 1024 / 1024
    log.info("PHASE 6a: Sort/merge complete — %.1f MB", size_mb)
