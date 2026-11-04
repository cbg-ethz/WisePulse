"""State builder: sorted.ndjson.zst into the `data` table (no Docker needed)."""
import json
import shutil
from pathlib import Path

import pytest

from pipeline.config import VirusPaths
from pipeline.phases import build_state

rhydb = pytest.importorskip("rhydb")
zstandard = pytest.importorskip("zstandard")

VIRUSES = Path(__file__).parent.parent.parent / "roles" / "srsilo" / "files" / "viruses"
COVID_GENES = ["E", "M", "N", "ORF1a", "ORF1b", "ORF3a", "ORF6", "ORF7a", "ORF7b", "ORF8", "ORF9b", "S"]


def covid_record(read_id):
    return {
        "readId": read_id, "sampleId": "A1_05_2025_06_18", "batchId": "b", "locationCode": "5",
        "samplingDate": "2025-06-18", "date": "2025-07-01", "locationName": "Lugano (TI)", "sr2siloVersion": "1",
        "main": {"sequence": "ACGTACGT", "insertions": [], "offset": 100},
        **{g: None for g in COVID_GENES},
    }


def write_sorted(paths, records):
    data = "".join(json.dumps(r) + "\n" for r in records).encode()
    paths.sorted_file.write_bytes(zstandard.ZstdCompressor().compress(data))


@pytest.fixture
def paths(tmp_path):
    paths = VirusPaths(tmp_path / "covid")
    paths.config.mkdir(parents=True)
    shutil.copy(VIRUSES / "covid" / "schema.rhyql", paths.config / "schema.rhyql")
    return paths


def test_builds_state(paths):
    write_sorted(paths, [covid_record("r1"), covid_record("r2")])
    build_state.run(None, None, paths)

    assert len([d for d in paths.output.iterdir() if d.is_dir()]) == 1
    db = rhydb.Database(str(paths.output))
    assert db.query("data.project({readId})").num_rows == 2
    assert db.get_nucleotide_reference_sequence("data", "main").startswith("ATTAAAGG")
    assert list(paths.tmp.iterdir()) == [], "FIFO must be cleaned up"


def test_invalid_record_fails_without_hanging(paths):
    write_sorted(paths, [{"readId": "r1"}])  # missing columns
    with pytest.raises(Exception):
        build_state.run(None, None, paths)
    assert not any(paths.tmp.glob("*.fifo"))
