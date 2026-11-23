"""schema.rhyql loading, and its consistency with the duplicated LAPIS config files."""
import json
from pathlib import Path

import pytest
import yaml

from pipeline import schema

FILES = Path(__file__).parent.parent.parent / "roles" / "srsilo" / "files"
VIRUSES = FILES / "viruses"
SCHEMA = FILES / "rhydb" / "schema.rhyql"
# virus -> table, as in srsilo_viruses (roles/srsilo/defaults/main.yml)
TABLES = {"covid": "data", "rsva": "rsva", "rsvb": "rsvb"}
# viruses without a LAPIS instance, so without LAPIS config files to keep in sync
RHYDB_ONLY_TABLES = {v: v for v in ["flua_h1", "flua_h3", "flua_n1", "flua_n2", "flua_mp"]}

# RhyDB schema() type -> (LAPIS database_config type, generateIndex)
LAPIS_TYPES = {
    "STRING": ("string", False),
    "DICTIONARY_ENCODED(STRING)": ("string", True),
    "DATE32": ("date", False),
    "DICTIONARY_ENCODED(DATE32)": ("date", True),
    "INT32": ("int", False),
    "INT64": ("int", False),
    "FLOAT": ("float", False),
    "BOOL": ("boolean", False),
}


def test_statements_end_with_semicolon_at_end_of_line(tmp_path):
    fp = tmp_path / "schema.rhyql"
    fp.write_text(
        "-- header comment; not a statement end\n"
        "tables().limit(1)\n"
        "  .map({name := 'a;b'})\n"
        "  .insertInto(t);\n"
        "\n"
        "-- comment before a statement\n"
        "createTable(data, {x := int32}, primaryKey := x);  \n"
        "-- trailing comment\n"
    )
    statements = schema.load(fp)
    assert statements == [
        "-- header comment; not a statement end\ntables().limit(1)\n  .map({name := 'a;b'})\n  .insertInto(t)",
        "-- comment before a statement\ncreateTable(data, {x := int32}, primaryKey := x)",
    ]
    assert schema.primary_key(statements, "data") == "x"
    with pytest.raises(ValueError, match="no createTable"):
        schema.primary_key(statements, "other")


@pytest.fixture(scope="module")
def shared_db():
    pytest.importorskip("rhydb")
    statements = schema.load(SCHEMA)
    return statements, schema.create_database(statements)


def test_schema_has_a_table_per_virus(shared_db):
    _, db = shared_db
    assert set(TABLES.values()) | set(RHYDB_ONLY_TABLES.values()) <= set(db.query("tables()").to_pydict()["tableName"])


@pytest.mark.parametrize("virus", TABLES)
def test_lapis_config_matches_schema(virus, shared_db):
    """LAPIS keeps its own copy of each virus' table until the configs are consolidated."""
    statements, db = shared_db
    table = TABLES[virus]

    metadata, nucleotides, genes = set(), {}, {}
    for col, rhydb_type in zip(*db.query(f"{table}.schema()").to_pydict().values()):
        if nuc := db.get_nucleotide_reference_sequence(table, col):
            nucleotides[col] = nuc
        elif aa := db.get_amino_acid_reference_sequence(table, col):
            genes[col] = aa
        else:
            metadata.add((col, *LAPIS_TYPES[rhydb_type]))

    config = yaml.safe_load((VIRUSES / virus / "database_config.yaml").read_text())["schema"]
    pk = config["primaryKey"]
    assert metadata == {(m["name"], m["type"], bool(m.get("generateIndex")) and m["name"] != pk) for m in config["metadata"]}
    assert schema.primary_key(statements, table) == pk

    refs = json.loads((VIRUSES / virus / "reference_genomes.json").read_text())
    assert nucleotides == {r["name"]: r["sequence"] for r in refs["nucleotideSequences"]}
    assert genes == {r["name"]: r["sequence"] for r in refs["genes"]}
