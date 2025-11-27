# WisePulse Monitoring Stack

This monitoring setup provides comprehensive metrics tracking for the WisePulse project, including multi-instance Lapis API monitoring.

## Overview

The monitoring stack uses **official Ansible collections** for better maintainability:

- **[prometheus.prometheus](https://github.com/prometheus-community/ansible)** - Prometheus and Node Exporter
- **[grafana.grafana](https://github.com/grafana/grafana-ansible-collection)** - Grafana

## Components

| Component | Role | Description |
|-----------|------|-------------|
| Prometheus | `prometheus.prometheus.prometheus` | Metrics collection and storage |
| Node Exporter | `prometheus.prometheus.node_exporter` | System-level metrics |
| JSON Exporter | `json_exporter` (custom) | Converts Lapis JSON metrics to Prometheus format |
| Grafana | `grafana.grafana.grafana` | Metrics visualization and dashboards |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Grafana (:3000)                          │
│                    Dashboards & Visualization                   │
└─────────────────────────────────────────────────────────────────┘
                              ▲
                              │ Query
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Prometheus (:9090)                         │
│                   Metrics Storage & Scraping                    │
└─────────────────────────────────────────────────────────────────┘
        ▲                     ▲                     ▲
        │ Scrape              │ Scrape              │ Scrape
        │                     │                     │
┌───────┴───────┐   ┌─────────┴─────────┐   ┌──────┴──────┐
│ Node Exporter │   │   JSON Exporter   │   │   Future    │
│    (:9100)    │   │     (:7979)       │   │  Exporters  │
│ System Metrics│   │    ▼ Probe        │   │             │
└───────────────┘   └─────────┬─────────┘   └─────────────┘
                              │
                              ▼
            ┌─────────────────────────────────────┐
            │           Lapis Instances           │
            ├─────────────────────────────────────┤
            │ • wasap (lapis.wasap.genspectrum)   │
            │ • cov-spectrum (future)             │
            │ • staging (future)                  │
            └─────────────────────────────────────┘
```

## Lapis Multi-Instance Monitoring

### Adding New Lapis Instances

Simply add entries to `lapis_instances` in `group_vars/monitoring/main.yml`:

```yaml
lapis_instances:
  - name: wasap
    url: https://lapis.wasap.genspectrum.org
  
  - name: cov-spectrum
    url: https://lapis.cov-spectrum.org
  
  - name: staging
    url: https://lapis-staging.genspectrum.org
    # actuator_path: /management  # if different from /actuator
```

Each instance automatically gets all metrics defined in `lapis_metrics`. The generated scrape jobs include:
- **Job naming**: `lapis_{metric}_{instance}` (e.g., `lapis_cache_wasap`)
- **Labels**: `lapis_instance: wasap` for filtering in Grafana

### Metrics Collected

| Category | Metrics |
|----------|---------|
| Health | `lapis_health__value{status="UP\|DOWN"}` |
| Cache | `lapis_cache_size` |
| HTTP | `lapis_http_requests_value`, `lapis_http_requests_active` |
| JVM Memory | `lapis_jvm_memory_used_bytes`, `lapis_jvm_memory_max_bytes`, `lapis_jvm_memory_committed_bytes` |
| JVM GC | `lapis_jvm_gc_pause_value{statistic="COUNT\|TOTAL_TIME"}`, `lapis_jvm_gc_overhead` |
| JVM Threads | `lapis_jvm_threads_live`, `lapis_jvm_threads_peak`, `lapis_jvm_threads_daemon` |
| Process | `lapis_process_uptime_seconds`, `lapis_process_cpu_usage`, `lapis_system_cpu_usage` |
| Disk | `lapis_disk_free_bytes`, `lapis_disk_total_bytes` |
| Executor | `lapis_executor_active`, `lapis_executor_pool_size`, `lapis_executor_queue_remaining` |

### Customizing Metrics Per Instance

To collect different metrics for specific instances, you can override `lapis_metrics` in host_vars or define instance-specific metric lists.

### Custom Dashboard

A comprehensive Lapis monitoring dashboard is included at:
`roles/monitoring/files/dashboards/lapis-dashboard.json`

Use the `lapis_instance` label to filter by instance in Grafana.

## Installation

### Prerequisites

Install required Ansible collections:

```bash
ansible-galaxy collection install -r requirements.yml
```

### Deployment

Deploy the full monitoring stack:

```bash
ansible-playbook playbooks/monitoring/full.yml -e "grafana_admin_password=your_secure_password"
```

## Ports

| Service | Port | Binding |
|---------|------|---------|
| Prometheus | 9090 | 0.0.0.0 |
| Grafana | 3000 | 0.0.0.0 |
| Node Exporter | 9100 | 0.0.0.0 |
| JSON Exporter | 7979 | 0.0.0.0 |

## Access

After deployment:

- **Grafana UI**: http://localhost:3000
- **Prometheus UI**: http://localhost:9090
- **Credentials**: admin / (password from `-e grafana_admin_password=...`)

## Dashboards

Auto-provisioned dashboards:

1. **Node Exporter Full** (ID: 1860) - System monitoring
2. **JVM Micrometer** (ID: 4701) - JVM metrics reference  
3. **Lapis API Monitoring** - Custom dashboard for Lapis

## Extending

To add new services to monitor:

1. Add scrape configs to `group_vars/monitoring/main.yml` under `prometheus_scrape_configs`
2. For JSON APIs, add modules to `roles/json_exporter/templates/config.yml.j2`
3. Create Grafana dashboards in `roles/monitoring/files/dashboards/`

## Requirements

- Ubuntu/Debian (apt-based)
- ARM64 (aarch64) or AMD64 (x86_64)
- Ansible 2.14+
- Collections: `prometheus.prometheus`, `grafana.grafana`
