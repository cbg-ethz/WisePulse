INPUT_DIR = silo_input
SORTED_CHUNKS_DIR = sorted_chunks
SORTED_CHUNKS_FILE = sorted_chunks.file
TMP_DIR = tmp
SORTED = sorted.ndjson.zst
SILO_OUTPUT_FLAG = silo_output.file

all: $(SILO_OUTPUT_FLAG)

$(SORTED_CHUNKS_DIR):
	mkdir $(SORTED_CHUNKS_DIR)

$(TMP_DIR):
	mkdir $(TMP_DIR)

$(SORTED_CHUNKS_FILE): $(SORTED_CHUNKS_DIR)
	find "$(INPUT_DIR)" -name '*.ndjson.zst' -type f -print0 | xargs -0 -P 16 -I {} sh -c 'zstdcat "{}" | target/release/add_offset | target/release/split_into_sorted_chunks --output-path "$</{}" --chunk-size 100000 --sort-field offset' > $@


$(SORTED): $(SORTED_CHUNKS_FILE) $(TMP_DIR)
	cat $(SORTED_CHUNKS_FILE) | target/release/merge_sorted_chunks --tmp-directory $(TMP_DIR) --sort-field offset | zstd > $@

$(SILO_OUTPUT_FLAG): $(SORTED)
	docker compose -f docker-compose-preprocessing.yml up
	touch $(SILO_OUTPUT_FLAG)

.PHONY: clean

clean:
	rm -f $(SORTED_CHUNKS_FILE) $(SORTED) $(SILO_OUTPUT_FLAG)
	rm -rf $(SORTED_CHUNKS_DIR) $(TMP_DIR)

