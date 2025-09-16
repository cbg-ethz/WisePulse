# === DIRECTORY STRUCTURE ===
INPUT_DIR = silo_input
SORTED_CHUNKS_DIR = sorted_chunks
TMP_DIR = tmp
SILO_OUTPUT_DIR = silo_output

# === BUILD ARTIFACTS ===
RUST_BINARIES = target/release/split_into_sorted_chunks target/release/merge_sorted_chunks target/release/fetch_silo_data target/release/add_offset
SORTED_CHUNKS_FILE = $(SORTED_CHUNKS_DIR)/chunks.list
SORTED_FILE = sorted.ndjson.zst
SILO_OUTPUT_FLAG = $(SILO_OUTPUT_DIR)/.processed

# === FETCH CONFIGURATION ===
FETCH_START_DATE ?= $(shell date +%Y-%m-%d)
FETCH_DAYS ?= 60
FETCH_MAX_READS ?= 1000000
FETCH_OUTPUT_DIR ?= $(INPUT_DIR)
FETCH_API_BASE_URL ?= https://api.db.wasap.genspectrum.org

# === MAIN TARGETS ===
.PHONY: all build fetch-data fetch-and-process clean clean-all help

all: $(SILO_OUTPUT_FLAG)

# Build all Rust tools
build: $(RUST_BINARIES)

# Enhanced clean with options
clean:
	rm -f $(SORTED_CHUNKS_FILE) $(SORTED_FILE) $(SILO_OUTPUT_FLAG)
	rm -rf $(SORTED_CHUNKS_DIR) $(TMP_DIR)

clean-data:
	rm -rf $(INPUT_DIR)/*.ndjson.zst

clean-all: clean clean-data
	cargo clean
	docker compose -f docker-compose-preprocessing.yml down -v

# Help target
help:
	@echo "Available targets:"
	@echo "  build             - Build all Rust tools"
	@echo "  fetch-data        - Fetch data from LAPIS API"
	@echo "  all               - Process existing data through pipeline"
	@echo "  fetch-and-process - Fetch data and run full pipeline"
	@echo "  clean             - Clean intermediate files"
	@echo "  clean-data        - Clean downloaded data"
	@echo "  clean-all         - Clean everything including Docker"

# === TARGET IMPLEMENTATIONS ===

# Build individual Rust tools
$(RUST_BINARIES):
	cargo build --release

# Fetch data from LAPIS API
fetch-data:
	cd fetch_silo_data && cargo run --release -- \
		--start-date "$(FETCH_START_DATE)" \
		--days $(FETCH_DAYS) \
		--max-reads $(FETCH_MAX_READS) \
		--output-dir "../$(FETCH_OUTPUT_DIR)" \
		--api-base-url "$(FETCH_API_BASE_URL)"

# Convenience target to fetch fresh data and run full pipeline
fetch-and-process: fetch-data all

# Create directories
$(SORTED_CHUNKS_DIR):
	mkdir -p $(SORTED_CHUNKS_DIR)

$(TMP_DIR):
	mkdir -p $(TMP_DIR)

$(SILO_OUTPUT_DIR):
	mkdir -p $(SILO_OUTPUT_DIR)

# Processing pipeline
$(SORTED_CHUNKS_FILE): $(SORTED_CHUNKS_DIR) build
	find "$(INPUT_DIR)" -name '*.ndjson.zst' -type f -print0 | xargs -0 -P 16 -I {} sh -c 'zstdcat "{}" | target/release/split_into_sorted_chunks --output-path "$(SORTED_CHUNKS_DIR)/{}" --chunk-size 1000000 --sort-field-path /main/offset' > $@

$(SORTED_FILE): $(SORTED_CHUNKS_FILE) $(TMP_DIR) build
	cat $(SORTED_CHUNKS_FILE) | target/release/merge_sorted_chunks --tmp-directory $(TMP_DIR) --sort-field-path /main/offset | zstd > $@

$(SILO_OUTPUT_FLAG): $(SORTED_FILE) $(SILO_OUTPUT_DIR)
	@echo "=== Docker preprocessing step ==="
	@echo "Running: docker compose -f docker-compose-preprocessing.yml up"
	@if command -v docker >/dev/null 2>&1; then \
		docker compose -f docker-compose-preprocessing.yml up && touch $(SILO_OUTPUT_FLAG); \
	else \
		echo "Warning: Docker not found. Skipping Docker preprocessing step."; \
		echo "The pipeline has successfully processed data up to: $(SORTED_FILE)"; \
		touch $(SILO_OUTPUT_FLAG); \
	fi