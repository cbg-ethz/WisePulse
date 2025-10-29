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

# === RETENTION POLICY ===
RETENTION_DAYS ?= 7
RETENTION_MIN_KEEP ?= 2

# === MAIN TARGETS ===

.PHONY: all
all: $(SILO_OUTPUT_FLAG)

# Build all Rust tools (force rebuild to ensure latest code)
.PHONY: build
build:
	@echo "=== Building Rust tools ==="
	cargo build --release
	@echo "✓ Build complete"

# Enhanced clean with options
.PHONY: clean
clean:
	@echo "=== Cleaning intermediate files ==="
	rm -f $(SORTED_CHUNKS_FILE) $(SORTED_FILE) $(SILO_OUTPUT_FLAG)
	find $(SORTED_CHUNKS_DIR) -mindepth 1 -delete || sudo find $(SORTED_CHUNKS_DIR) -mindepth 1 -delete
	find $(TMP_DIR) -mindepth 1 -delete || sudo find $(TMP_DIR) -mindepth 1 -delete
	@mkdir -p $(SORTED_CHUNKS_DIR) $(TMP_DIR)
	@echo "✓ Clean complete"

.PHONY: clean-data
clean-data:
	rm -rf $(INPUT_DIR)/*.ndjson.zst

.PHONY: clean-all
clean-all: clean clean-data
	cargo clean
	docker compose -f docker-compose-preprocessing.yml down -v

# Help target
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  build                   - Build all Rust tools"
	@echo "  fetch-data              - Fetch data from LAPIS API"
	@echo "  all                     - Process existing data through pipeline"
	@echo "  fetch-and-process       - Fetch data and run full pipeline (no API mgmt)"
	@echo "  smart-fetch-and-process - Smart: check for new data, stop API, process, restart API"
	@echo "  cleanup-old-indexes     - Run retention policy on SILO indexes"
	@echo "  clean                   - Clean intermediate files"
	@echo "  clean-data              - Clean downloaded data"
	@echo "  clean-all               - Clean everything including Docker"
	@echo ""
	@echo "Configuration:"
	@echo "  LAPIS_PORT              - Port for SILO API (default: 8083)"
	@echo "  FETCH_DAYS              - Days of data to fetch (default: 90)"
	@echo "  FETCH_MAX_READS         - Max reads per batch (default: 125000000)"
	@echo "  RETENTION_DAYS          - Keep indexes newer than N days (default: 7)"
	@echo "  RETENTION_MIN_KEEP      - Always keep N newest indexes (default: 2)"

# === TARGET IMPLEMENTATIONS ===

# Cleanup old SILO indexes (retention policy)
.PHONY: cleanup-old-indexes
cleanup-old-indexes:
	@echo "Running retention policy (keep $(RETENTION_MIN_KEEP) newest, delete $(RETENTION_DAYS)+ days old)"
	@find $(SILO_OUTPUT_DIR) -maxdepth 1 -type d -mtime +$(RETENTION_DAYS) -print0 2>/dev/null \
		| sort -n --zero-terminated \
		| head -n -$(RETENTION_MIN_KEEP) --zero-terminated \
		| xargs --null -I {} sh -c 'echo "Deleting old index: $$(basename {})"; sudo rm -rf {}' || true

# Fetch data from LAPIS API
.PHONY: fetch-data
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
.PHONY: fetch-and-process
fetch-and-process:
	@echo "=== WisePulse Pipeline ==="
	@$(MAKE) fetch-data
	@$(MAKE) all
	@echo "✓ Pipeline complete"

# Smart pipeline: only fetch and process if new data is available
.PHONY: smart-fetch-and-process
smart-fetch-and-process: build
	@echo "=== WisePulse Smart Pipeline ==="
	@if target/release/check_new_data --api-base-url "$(FETCH_API_BASE_URL)" --timestamp-file "$(TIMESTAMP_FILE)" --days-back $(FETCH_DAYS) --output-timestamp-file ".next_timestamp"; then \
		echo "New data detected - running full pipeline"; \
		$(MAKE) cleanup-old-indexes; \
		if [ -f "$(SILO_OUTPUT_DIR)/.preprocessing_in_progress" ]; then \
			orphan=$$(cat "$(SILO_OUTPUT_DIR)/.preprocessing_in_progress"); \
			echo "Cleaning up orphaned preprocessing: $$orphan"; \
			rm -rf "$(SILO_OUTPUT_DIR)/$$orphan" 2>/dev/null || true; \
			rm -f "$(SILO_OUTPUT_DIR)/.preprocessing_in_progress"; \
		fi; \
		$(MAKE) clean-data; \
		$(MAKE) clean; \
		$(MAKE) fetch-data; \
		echo "Stopping SILO API for preprocessing"; \
		docker compose down || true; \
		date +%s > "$(SILO_OUTPUT_DIR)/.preprocessing_in_progress"; \
		if $(MAKE) $(SILO_OUTPUT_FLAG); then \
			echo "✓ Preprocessing successful"; \
			new_index=$$(find $(SILO_OUTPUT_DIR) -maxdepth 1 -type d 2>/dev/null | sort -n | tail -1 | xargs basename 2>/dev/null || echo ""); \
			rm -f "$(SILO_OUTPUT_DIR)/.preprocessing_in_progress"; \
			docker compose down --remove-orphans || true; \
			docker network prune -f || true; \
			echo "Starting API with index: $$new_index"; \
			LAPIS_PORT=$${LAPIS_PORT:-8083} docker compose up -d; \
			cp .next_timestamp "$(TIMESTAMP_FILE)"; \
			rm -f .next_timestamp; \
			echo "✓ Pipeline complete"; \
		else \
			echo "✗ Preprocessing failed"; \
			# Rollback: delete bad index, restart API with previous good index \
			failed=$$(cat "$(SILO_OUTPUT_DIR)/.preprocessing_in_progress" 2>/dev/null || echo ""); \
			[ -n "$$failed" ] && rm -rf "$(SILO_OUTPUT_DIR)/$$failed" 2>/dev/null || true; \
			rm -f "$(SILO_OUTPUT_DIR)/.preprocessing_in_progress"; \
			LAPIS_PORT=$${LAPIS_PORT:-8083} docker compose up -d || true; \
			rm -f .next_timestamp; \
			exit 1; \
		fi; \
	else \
		echo "No new data - skipping pipeline"; \
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
	@echo "Ensuring sorted_chunks is clean..."
	@find $(SORTED_CHUNKS_DIR) -mindepth 1 -delete 2>/dev/null || sudo find $(SORTED_CHUNKS_DIR) -mindepth 1 -delete 2>/dev/null || true
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
	@echo "Ensuring tmp is clean..."
	@find $(TMP_DIR) -mindepth 1 -delete 2>/dev/null || sudo find $(TMP_DIR) -mindepth 1 -delete 2>/dev/null || true
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