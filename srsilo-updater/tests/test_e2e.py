"""End-to-end pipeline test with a mock LAPIS HTTP server.

Covers the full run_virus() + build() path:
  check_new_data (real Rust binary, mock HTTP)
  → cleanup (Python, no external deps)
  → fetch_silo_data (real Rust binary, mock HTTP + file download)
  → split_into_sorted_chunks + merge_sorted_chunks (real Rust binaries, no network)
  → build_state from all sorted data (RhyDB Python bindings, shared schema from this repo)
  → timestamp promotion (__main__.py logic)

Requires:
  - Rust binaries built:  cd rust && cargo build --release
"""

import json
import shutil
from datetime import date
from pathlib import Path

import pytest
from werkzeug import Request, Response

from pipeline.__main__ import build, run_virus
from pipeline.config import PipelineConfig, VirusConfig

TEST_DATA = Path(__file__).parent / "data"
_TEST_FILE = "sampleId-D1_10_2025_07_06.ndjson.zst"
_FAKE_TIMESTAMP = 1751500000

FILES = Path(__file__).parent.parent.parent / "roles" / "srsilo" / "files"


@pytest.fixture
def virus_dir(tmp_path):
    base = tmp_path / "covid"
    for d in ("input", "sorted_chunks", "tmp", "config"):
        (base / d).mkdir(parents=True)
    (tmp_path / "rhydb").mkdir()
    shutil.copy2(FILES / "rhydb" / "schema.rhyql", tmp_path / "rhydb")
    return base


def test_pipeline_e2e(httpserver, rust_bins, virus_dir):
    today = date.today().isoformat()
    file_bytes = (TEST_DATA / _TEST_FILE).read_bytes()

    pytest.importorskip("rhydb")
    # database_config.yaml: `date` injection
    shutil.copy2(FILES / "viruses" / "covid" / "database_config.yaml", virus_dir / "config")

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
            table="data",
            fetch_days=2,           # keep small: limits fetch loop iterations + 100ms sleeps
            fetch_max_reads=10_000_000,
            chunk_size=30_000,
        )},
    )

    staged = []
    assert run_virus("covid", config, staged), "run_virus() returned False — check logs above"
    assert build(config, staged), "build() returned False — check logs above"

    paths = config.virus_paths("covid")
    assert paths.last_update.exists(), ".last_update was not written"
    assert not paths.next_timestamp.exists(), ".next_timestamp was not cleaned up"
    assert paths.sorted_file.exists(), "sorted.ndjson.zst was not produced"
    assert paths.sorted_file.stat().st_size > 0, "sorted.ndjson.zst is empty"
    assert paths.last_update.read_text().strip() == str(_FAKE_TIMESTAMP)

    # The builder should have written one shared RhyDB state under rhydb/output/
    output = config.rhydb_paths().output
    index_dirs = [d for d in output.iterdir() if d.is_dir()]
    assert len(index_dirs) == 1, "expected exactly one RhyDB state in rhydb/output/"
    assert (index_dirs[0] / "data_version.silo").exists(), "RhyDB state is incomplete"

    from rhydb import Database
    db = Database(str(output))
    rows = db.query("data.group(by := {}, aggs := {n := count()})").column("n")[0].as_py()
    assert rows > 0, "data table is empty"
    dates = db.query("data.filter(isNotNull(date)).project({readId})").num_rows
    assert dates == rows, "`date` must be copied from samplingDate for covid"
