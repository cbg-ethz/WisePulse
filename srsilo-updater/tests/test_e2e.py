"""End-to-end pipeline test with a mock LAPIS HTTP server.

Covers the full run_virus() path:
  check_new_data (real Rust binary, mock HTTP)
  → cleanup (Python, no external deps)
  → fetch_silo_data (real Rust binary, mock HTTP + file download)
  → split_into_sorted_chunks + merge_sorted_chunks (real Rust binaries, no network)
  → preprocessing (real SILO Docker container)
  → timestamp promotion (__main__.py logic)

Requires:
  - Rust binaries built:  cd rust && cargo build --release
  - Docker + SILO image:  ghcr.io/genspectrum/lapis-silo (pulled by Ansible or manually)
  - Production SILO config files at /opt/srsilo/covid/config/
"""

import json
import shutil
from datetime import date
from pathlib import Path

import pytest
from werkzeug import Request, Response

from pipeline.__main__ import run_virus
from pipeline.config import PipelineConfig, VirusConfig

TEST_DATA = Path(__file__).parent / "data"
_TEST_FILE = "sampleId-D1_10_2025_07_06.ndjson.zst"
_FAKE_TIMESTAMP = 1751500000

_SILO_CONFIG_SRC = Path("/opt/srsilo/covid/config")


def _setup_preprocessing_config(virus_dir: Path) -> None:
    """Populate config/ with SILO config files and a rendered docker-compose."""
    if not _SILO_CONFIG_SRC.exists():
        pytest.skip(f"SILO config not found at {_SILO_CONFIG_SRC} — is this the production server?")

    config_dir = virus_dir / "config"
    for fname in ("preprocessing_config.yaml", "database_config.yaml", "reference_genomes.json"):
        shutil.copy2(_SILO_CONFIG_SRC / fname, config_dir / fname)

    # Read the image name from the production compose so we stay in sync
    prod_compose = _SILO_CONFIG_SRC / "docker-compose-preprocessing.yml"
    import yaml
    image = yaml.safe_load(prod_compose.read_text())["services"]["siloPreprocessing"]["image"]

    output_dir = virus_dir / "output"
    sorted_file = virus_dir / "sorted.ndjson.zst"

    compose = f"""\
services:
  siloPreprocessing:
    container_name: test-silo-preprocessing
    image: {image}
    command: preprocessing
    mem_limit: 2g
    volumes:
      - type: bind
        source: {output_dir}
        target: /preprocessing/output
        read_only: false
      - type: bind
        source: {config_dir}/preprocessing_config.yaml
        target: /app/preprocessing_config.yaml
        read_only: true
      - type: bind
        source: {config_dir}/database_config.yaml
        target: /preprocessing/input/database_config.yaml
        read_only: true
      - type: bind
        source: {config_dir}/reference_genomes.json
        target: /preprocessing/input/reference_genomes.json
        read_only: true
      - type: bind
        source: {sorted_file}
        target: /preprocessing/input/sorted.ndjson.zst
        read_only: true
    stop_grace_period: 5s
"""
    (config_dir / "docker-compose-preprocessing.yml").write_text(compose)


@pytest.fixture
def virus_dir(tmp_path):
    base = tmp_path / "covid"
    for d in ("input", "sorted_chunks", "tmp", "output", "config"):
        (base / d).mkdir(parents=True)
    return base


def test_pipeline_e2e(httpserver, rust_bins, virus_dir):
    today = date.today().isoformat()
    file_bytes = (TEST_DATA / _TEST_FILE).read_bytes()

    _setup_preprocessing_config(virus_dir)

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

    ok = run_virus("covid", config)

    assert ok, "run_virus() returned False — check logs above"

    paths = config.virus_paths("covid")
    assert paths.last_update.exists(), ".last_update was not written"
    assert not paths.next_timestamp.exists(), ".next_timestamp was not cleaned up"
    assert paths.sorted_file.exists(), "sorted.ndjson.zst was not produced"
    assert paths.sorted_file.stat().st_size > 0, "sorted.ndjson.zst is empty"
    assert paths.last_update.read_text().strip() == str(_FAKE_TIMESTAMP)

    # SILO should have written an index directory under output/
    index_dirs = [d for d in paths.output.iterdir() if d.is_dir()]
    assert index_dirs, "SILO did not write any index directory to output/"
    assert (index_dirs[0] / "data_version.silo").exists(), "SILO index is incomplete"
