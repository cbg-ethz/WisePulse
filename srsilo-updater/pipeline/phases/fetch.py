import logging
import subprocess
from datetime import date

from pipeline.config import PipelineConfig, VirusConfig, VirusPaths

log = logging.getLogger(__name__)


def run(config: PipelineConfig, virus: VirusConfig, paths: VirusPaths) -> None:
    """Phase 4: fetch data from LAPIS API."""
    start_date = date.today().isoformat()
    log.info("PHASE 4: Fetching data (start=%s, days=%d, max_reads=%d)",
             start_date, virus.fetch_days, virus.fetch_max_reads)

    subprocess.run(
        [
            config.binaries() / "fetch_silo_data",
            "--organism", virus.organism,
            "--start-date", start_date,
            "--days", str(virus.fetch_days),
            "--max-reads", str(virus.fetch_max_reads),
            "--output-dir", str(paths.input),
            "--api-base-url", config.api_base_url,
        ],
        cwd=paths.base,
        check=True,
    )

    downloaded = list(paths.input.glob("*.ndjson.zst"))
    if not downloaded:
        raise RuntimeError("No files downloaded from API — check API connectivity and fetch window")

    log.info("PHASE 4: Fetched %d file(s)", len(downloaded))
