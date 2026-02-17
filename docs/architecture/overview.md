# Architecture Overview

WisePulse consists of four main components, all managed by Ansible playbooks.

## Components

```
                         ┌─────────────────┐
                         │    Internet     │
                         └────────┬────────┘
                                  │
                    ┌─────────────┴─────────────┐
                    │     Nginx Reverse Proxy   │
                    │  (SSL termination, routing)│
                    └─────────────┬─────────────┘
                                  │
        ┌─────────────────────────┼─────────────────────────┐
        │                         │                         │
        ▼                         ▼                         ▼
┌───────────────┐       ┌─────────────────┐       ┌─────────────────┐
│    Loculus    │       │  srSILO/LAPIS   │       │   Monitoring    │
│  (Kubernetes) │       │    (Docker)     │       │ (Prometheus/    │
│               │       │                 │       │    Grafana)     │
│  Sequence     │       │  Multi-virus    │       │                 │
│  data mgmt    │       │  genomic APIs   │       │  Metrics &      │
│  & sharing    │       │  (COVID, RSV-A) │       │  dashboards     │
└───────────────┘       └─────────────────┘       └─────────────────┘
     :8080                  :8081-8084                 :3000, :9090
```

## Data flow

```
┌──────────────┐    sr2silo     ┌──────────────┐    srSILO pipeline     ┌──────────────┐
│   BAM files  │ ─────────────▶ │   Loculus    │ ─────────────────────▶ │ srSILO /     │
│              │                │    (S3)      │                        │ LAPIS        │
│              │                │ Sequence mgmt│                        │              │
│              │                │ & storage    │                        │              │
└──────────────┘                └──────────────┘                        └──────────────┘

```

1. BAM files are created by V-pipe.
2. [sr2silo](https://github.com/cbg-ethz/sr2silo) is used to convert the files and upload them to Loculus.
3. Loculus stores the data (using S3 under the hood to do so).
4. the [srSILO pipeline](srsilo-pipeline.md) fetches the data from Loculus and ingest them into the short-read SILO instance (srSILO).
5. Amplicon sequences are available in SILO/LAPIS to be queried.

## srSILO Pipeline

Multi-virus genomic data pipeline **fully managed by Ansible** with:

- **Multi-virus support** (SARS-CoV-2, RSV-A; RSV-B and Influenza planned)
- **Low downtime** (API managed automatically)
- **Self-healing** (automatic rollback on failures)
- **Smart execution** (exits early if no new data)
- **Retention policy** (automatic cleanup of old indexes)

See [srSILO Pipeline Architecture](srsilo-pipeline.md) for detailed documentation.

**Playbooks:**

| Playbook | Purpose |
|----------|---------|
| `playbooks/srsilo/setup.yml` | One-time server setup |
| `playbooks/srsilo/update-all-viruses.yml` | Update all enabled viruses (production) |
| `playbooks/srsilo/update-pipeline.yml` | Update single virus (debug/testing) |
| `playbooks/srsilo/setup-timer.yml` | Configure daily automated runs |

## Loculus

Pathogen sequence sharing platform deployed to Kubernetes.

**Playbook:** `playbooks/loculus/deploy-loculus.yml`

See [Loculus Deployment](../deployment/loculus.md) for configuration details.

## Monitoring Stack

Prometheus + Grafana for observability.

| Component | Role |
|-----------|------|
| **Prometheus** | Metrics collection and storage |
| **Grafana** | Visualization and dashboards |
| **Node Exporter** | Host resource metrics (CPU, RAM, Disk) |
| **JSON Exporter** | LAPIS application metrics |

**Playbook:** `playbooks/monitoring/full.yml`

See [Monitoring Deployment](../deployment/monitoring.md) for details.

## Nginx Reverse Proxy

SSL termination and path-based routing for all services.

| URL Pattern | Backend |
|-------------|---------|
| `lapis.{domain}/covid/*` | LAPIS COVID (8083) |
| `lapis.{domain}/rsva/*` | LAPIS RSVA (8084) |
| `silo.{domain}/covid/*` | SILO COVID (8081) |
| `silo.{domain}/rsva/*` | SILO RSVA (8082) |

**Playbook:** `playbooks/setup_nginx.yml`

See [Nginx Deployment](../deployment/nginx.md) for configuration details.
