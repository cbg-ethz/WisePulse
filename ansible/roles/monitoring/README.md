# WisePulse Monitoring Stack

This monitoring setup provides basic system metrics tracking for the WisePulse project.

## Components

- **Prometheus**: Metrics collection and storage
- **Grafana**: Metrics visualization and dashboards
- **Node Exporter**: System-level metrics (CPU, RAM, disk, network, uptime)

## Architecture

- All services run on localhost
- Services listen on 127.0.0.1 (not exposed externally)
- Requires x86_64 architecture (AMD64)
- Ubuntu/Debian compatible

## Dashboards

The setup automatically downloads the **Node Exporter Full** dashboard (ID: 1860) from Grafana.com during deployment. This provides comprehensive system monitoring including:

- CPU usage and load
- RAM and SWAP usage
- Disk I/O and space
- Network traffic
- System uptime
- Process-level metrics

## Ports

- Prometheus: 9090 (localhost only)
- Grafana: 3000 (localhost only)
- Node Exporter: 9100 (localhost only)

## Usage

Deploy the full monitoring stack:

```bash
cd ansible
ansible-playbook playbooks/monitoring/full.yml
```

Deploy only core components (Prometheus + Grafana):

```bash
ansible-playbook playbooks/monitoring/core.yml
```

Deploy only exporters (Node Exporter):

```bash
ansible-playbook playbooks/monitoring/exporters.yml
```

## Access

After deployment:

- Grafana UI: http://localhost:3000
- Default credentials: admin / (see vault for password)
- Prometheus UI: http://localhost:9090

## Notes

- Prometheus and Node Exporter: x86_64 only (hardcoded linux-amd64 downloads)
- Grafana: x86_64 / ARM64 (installed via apt, no architecture limitation)
- Dashboard is auto-downloaded from Grafana.com (not stored in git)
- All binaries are downloaded from official GitHub releases
- Compatible with Ubuntu and Debian
