"""schema.rhyql: the RhyDB write statements that create an instance's tables.

Every statement ends with `;` at the end of a line; `--` starts a comment
(handled by RhyDB). The statements are not parsed beyond that.
"""
from __future__ import annotations

import re
from pathlib import Path
from typing import List

DEFAULT_TABLE = "data"

_STATEMENT_END = re.compile(r";[ \t]*$", re.MULTILINE)


def load(path: Path) -> List[str]:
    """The statements of a schema.rhyql file, in order."""
    chunks = _STATEMENT_END.split(Path(path).read_text())
    return [c.strip() for c in chunks if any(_is_code(line) for line in c.splitlines())]


def _is_code(line: str) -> bool:
    line = line.strip()
    return bool(line) and not line.startswith("--")


def create_database(statements: List[str]):
    """An empty in-memory database with all references and tables created."""
    from rhydb import Database

    db = Database()
    for statement in statements:
        try:
            db.query(statement)
        except Exception as e:
            raise ValueError(f"schema statement failed: {statement[:120]}...: {e}") from e
    return db


def primary_key(statements: List[str], table: str) -> str | None:
    """The `primaryKey := ...` of the `createTable(<table>, ...)` statement."""
    for statement in statements:
        if re.search(rf"\bcreateTable\(\s*{re.escape(table)}\s*,", statement):
            match = re.search(r"\bprimaryKey\s*:=\s*(\w+)", statement)
            return match.group(1) if match else None
    raise ValueError(f"no createTable({table}, ...) statement")
