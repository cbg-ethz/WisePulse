# Operations

Runbook for the production server. Replace `<virus>` with `covid`, `rsva` or `rsvb`. Container names are `<instance_name>-lapis` / `<instance_name>-silo` (e.g. `wise-sarsCoV2-lapis`, `wise-rsva-silo`; see the table in the [README](../README.md#architecture)).

## Server layout

```
/opt/srsilo/
├── pipeline.yml           # Pipeline config (written by Ansible, do not edit)
├── venv/                  # Python venv for srsilo-updater
├── tools/target/release/  # Rust binaries
└── <virus>/
    ├── config/            # docker-compose.yml, database/preprocessing configs, reference genomes
    ├── input/             # Downloaded .ndjson.zst files
    ├── output/            # SILO indexes (one timestamped dir per run; SILO serves the newest)
    ├── sorted_chunks/ tmp/ # Working dirs (reset each run)
    ├── logs/              # SILO logs
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

The pipeline exits early if the API has no new data since `.last_update`.

## APIs (LAPIS / SILO)

```bash
# Health
curl -s http://localhost:8083/sample/info   # covid
curl -s http://localhost:8084/sample/info   # rsva
curl -s http://localhost:8085/sample/info   # rsvb
curl -s https://lapis.wasap.genspectrum.org/covid/sample/info   # through nginx

# Containers and logs
docker ps -a | grep -E 'silo|lapis'
docker logs wise-sarsCoV2-lapis -f
tail -f /opt/srsilo/<virus>/logs/*.log

# Restart one virus
cd /opt/srsilo/<virus>/config && docker compose up -d
```

Don't edit `docker-compose.yml` on the server; it is overwritten by the setup playbook. Change `roles/srsilo/templates/docker-compose.yml.j2` instead.

**Containers down after a reboot / Docker upgrade:** the containers use `restart: unless-stopped`. Upgrading `containerd.io` / `docker-ce` restarts the Docker daemon, which marks the containers as stopped, so they do **not** come back after the following reboot. Check `apt list --upgradable` before upgrading, and afterwards bring each virus back up with `docker compose up -d` as above.

## Troubleshooting

**Preprocessing runs out of memory:** lower `chunk_size` and/or raise `docker_memory_limit` for that virus in `group_vars/srsilo/main.yml`, then re-run `playbooks/srsilo/setup.yml` so `pipeline.yml` is regenerated.

**Fetch downloads nothing even though there is new data:** `fetch_days` must cover the age of the newest data, and `fetch_max_reads` must exceed the read count of the busiest single sampling day.

**Disk full:** check `df -h /opt/srsilo` and `ls /opt/srsilo/<virus>/output/`. Old indexes are removed according to `srsilo_retention_days` / `srsilo_retention_min_keep`. `sorted_chunks/` and `tmp/` can be emptied safely when no pipeline run is active.

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
