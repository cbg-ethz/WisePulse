# WisePulse Logging Architecture

## Overview

**Dual-layer strategy**: systemd journal for automation, Docker logs for services.

**Key principles**: Structured tags, phase-based organization, parsimonious output.

---

## Log Locations

| Component | Location | Access Command |
|-----------|----------|----------------|
| **Pipeline automation** | systemd journal | `journalctl -t srsilo-*` |
| **Timer service** | systemd journal | `journalctl -u srsilo-update.service` |
| **LAPIS-SILO API** | `/opt/srsilo/tools/logs/` | `docker logs tools-lapisOpen-1` or `docker logs tools-silo-1` |
| **Monitoring (Prometheus, Grafana, Node Exporter)** | systemd journal | `journalctl -u <service-name>` |
| **Loculus (K8s)** | K8s cluster | `kubectl logs <pod>` |

---

## Pipeline Tags

| Tag | Purpose |
|-----|---------|
| `srsilo-pipeline` | Overall pipeline status |
| `srsilo-phase2` to `srsilo-phase6` | Phase markers (check, cleanup, fetch, process) |
| `srsilo-check-data`, `srsilo-fetch`, `srsilo-split`, `srsilo-merge` | Component operations |

**Example**:
```bash
$ journalctl -t srsilo-phase4 -n 3 --no-pager # Last 3 entries
```


---

## Common Commands

```bash
# Pipeline status
journalctl -t srsilo-pipeline -f
journalctl -u srsilo-update.service -n 100

# Specific phases
journalctl -t srsilo-phase4 -n 50
journalctl -t srsilo-fetch --since "1 hour ago"

# Errors (all srsilo logs)
journalctl -p err --since today | grep srsilo

# API logs
docker logs tools-lapisOpen-1 -f
docker logs tools-silo-1 -f
tail -f /opt/srsilo/tools/logs/*.log

# Timer
systemctl status srsilo-update.timer
systemctl list-timers srsilo-update.timer

# Monitoring services
journalctl -u prometheus -n 50
journalctl -u grafana-server -n 50

# Kubernetes (Loculus)
kubectl logs <pod-name> -n <namespace> -f
```

---

## Log Retention

**Systemd journal**: System default (~1-2 months)
```bash
sudo journalctl --vacuum-time=2weeks  # Clean old logs
```

**Docker logs** (`/opt/srsilo/tools/logs/`): Manual management
```bash
# Archive old logs
cd /opt/srsilo/tools/logs
for log in *.log; do mv "$log" "$log.$(date +%Y%m%d)" && gzip "$log.$(date +%Y%m%d)"; done
find . -name "*.gz" -mtime +30 -delete
```

---

## Best Practices

1. **Consistent tagging**: Use `srsilo-phase<N>` for phases, `srsilo-<component>` for operations
2. **Severity levels**: `info` for normal ops, `err` for failures
3. **Structured output**: Include metrics in messages (e.g., "COMPLETE (files: 10, size: 5MB)")
4. **Separation**: Systemd for orchestration, Docker for service runtime
