To build the Rust helper scripts, run `cargo build --release`.

To process a set of `.ndjson.zst` files, put these in the directory `silo_input`. Then run `make`. 
This will generate SILO Indexes which can be readily used to run a LAPIS/SILO API.

To start the API you can run `LAPIS_PORT=80 docker compose up`.
Note that you can replace the `LAPIS_PORT` with another port that the api should listen on.

A swagger UI to the API can then be accessed at:
`http://localhost:80/swagger-ui/index.html`

Prerequisites:
- installed cargo
- installed Docker Compose
- platform: Linux (w.r.t. pre-processing scripts invoked by `make`)