INPUT_DIR = silo_input
SORTED_CHUNKS_DIR = sorted_chunks
SORTED_CHUNKS_FILE = sorted_chunks.file
TMP_DIR = tmp
SORTED = sorted.ndjson.zst
SILO_OUTPUT_FLAG = silo_output.file

# Fetch configuration variables
FETCH_START_DATE ?= $(shell date +%Y-%m-%d)
FETCH_DAYS ?= 7
FETCH_MAX_READS ?= 1000000
FETCH_OUTPUT_DIR ?= $(INPUT_DIR)

all: $(SILO_OUTPUT_FLAG)

# Fetch data from LAPIS API
.PHONY: fetch-data
fetch-data:
	cd fetch_silo_data && cargo run --release -- \
		--start-date "$(FETCH_START_DATE)" \
		--days $(FETCH_DAYS) \
		--max-reads $(FETCH_MAX_READS) \
		--output-dir "../$(FETCH_OUTPUT_DIR)"

# Convenience target to fetch fresh data and run full pipeline
.PHONY: fresh-data
fresh-data: fetch-data all

$(SORTED_CHUNKS_DIR):
	mkdir $(SORTED_CHUNKS_DIR)

$(TMP_DIR):
	mkdir $(TMP_DIR)

$(SORTED_CHUNKS_FILE): $(SORTED_CHUNKS_DIR)
	find "$(INPUT_DIR)" -name '*.ndjson.zst' -type f -print0 | xargs -0 -P 16 -I {} sh -c 'zstdcat "{}" | target/release/split_into_sorted_chunks --output-path "$</{}" --chunk-size 1000000 --sort-field-path /main/offset' > $@


$(SORTED): $(SORTED_CHUNKS_FILE) $(TMP_DIR)
	cat $(SORTED_CHUNKS_FILE) | target/release/merge_sorted_chunks --tmp-directory $(TMP_DIR) --sort-field-path /main/offset | zstd > $@

$(SILO_OUTPUT_FLAG): $(SORTED)
	sudo docker compose -f docker-compose-preprocessing.yml up
	touch $(SILO_OUTPUT_FLAG)

.PHONY: clean

clean:
	rm -f $(SORTED_CHUNKS_FILE) $(SORTED) $(SILO_OUTPUT_FLAG)
	rm -rf $(SORTED_CHUNKS_DIR) $(TMP_DIR)

