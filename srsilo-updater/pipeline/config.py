from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List

import yaml


@dataclass
class VirusConfig:
    organism: str
    table: str  # table of the shared RhyDB instance that holds this virus
    fetch_days: int
    fetch_max_reads: int
    chunk_size: int


@dataclass
class PipelineConfig:
    base_path: Path
    tools_path: Path
    api_base_url: str
    retention_days: int
    retention_min_keep: int
    enabled_viruses: List[str]
    viruses: Dict[str, VirusConfig]

    @classmethod
    def load(cls, path: Path) -> "PipelineConfig":
        with open(path) as f:
            data = yaml.safe_load(f)

        viruses = {
            name: VirusConfig(
                organism=cfg["organism"],
                table=cfg["table"],
                fetch_days=int(cfg["fetch_days"]),
                fetch_max_reads=int(cfg["fetch_max_reads"]),
                chunk_size=int(cfg["chunk_size"]),
            )
            for name, cfg in data["viruses"].items()
        }

        return cls(
            base_path=Path(data["base_path"]),
            tools_path=Path(data["tools_path"]),
            api_base_url=data["api_base_url"],
            retention_days=int(data["retention_days"]),
            retention_min_keep=int(data["retention_min_keep"]),
            enabled_viruses=data["enabled_viruses"],
            viruses=viruses,
        )

    def virus_paths(self, virus: str) -> "VirusPaths":
        return VirusPaths(self.base_path / virus)

    def rhydb_paths(self) -> "RhydbPaths":
        return RhydbPaths(self.base_path / "rhydb")

    def binaries(self) -> Path:
        return self.tools_path / "target" / "release"


@dataclass
class VirusPaths:
    base: Path

    @property
    def input(self) -> Path:
        return self.base / "input"

    @property
    def sorted_chunks(self) -> Path:
        return self.base / "sorted_chunks"

    @property
    def tmp(self) -> Path:
        return self.base / "tmp"

    @property
    def config(self) -> Path:
        return self.base / "config"

    @property
    def last_update(self) -> Path:
        return self.base / ".last_update"

    @property
    def next_timestamp(self) -> Path:
        return self.base / ".next_timestamp"

    @property
    def sorted_file(self) -> Path:
        return self.base / "sorted.ndjson.zst"


@dataclass
class RhydbPaths:
    base: Path

    @property
    def schema(self) -> Path:
        return self.base / "schema.rhyql"

    @property
    def output(self) -> Path:
        return self.base / "output"

    @property
    def tmp(self) -> Path:
        return self.base / "tmp"
