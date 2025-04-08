To generate the input data run `./generate_silo_input.bash`.

This downloads all data from the loculus instance wise-seqs.loculus.org,
looks at the short-read s3Link, downloads all s3 buckets where the file
ends with .ndjson.bz2 and merges them into a single .ndjson file.

To build the indexes and start the api, run `LAPIS_PORT=8080 docker compose up` where
you can replace the `LAPIS_PORT`.

This builds the SILO indexes (service `siloPreprocessing`),
starts the silo api (service `silo`) and the LAPIS api (service `lapis`).

The GUI to the API can be accessed at:
`http://localhost:8080/swagger-ui/index.html`

Prerequisites:
- installed Docker Compose
- install jq (on Ubuntu: `sudo apt-get install jq`)
- install aws cli (on Ubuntu: `sudo apt install awscli`)
- authenticate aws cli for S3 bucket containing files (`aws configure`)
