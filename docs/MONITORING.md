# WisePulse Monitoring

## Overview

**Components**: Prometheus, Grafana, Node Exporter, JSON Exporter (for LAPIS).
**External Uptime**: [UptimeRobot Status Page](https://stats.uptimerobot.com/EfH9UmhAYf) (independent check).

---

## Deployment

**Canonical Command**:
```bash
# Deploys full stack using official collections
ansible-playbook playbooks/monitoring/full.yml -i inventory.ini --become
```

**Prerequisites**:
```bash
ansible-galaxy collection install -r requirements.yml
```

---

## Access & Ports

Services are bound to `127.0.0.1` (localhost) for security. Use SSH tunneling to access.

| Service | URL | Credentials |
|---------|-----|-------------|
| **Grafana** | http://localhost:3000 | `admin` / (see `vault.yml`) |
| **Prometheus** | http://localhost:9090 | None |
| **Node Exporter** | http://localhost:9100/metrics | None |

---

## Configuration

**Targets**:
- **LAPIS**: Configured in `group_vars/monitoring/main.yml` under `lapis_instances`.
- **Host**: Node Exporter runs on all monitoring hosts.

**Dashboards**:
- **Lapis Covid**: Custom dashboard for API health & metrics.
- **Node Exporter Full**: System resource usage (CPU, RAM, Disk).

**Retention**:
- Prometheus data is retained for **30 days**.

---

## Operations

**Logs**:
See [LOGGING.md](LOGGING.md) for detailed `journalctl` commands.

**Alerting**:
Currently disabled (no Alertmanager configured).