# Monitoring Deployment

Deploys Prometheus, Grafana, Node Exporter, and JSON Exporter for system and LAPIS API monitoring.

## Components

| Component | Role | Purpose |
|-----------|------|---------|
| **Prometheus** | `prometheus.prometheus` | Metrics collection and storage |
| **Grafana** | `grafana.grafana` | Visualization and dashboards |
| **Node Exporter** | `prometheus.prometheus.node_exporter` | Host resource metrics (CPU, RAM, Disk) |
| **JSON Exporter** | `json_exporter` | LAPIS application metrics (via Actuator) |

**External Uptime**: [UptimeRobot Status Page](https://stats.uptimerobot.com/EfH9UmhAYf) (independent check).

## Prerequisites

```bash
ansible-galaxy collection install -r requirements.yml
```

## Deploy

```bash
# Deploys full stack using official collections
ansible-playbook playbooks/monitoring/full.yml -i inventory.ini --ask-become-pass
```

## Access

Services are bound to `127.0.0.1` (localhost) for security. Use SSH tunneling to access.

| Service | URL | Credentials |
|---------|-----|-------------|
| **Grafana** | http://localhost:3000 | `admin` / (see `vault.yml`) |
| **Prometheus** | http://localhost:9090 | None |
| **Node Exporter** | http://localhost:9100/metrics | None |

## Configuration

### LAPIS Targets

Configure in `group_vars/monitoring/main.yml`:

```yaml
lapis_instances:
  - name: covid
    url: https://lapis.wasap.genspectrum.org
  - name: rsva
    url: https://lapis.wasap.genspectrum.org/rsva

lapis_metrics:
  - name: info_count
    path: $.info.count
  - name: response_time
    path: $.responseTime
```

### Grafana

| Variable | Description |
|----------|-------------|
| `grafana_admin_password` | Admin password (use vault) |

### Dashboards

| Dashboard | Purpose |
|-----------|---------|
| **Lapis COVID** | COVID LAPIS API health & metrics |
| **Lapis RSV-A** | RSV-A LAPIS API health & metrics |
| **Node Exporter Full** | System resource usage (CPU, RAM, Disk) |

### Retention

Prometheus data is retained for **30 days**.

## Ports

| Service | Port |
|---------|------|
| Prometheus | 9090 |
| Grafana | 3000 |
| Node Exporter | 9100 |
| JSON Exporter | 7979 |

## Alerting

Currently disabled (no Alertmanager configured).

## See Also

- [Logging Guide](../operations/logging.md)
- [Architecture Overview](../architecture/overview.md)
