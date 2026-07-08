"""End-to-end pipeline test with a mock LAPIS HTTP server.

Covers the full run_virus() path:
  check_new_data (real Rust binary, mock HTTP)
  → cleanup (Python, no external deps)
  → fetch_silo_data (real Rust binary, mock HTTP + file download)
  → split_into_sorted_chunks + merge_sorted_chunks (real Rust binaries, no network)
  → preprocessing (mocked — requires Docker/SILO)
  → timestamp promotion (__main__.py logic)

Requires the Rust binaries to be built first:
  cd rust && cargo build --release
"""

import json
import unittest.mock
from datetime import date
from pathlib import Path

import pytest
from werkzeug import Request, Response

from pipeline.__main__ import run_virus
from pipeline.config import PipelineConfig, VirusConfig

TEST_DATA = Path(__file__).parent / "data"
_TEST_FILE = "sampleId-D1_10_2025_07_06.ndjson.zst"
_FAKE_TIMESTAMP = 1751500000


@pytest.fixture
def virus_dir(tmp_path):
    base = tmp_path / "covid"
    for d in ("input", "sorted_chunks", "tmp", "output", "config"):
        (base / d).mkdir(parents=True)
    return base


def test_pipeline_e2e(httpserver, rust_bins, virus_dir):
    today = date.today().isoformat()
    file_bytes = (TEST_DATA / _TEST_FILE).read_bytes()

    # Serve the .ndjson.zst file that fetch_silo_data will download
    httpserver.expect_request(f"/files/{_TEST_FILE}").respond_with_data(
        file_bytes, content_type="application/octet-stream"
    )

    server_url = httpserver.url_for("").rstrip("/")

    def api_handler(request: Request) -> Response:
        """Dispatch all /covid/sample/details calls based on query params."""
        args = request.args

        # check_new_data: revocation check
        if "isRevocation" in args:
            return Response('{"data":[]}', content_type="application/json")

        # check_new_data: new submissions check (first run, no .last_update)
        if "submittedAtTimestampFrom" in args:
            body = json.dumps({
                "data": [{"sampleId": "s1", "submittedAtTimestamp": _FAKE_TIMESTAMP}]
            })
            return Response(body, content_type="application/json")

        # fetch_silo_data: per-date metadata query
        if args.get("samplingDate") == today:
            payload = {"data": [{
                "sampleId": "D1_10",
                "samplingDate": today,
                "countSiloReads": "1000",
                "siloReads": json.dumps([{
                    "name": _TEST_FILE,
                    "url": f"{server_url}/files/{_TEST_FILE}",
                }]),
            }]}
            return Response(json.dumps(payload), content_type="application/json")

        # All other dates → no data
        return Response('{"data":[]}', content_type="application/json")

    httpserver.expect_request("/covid/sample/details").respond_with_handler(api_handler)

    config = PipelineConfig(
        base_path=virus_dir.parent,
        tools_path=rust_bins.parent.parent,  # rust/ dir; binaries() → rust/target/release/
        api_base_url=server_url,
        retention_days=7,
        retention_min_keep=1,
        enabled_viruses=["covid"],
        viruses={"covid": VirusConfig(
            organism="covid",
            instance_name="covid",
            lapis_port=8080,
            silo_port=8081,
            fetch_days=2,           # keep small: limits fetch loop iterations + 100ms sleeps
            fetch_max_reads=10_000_000,
            chunk_size=30_000,
            docker_memory_limit="4g",
        )},
    )

    with unittest.mock.patch("pipeline.phases.preprocessing.run"):
        ok = run_virus("covid", config)

    assert ok, "run_virus() returned False — check logs above"

    paths = config.virus_paths("covid")
    assert paths.last_update.exists(), ".last_update was not written"
    assert not paths.next_timestamp.exists(), ".next_timestamp was not cleaned up"
    assert paths.sorted_file.exists(), "sorted.ndjson.zst was not produced"
    assert paths.sorted_file.stat().st_size > 0, "sorted.ndjson.zst is empty"
    assert paths.last_update.read_text().strip() == str(_FAKE_TIMESTAMP)
