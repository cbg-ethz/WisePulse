# === DIRECTORY STRUCTURE ===
INPUT_DIR = silo_input
SORTED_CHUNKS_DIR = sorted_chunks
TMP_DIR = tmp
SILO_OUTPUT_DIR = silo_output

# === BUILD ARTIFACTS ===
RUST_BINARIES = target/release/split_into_sorted_chunks target/release/merge_sorted_chunks target/release/fetch_silo_data target/release/add_offset target/release/check_new_data
SORTED_CHUNKS_FILE = $(SORTED_CHUNKS_DIR)/chunks.list
SORTED_FILE = sorted.ndjson.zst
SILO_OUTPUT_FLAG = $(SILO_OUTPUT_DIR)/.processed
TIMESTAMP_FILE = .last_update

# === FETCH CONFIGURATION ===
FETCH_START_DATE ?= $(shell date +%Y-%m-%d)
FETCH_DAYS ?= 90
FETCH_MAX_READS ?= 125000000  # Override: FETCH_MAX_READS=1000000 make ...
FETCH_OUTPUT_DIR ?= $(INPUT_DIR)
FETCH_API_BASE_URL ?= https://api.db.wasap.genspectrum.org

# === API CONFIGURATION ===
# LAPIS_PORT: Port for SILO API (default: 8083)
# Set via environment: LAPIS_PORT=8083 make smart-fetch-and-process
LAPIS_PORT ?= 8083

# === MAIN TARGETS ===
.PHONY: all build fetch-data fetch-and-process smart-fetch-and-process clean clean-all help

all: $(SILO_OUTPUT_FLAG)

# Build all Rust tools
build: $(RUST_BINARIES)

# Enhanced clean with options
clean:
	@echo "=== Cleaning intermediate files ==="
	-rm -f $(SORTED_CHUNKS_FILE) $(SORTED_FILE) $(SILO_OUTPUT_FLAG)
	-find $(SORTED_CHUNKS_DIR) -mindepth 1 -delete 2>/dev/null || true
	-find $(TMP_DIR) -mindepth 1 -delete 2>/dev/null || true
	@mkdir -p $(SORTED_CHUNKS_DIR) $(TMP_DIR)
	@echo "✓ Clean complete"

