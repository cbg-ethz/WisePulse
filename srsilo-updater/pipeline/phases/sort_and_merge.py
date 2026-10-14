import logging
import subprocess

import yaml

from pipeline.config import PipelineConfig, VirusConfig, VirusPaths

log = logging.getLogger(__name__)

SORT_FIELD = "/main/offset"

# Loculus does not send a `date` field — SILO needs one for viruses whose schema
# declares it (see database_config.yaml), so we duplicate `samplingDate` into it
# while streaming the downloaded records. Plain text substitution rather than a
# JSON round-trip: this runs over every single read, and the chunker downstream
# already pays for parsing. Drop this stage once Loculus emits `date` itself.
COPY_SAMPLING_DATE = [
    "sed",
    "-E",
    "-e", r's/"samplingDate":"([^"]*)"/"samplingDate":"\1","date":"\1"/',
    "-e", r's/"samplingDate":null/"samplingDate":null,"date":null/',
]


def _declares_date_field(paths: VirusPaths) -> bool:
    """True if the virus schema declares a `date` metadata field."""
    db_config = paths.config / "database_config.yaml"
    if not db_config.exists():
        log.warning("No database_config.yaml at %s — skipping `date` injection", db_config)
        return False

    with db_config.open() as f:
        schema = yaml.safe_load(f) or {}

    metadata = (schema.get("schema") or {}).get("metadata") or []
    return any(field.get("name") == "date" for field in metadata)


def run(config: PipelineConfig, virus: VirusConfig, paths: VirusPaths) -> None:
    """Phase 6a: split input files into sorted chunks, then merge."""
    input_files = sorted(paths.input.glob("*.ndjson.zst"))
    if not input_files:
        raise RuntimeError(f"No input files found in {paths.input}")

    log.info("PHASE 6a: Splitting %d file(s) into sorted chunks (chunk_size=%d)",
             len(input_files), virus.chunk_size)

    copy_sampling_date = _declares_date_field(paths)
    if copy_sampling_date:
        log.info("PHASE 6a: Schema declares `date` — copying samplingDate into it")

    chunks_list = paths.sorted_chunks / "chunks.list"
    chunks_list.unlink(missing_ok=True)

    bins = config.binaries()

    for input_file in input_files:
        chunk_output = paths.sorted_chunks / input_file.name
        chunk_output.mkdir(parents=True, exist_ok=True)

        # zstdcat <file> [| sed (copy date)] | split_into_sorted_chunks --output-path <dir> ...
        stages = [("zstdcat", subprocess.Popen(
            ["zstdcat", str(input_file)],
            stdout=subprocess.PIPE,
        ))]

        if copy_sampling_date:
            upstream = stages[-1][1].stdout
            stages.append(("sed", subprocess.Popen(
                COPY_SAMPLING_DATE,
                stdin=upstream,
                stdout=subprocess.PIPE,
            )))
            # Only sed needs the read end now, so if it dies zstdcat sees EPIPE
            upstream.close()

        subprocess.run(
            [
                str(bins / "split_into_sorted_chunks"),
                "--output-path", str(chunk_output),
                "--chunk-size", str(virus.chunk_size),
                "--sort-field-path", SORT_FIELD,
            ],
            stdin=stages[-1][1].stdout,
            cwd=paths.base,
            check=True,
        )
        for name, proc in stages:
            proc.wait()
            if proc.returncode != 0:
                raise RuntimeError(f"{name} failed for {input_file.name}")

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
