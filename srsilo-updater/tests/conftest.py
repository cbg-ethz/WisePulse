import pytest
from pathlib import Path

RUST_BINS = Path(__file__).parent.parent / "rust" / "target" / "release"


@pytest.fixture(scope="session")
def rust_bins():
    required = ["check_new_data", "fetch_silo_data", "split_into_sorted_chunks", "merge_sorted_chunks"]
    missing = [b for b in required if not (RUST_BINS / b).exists()]
    if missing:
        pytest.skip(
            f"Rust binaries not built ({', '.join(missing)}). "
            "Run: cd rust && cargo build --release"
        )
    return RUST_BINS
