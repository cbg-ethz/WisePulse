INPUT_DIR = silo_input
SORTED_CHUNKS_DIR = sorted_chunks
SORTED_CHUNKS_FILE = sorted_chunks.file
TMP_DIR = tmp
SORTED = sorted.ndjson.zst
SILO_OUTPUT_FLAG = silo_output.file

# Configuration for fetch_silo_data
MAX_READS = 100000000
MAX_WEEKS = 6

.PHONY: all fetch-data fresh-data clean

all: $(SILO_OUTPUT_FLAG)

# Fetch fresh data from LAPIS API
fetch-data:
	@echo "Building fetch_silo_data..."
	cargo build --release
	@echo "Fetching data from LAPIS API..."
	./target/release/fetch_silo_data \
		--max-reads $(MAX_READS) \
		--max-weeks $(MAX_WEEKS) \
		--output-dir $(INPUT_DIR)

# Fresh pipeline: fetch new data and process it
fresh-data: clean fetch-data all

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

clean:
	rm -f $(SORTED_CHUNKS_FILE) $(SORTED) $(SILO_OUTPUT_FLAG)
	rm -rf $(SORTED_CHUNKS_DIR) $(TMP_DIR)

