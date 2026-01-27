# Recovery Procedures

## srSILO API Recovery

### Restart API with Latest Index

```bash
# Replace <virus> with covid, rsva, etc.
cd /opt/srsilo/<virus>/config
docker compose down
docker compose up -d
```

### Restart with Specific Index

To use an older index, update the symlink that points to the active output:

```bash
cd /opt/srsilo/<virus>
# List available indexes
ls -la output/
# The compose file mounts 'output/' which should symlink to the active index
```

### List Available Indexes

```bash
# Each virus has its own output directory
ls -la /opt/srsilo/covid/output/
ls -la /opt/srsilo/rsva/output/
```

### Verify API Health

```bash
curl http://localhost:8083/sample/info  # COVID
curl http://localhost:8084/sample/info  # RSV-A
```

## Clean Failed Run Artifacts

If a pipeline run failed mid-way (replace `<virus>` with covid, rsva, etc.):

```bash
# Remove temporary processing files for a specific virus
sudo rm -rf /opt/srsilo/<virus>/sorted_chunks/*
sudo rm -rf /opt/srsilo/<virus>/tmp/*

# Remove in-progress marker for that virus
sudo rm /opt/srsilo/<virus>/output/.preprocessing_in_progress
```

## Manual Pipeline Run

After cleanup, re-run the pipeline:

```bash
ansible-playbook playbooks/srsilo/update-pipeline.yml -i inventory.ini --become --ask-become-pass
```

## Loculus Recovery

### Check Pod Status

```bash
kubectl get pods -A | grep loculus
```

### Restart Pods

```bash
kubectl rollout restart deployment/<deployment-name> -n <namespace>
```

### View Pod Logs

```bash
kubectl logs <pod-name> -n <namespace> -f
```

## Monitoring Recovery

### Restart Services

```bash
sudo systemctl restart prometheus
sudo systemctl restart grafana-server
```

### Check Service Status

```bash
systemctl status prometheus
systemctl status grafana-server
journalctl -u prometheus -n 50
journalctl -u grafana-server -n 50
```

## Quick Reference

### All Manual Restart Commands

```bash
# SILO/LAPIS (per-virus; run from the appropriate config directory)
cd /opt/srsilo/covid/config && docker compose up -d   # COVID
cd /opt/srsilo/rsva/config && docker compose up -d    # RSV-A

# V-Pipe Scout (if deployed)
cd /opt/v-pipe-scout && docker compose up -d

# Check status
kubectl get pods -A  # Loculus
docker ps -a         # All containers
```

### Full System Check

```bash
# Docker containers
docker ps -a

# Kubernetes
kubectl get pods -A

# Systemd services
systemctl status srsilo-update.timer
systemctl status prometheus
systemctl status grafana-server

# API endpoints
curl -s http://localhost:8083/sample/info
curl -s http://localhost:8084/sample/info
```

## See Also

- [Troubleshooting Guide](troubleshooting.md)
- [Logging Guide](logging.md)
- [2025-11-20 Service Outage](../incidents/2025-11-20-service-outage.md) - Example incident and lessons learned
