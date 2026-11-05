"""State builder: a fresh shared state from every virus' sorted.ndjson.zst (no Docker needed)."""
import json
import shutil
import time
from pathlib import Path

import pytest

from pipeline.config import PipelineConfig, VirusConfig
from pipeline.phases import build_state

rhydb = pytest.importorskip("rhydb")
zstandard = pytest.importorskip("zstandard")

SCHEMA = Path(__file__).parent.parent.parent / "roles" / "srsilo" / "files" / "rhydb" / "schema.rhyql"
TABLES = {"covid": "data", "rsva": "rsva", "rsvb": "rsvb"}
COVID_GENES = ["E", "M", "N", "ORF1a", "ORF1b", "ORF3a", "ORF6", "ORF7a", "ORF7b", "ORF8", "ORF9b", "S"]
RSVA_GENES = ["NS1", "NS2", "N", "P", "M", "SH", "G", "F", "M2-1", "M2-2", "L"]
METADATA = {
    "sampleId": "A1_05_2025_06_18", "batchId": "b", "locationCode": "5",
    "samplingDate": "2025-06-18", "locationName": "Lugano (TI)", "sr2siloVersion": "1",
}


def covid_record(read_id):
    return {
        "readId": read_id, **METADATA, "date": "2025-07-01",
        "main": {"sequence": "ACGTACGT", "insertions": [], "offset": 100},
        **{g: None for g in COVID_GENES},
    }


def rsva_record(read_id):
    return {
        "readId": read_id, **METADATA,
        "main": {"sequence": "ACGTACGT", "insertions": [], "offset": 100},
        **{g: None for g in RSVA_GENES},
    }


@pytest.fixture
def config(tmp_path):
    (tmp_path / "rhydb").mkdir()
    shutil.copy(SCHEMA, tmp_path / "rhydb" / "schema.rhyql")
    return PipelineConfig(
        base_path=tmp_path, tools_path=tmp_path, api_base_url="http://unused",
        retention_days=7, retention_min_keep=2, enabled_viruses=list(TABLES),
        viruses={v: VirusConfig(organism=v, table=t, fetch_days=1, fetch_max_reads=1, chunk_size=1)
                 for v, t in TABLES.items()},
    )


def write_sorted(config, virus, records):
    paths = config.virus_paths(virus)
    paths.base.mkdir(parents=True, exist_ok=True)
    data = "".join(json.dumps(r) + "\n" for r in records).encode()
    paths.sorted_file.write_bytes(zstandard.ZstdCompressor().compress(data))
    return paths


def build(config, virus, records):
    write_sorted(config, virus, records)
    build_state.run(config)


def states(config):
    return sorted((d for d in config.rhydb_paths().output.iterdir() if d.is_dir()), key=lambda d: int(d.name))


def read_ids(config, table):
    return sorted(rhydb.Database(str(config.rhydb_paths().output)).query(f"{table}.project({{readId}})").to_pydict()["readId"])


def test_first_build_creates_state_from_schema(config):
    build(config, "covid", [covid_record("c1"), covid_record("c2")])

    assert len(states(config)) == 1
    db = rhydb.Database(str(config.rhydb_paths().output))
    assert db.query("data.project({readId})").num_rows == 2
    assert db.query("rsva.project({readId})").num_rows == 0
    # Same column names, per-virus references
    assert db.get_nucleotide_reference_sequence("data", "main").startswith("ATTAAAGG")
    assert db.get_nucleotide_reference_sequence("rsva", "main").startswith("GTAAACCA")
    assert db.get_amino_acid_reference_sequence("data", "N") != db.get_amino_acid_reference_sequence("rsva", "N")
    assert list(config.rhydb_paths().tmp.iterdir()) == [], "FIFO must be cleaned up"


def test_rebuilds_from_all_sorted_files(config):
    write_sorted(config, "covid", [covid_record("c1"), covid_record("c2")])
    build(config, "rsva", [rsva_record("a1")])
    time.sleep(1.1)  # data versions are timestamps in seconds
    build(config, "rsva", [rsva_record("a2"), rsva_record("a3")])

    assert len(states(config)) == 2
    assert read_ids(config, "rsva") == ["a2", "a3"], "old rows of the table are gone"
    assert read_ids(config, "data") == ["c1", "c2"], "unchanged viruses are rebuilt from their sorted file"
    assert read_ids(config, "rsvb") == [], "virus without a sorted file gets an empty table"


def test_invalid_record_fails_without_new_state(config):
    build(config, "covid", [covid_record("c1")])
    with pytest.raises(Exception):
        build(config, "rsva", [{"readId": "r1"}])  # missing columns
    assert not any(config.rhydb_paths().tmp.glob("*.fifo"))
    assert len(states(config)) == 1, "no state on failure"


def test_failed_build_promotes_no_timestamp(config):
    from pipeline.__main__ import build as build_and_promote

    build(config, "covid", [covid_record("c1")])
    write_sorted(config, "covid", [covid_record("c2")])
    write_sorted(config, "rsva", [{"readId": "r1"}])  # missing columns
    for virus in ("covid", "rsva"):
        config.virus_paths(virus).next_timestamp.write_text("123")

    assert not build_and_promote(config, ["covid", "rsva"])
    assert len(states(config)) == 1, "no state without rsva's data"
    for virus in ("covid", "rsva"):
        assert not config.virus_paths(virus).last_update.exists()
        assert not config.virus_paths(virus).next_timestamp.exists()
