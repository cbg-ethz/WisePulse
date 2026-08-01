# Logging

## Common Commands

### Pipeline Status

```bash
sudo journalctl -u srsilo-update -n 1000
```

### Errors

```bash
journalctl -p err --since today | grep srsilo
```

### API Logs

```bash
# Container names follow pattern: <instance_name>-lapis / <instance_name>-silo
# COVID: wise-sarsCoV2-lapis, wise-sarsCoV2-silo
# RSV-A: wise-rsva-lapis, wise-rsva-silo
docker logs wise-sarsCoV2-lapis -f
docker logs wise-sarsCoV2-silo -f

# SILO file logs (per-virus)
tail -f /opt/srsilo/covid/logs/*.log
tail -f /opt/srsilo/rsva/logs/*.log
```

### Timer

```bash
systemctl status srsilo-update.timer
systemctl list-timers srsilo-update.timer
```

### Monitoring Services

```bash
journalctl -u prometheus -n 50
journalctl -u grafana-server -n 50
```

### Kubernetes (Loculus)

```bash
kubectl logs <pod-name> -n <namespace> -f
```

## Log Retention

### Systemd Journal

System default (~1-2 months):

```bash
sudo journalctl --vacuum-time=2weeks  # Clean old logs
```

### Docker Logs

Manual management at `/opt/srsilo/<virus>/logs/`:

```bash
# Archive old logs (per-virus)
cd /opt/srsilo/covid/logs
for log in *.log; do mv "$log" "$log.$(date +%Y%m%d)" && gzip "$log.$(date +%Y%m%d)"; done
find . -name "*.gz" -mtime +30 -delete
```
