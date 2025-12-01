# Monitoring Role

Deploys Prometheus, Grafana, Node Exporter, and JSON Exporter for Lapis API monitoring.

## Dependencies

Uses official Ansible collections (install via `ansible-galaxy collection install -r requirements.yml`):
- `prometheus.prometheus`
- `grafana.grafana`

## Configuration

**Required** (`group_vars/monitoring/main.yml`):
- `lapis_instances`: List of Lapis instances to monitor
- `lapis_metrics`: Metrics to collect from each instance
- `grafana_admin_password`: Grafana admin password (use vault)

**Adding Lapis instances**:
```yaml
lapis_instances:
  - name: covid
    url: https://lapis.wasap.genspectrum.org
```

## Usage

```bash
ansible-playbook playbooks/monitoring/full.yml
```

## Ports

| Service | Port |
|---------|------|
| Prometheus | 9090 |
| Grafana | 3000 |
| Node Exporter | 9100 |
| JSON Exporter | 7979 |