clean-data:
	rm -rf $(INPUT_DIR)/*.ndjson.zst

clean-all: clean clean-data
	cargo clean
	docker compose -f docker-compose-preprocessing.yml down -v

# Help target
help:
	@echo "Available targets:"
	@echo "  build                   - Build all Rust tools"
	@echo "  fetch-data              - Fetch data from LAPIS API"
	@echo "  all                     - Process existing data through pipeline"
	@echo "  fetch-and-process       - Fetch data and run full pipeline (no API mgmt)"
	@echo "  smart-fetch-and-process - Smart: check for new data, stop API, process, restart API"
	@echo "  clean                   - Clean intermediate files"
	@echo "  clean-data              - Clean downloaded data"
	@echo "  clean-all               - Clean everything including Docker"
	@echo ""
	@echo "Configuration:"
	@echo "  LAPIS_PORT              - Port for SILO API (default: 8083)"
	@echo "  FETCH_DAYS              - Days of data to fetch (default: 90)"
	@echo "  FETCH_MAX_READS         - Max reads per batch (default: 125000000)"

# === TARGET IMPLEMENTATIONS ===

# Build individual Rust tools
$(RUST_BINARIES):
	@echo "=== Building Rust tools ==="
	cargo build --release
	@echo "✓ Build complete"
	@echo

# Fetch data from LAPIS API
fetch-data:
	@echo "=== Fetching data from LAPIS API ==="
	cd fetch_silo_data && cargo run --release -- \
		--start-date "$(FETCH_START_DATE)" \
		--days $(FETCH_DAYS) \
		--max-reads $(FETCH_MAX_READS) \
		--output-dir "../$(FETCH_OUTPUT_DIR)" \
		--api-base-url "$(FETCH_API_BASE_URL)"
	@echo

# Convenience target to fetch fresh data and run full pipeline
# Note: This target does NOT manage the API - use smart-fetch-and-process for automated runs
fetch-and-process:
	@echo "=== WisePulse Pipeline ==="
	@$(MAKE) fetch-data
	@$(MAKE) all
	@echo "✓ Pipeline complete"

# Smart pipeline: only fetch and process if new data is available
smart-fetch-and-process: build
	@echo "=== WisePulse Smart Pipeline ==="
	@if target/release/check_new_data --api-base-url "$(FETCH_API_BASE_URL)" --timestamp-file "$(TIMESTAMP_FILE)" --days-back $(FETCH_DAYS) --output-timestamp-file ".next_timestamp"; then \
		echo "=== New data detected - running full pipeline ==="; \
		$(MAKE) clean-data; \
		$(MAKE) clean; \
		$(MAKE) fetch-data; \
		echo "=== Stopping SILO API for preprocessing ==="; \
		docker compose down || true; \
		if $(MAKE) $(SILO_OUTPUT_FLAG); then \
            echo "=== Restarting SILO API ==="; \
            docker compose down --remove-orphans || true; \
            docker network prune -f || true; \
            LAPIS_PORT=$${LAPIS_PORT:-8083} docker compose up -d; \
            cp .next_timestamp "$(TIMESTAMP_FILE)"; \
            rm -f .next_timestamp; \
            echo "✓ Pipeline complete - timestamp updated"; \
        else \
            echo "✗ Pipeline failed - restarting API anyway"; \
            LAPIS_PORT=$${LAPIS_PORT:-8083} docker compose up -d || true; \
            rm -f .next_timestamp; \
            exit 1; \
        fi; \
	else \
		echo "=== No new data - skipping pipeline ==="; \
		rm -f .next_timestamp; \
	fi

# Create directories
$(SORTED_CHUNKS_DIR):
	mkdir -p $(SORTED_CHUNKS_DIR)

$(TMP_DIR):
	mkdir -p $(TMP_DIR)

$(SILO_OUTPUT_DIR):
	mkdir -p $(SILO_OUTPUT_DIR)

# Processing pipeline
$(SORTED_CHUNKS_FILE): $(SORTED_CHUNKS_DIR) build
	@echo "=== Splitting into sorted chunks ==="
	@file_count=$$(find "$(INPUT_DIR)" -name '*.ndjson.zst' -type f | wc -l); \
	echo "Processing $$file_count files..."; \
	> $@; \
	find "$(INPUT_DIR)" -name '*.ndjson.zst' -type f | while read -r file; do \
		echo "  $$(basename "$$file")"; \
		zstdcat "$$file" | target/release/split_into_sorted_chunks --output-path "$(SORTED_CHUNKS_DIR)/$$(basename "$$file")" --chunk-size 1000000 --sort-field-path /main/offset >> $@; \
	done; \
	chunk_count=$$(wc -l < $@ 2>/dev/null || echo 0); \
	echo "✓ Created $$chunk_count chunks"
	@echo

$(SORTED_FILE): $(SORTED_CHUNKS_FILE) $(TMP_DIR) build
	@echo "=== Merging sorted chunks ==="
	@chunk_count=$$(wc -l < $(SORTED_CHUNKS_FILE) 2>/dev/null || echo 0); \
	echo "Merging $$chunk_count chunks..."; \
	cat $(SORTED_CHUNKS_FILE) | target/release/merge_sorted_chunks --tmp-directory $(TMP_DIR) --sort-field-path /main/offset | zstd > $@; \
	file_size=$$(du -h $@ | cut -f1); \
	echo "✓ Created $@ ($$file_size)"
	@echo

$(SILO_OUTPUT_FLAG): $(SORTED_FILE) $(SILO_OUTPUT_DIR)
	@echo "=== SILO preprocessing ==="
	@if command -v docker >/dev/null 2>&1; then \
		echo "Running preprocessing in Docker"; \
		docker compose -f docker-compose-preprocessing.yml down -v 2>/dev/null || true; \
		docker compose -f docker-compose-preprocessing.yml up && \
		echo "✓ SILO preprocessing complete" && \
		touch $(SILO_OUTPUT_FLAG) || \
		(echo "✗ SILO preprocessing failed" && exit 1); \
	else \
		echo "⚠ Docker not found, skipping SILO step"; \
		touch $(SILO_OUTPUT_FLAG); \
	fi