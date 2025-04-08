#!/bin/bash

# This script generates the input data for SILO.
# I use this for testing how it could be used in Loculus to extract all the short-read data from S3

# Input file with metadata
INPUT_FILE="loculus_output.ndjson"

# Output file for the combined ndjson
OUTPUT_FILE="data.ndjson"

# Temporary file for storing S3 links
S3_LINKS_FILE="s3_links.txt"

curl -X 'GET' \
  'https://backend-wise-seqs.loculus.org/test/get-released-data' \
  -H 'accept: application/x-ndjson' \
  -H 'x-request-id: 1747481c-816c-4b60-af20-a61717a35067' > "$INPUT_FILE"

# Extract S3 links from the metadata
jq -r '.metadata.s3Link | select(test("\\.ndjson.bz2$"))' "$INPUT_FILE" > "$S3_LINKS_FILE"

rm "$INPUT_FILE"

# Ensure the output file is empty
rm "$OUTPUT_FILE" 2> /dev/null
touch "$OUTPUT_FILE"

# Loop through each S3 link and append the content to the output file
while read -r S3_LINK; do
    # Temporary file for downloaded content
    TEMP_FILE_COMPRESSED=$(mktemp)
    TEMP_FILE_UNCOMPRESSED=$(mktemp)

    # Download the ndjson file from S3
    aws s3 cp "$S3_LINK" "$TEMP_FILE_COMPRESSED"

    bunzip2 -dc "$TEMP_FILE_COMPRESSED" > "$TEMP_FILE_UNCOMPRESSED"

    # Append the content to the output file
    cat "$TEMP_FILE_UNCOMPRESSED" >> "$OUTPUT_FILE"

    # Clean up the temporary file
    rm "$TEMP_FILE_UNCOMPRESSED"
done < "$S3_LINKS_FILE"

rm "$S3_LINKS_FILE"
