INPUT_DIR = silo_input
SORTED_CHUNKS_DIR = sorted_chunks
SORTED_CHUNKS_FILE = sorted_chunks.file
TMP_DIR = tmp
OUTPUT_DIR = output
SORTED = sorted.ndjson.zst

all: $(SORTED) 

$(SORTED_CHUNKS_DIR):
	mkdir $(SORTED_CHUNKS_DIR)

$(TMP_DIR):
	mkdir $(TMP_DIR)

$(OUTPUT_DIR):
	mkdir $(OUTPUT_DIR)

$(SORTED_CHUNKS_FILE): $(SORTED_CHUNKS_DIR)
	find "$(INPUT_DIR)" -name '*.ndjson.zst' -type f -print0 | xargs -0 -P 16 -I {} sh -c 'zstdcat "{}" | target/release/add_offset | target/release/split_into_sorted_chunks --output-path "$</{}" --chunk-size 100000 --sort-field offset' > $@


$(SORTED): $(SORTED_CHUNKS_FILE) $(TMP_DIR)
	cat $(SORTED_CHUNKS_FILE) | target/release/merge_sorted_chunks --tmp-directory $(TMP_DIR) --sort-field offset | zstd > $@

.PHONY: clean

clean:
	rm -f $(SORTED_CHUNKS_FILE) $(SORTED) 
	rm -rf $(SORTED_CHUNKS_DIR) $(TMP_DIR) $(OUTPUT_DIR)

