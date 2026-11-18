# Operations

Runbook for the production server. Replace `<virus>` with `covid`, `rsva` or `rsvb`. One RhyDB container (`wise-rhydb`) serves all viruses, one table each; each virus has a LAPIS container `<instance_name>-lapis` (e.g. `wise-sarsCoV2-lapis`, `wise-rsva-lapis`; see the table in the [README](../README.md#architecture)).

## Server layout

```
/opt/srsilo/
├── pipeline.yml           # Pipeline config (written by Ansible, do not edit)
├── docker-compose.yml     # RhyDB + one LAPIS per virus (written by Ansible, do not edit)
├── venv/                  # Python venv for srsilo-updater
├── tools/target/release/  # Rust binaries
├── rhydb/
│   ├── schema.rhyql       # Schema of the shared instance (one table per virus)
│   ├── output/            # RhyDB states (one timestamped dir per run; RhyDB serves the newest)
│   ├── tmp/               # Working dir of the state build
│   └── logs/              # RhyDB logs
└── <virus>/
    ├── config/            # LAPIS database_config.yaml, reference genomes
    ├── input/             # Downloaded .ndjson.zst files
    ├── sorted.ndjson.zst  # Latest sorted data (read by every state build)
    ├── sorted_chunks/ tmp/ # Working dirs (reset each run)
    └── .last_update       # Timestamp of last successful run
```

## srSILO pipeline

```bash
# Timer status / next run (daily at 02:00)
systemctl list-timers srsilo-update.timer

# Trigger a run now (same as the timer)
sudo systemctl start srsilo-update.service

# Run a single virus by hand
sudo -u srsilo /opt/srsilo/venv/bin/python -m pipeline --config /opt/srsilo/pipeline.yml --virus <virus>

# Pipeline logs
journalctl -u srsilo-update -n 1000
journalctl -u srsilo-update -f
```

The pipeline skips a virus if the API has no new data since its `.last_update`. Otherwise it fetches and sorts that virus. If at least one virus got new data, a fresh RhyDB state is built from `schema.rhyql` and every virus' `sorted.ndjson.zst` and saved as a new state. A changed `schema.rhyql` therefore takes effect with the next run that has new data.

## APIs (LAPIS / SILO)

```bash
# Health
curl -s http://localhost:8083/sample/info   # covid
curl -s http://localhost:8084/sample/info   # rsva
curl -s http://localhost:8085/sample/info   # rsvb
curl -s https://lapis.wasap.genspectrum.org/covid/sample/info   # through nginx

# Containers and logs
docker ps -a | grep -E 'rhydb|lapis'
docker logs wise-sarsCoV2-lapis -f
tail -f /opt/srsilo/rhydb/logs/*.log

# (Re)start everything, or one service (rhydb, lapis-<virus>)
cd /opt/srsilo && docker compose up -d [lapis-<virus>]
```

Don't edit `docker-compose.yml` on the server; it is overwritten by the setup playbook. Change `roles/srsilo/templates/docker-compose.yml.j2` instead.

**Containers down after a reboot / Docker upgrade:** the containers use `restart: unless-stopped`. Upgrading `containerd.io` / `docker-ce` restarts the Docker daemon, which marks the containers as stopped, so they do **not** come back after the following reboot. Check `apt list --upgradable` before upgrading, and afterwards bring them back up with `docker compose up -d` as above.

## Troubleshooting

**Pipeline runs out of memory:** for sorting, lower `chunk_size` for that virus; for the state build (which holds the whole shared state), raise `srsilo_pipeline_memory_max`. Both are in `group_vars/srsilo/main.yml`; re-run `playbooks/srsilo/setup.yml` (and `setup-timer.yml` for the memory cap) afterwards. RhyDB serving runs out of memory: raise `srsilo_rhydb_memory_limit`.

**Fetch downloads nothing even though there is new data:** `fetch_days` must cover the age of the newest data, and `fetch_max_reads` must exceed the read count of the busiest single sampling day.

**Disk full:** check `df -h /opt/srsilo` and `ls /opt/srsilo/rhydb/output/`. Old states are removed according to `srsilo_retention_days` / `srsilo_retention_min_keep`. `sorted_chunks/` and `tmp/` can be emptied safely when no pipeline run is active.

## Loculus (Kubernetes)

```bash
kubectl get pods -A
kubectl logs <pod> -n <namespace> -f
kubectl rollout restart deployment/<deployment> -n <namespace>
```

## Monitoring

```bash
systemctl status prometheus grafana-server
journalctl -u prometheus -n 50
journalctl -u grafana-server -n 50
sudo systemctl restart prometheus grafana-server

# Grafana from your laptop
ssh -L 3000:localhost:3000 <deploy_user>@<target_host>   # then open http://localhost:3000
```

Prometheus keeps 30 days of data. No alerting is configured; external uptime is checked by [UptimeRobot](https://stats.uptimerobot.com/EfH9UmhAYf).
