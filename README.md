# WisePulse

<div align="center">

<img src="roles/loculus/files/images/wasap-logo.png" alt="WisePulse Logo" width="150"/>

### WISE Loculus with V-Pipe – Infrastructure for start-to-end viral wastewater analysis

![Status: Public Beta](https://img.shields.io/badge/status-public%20beta-blue)
![Platform: Linux](https://img.shields.io/badge/platform-linux-lightgrey)
![License: MIT](https://img.shields.io/badge/license-MIT-green)

</div>

---

## About

**WisePulse** is an end-to-end infrastructure for viral wastewater surveillance, combining [Loculus](https://loculus.org) for sequence data management with [LAPIS-SILO](https://github.com/GenSpectrum/LAPIS-SILO), a high-performance genomic database – here deployed for the first time to **S**hort-**R**ead / amplicon sequences. It enables real-time querying and visualization of viral genomic data from wastewater samples.

Built with [Ansible](https://www.ansible.com/) for infrastructure-as-code, enabling quick redeployment to other institutions and environments.

**Demo**: [Watch on YouTube](https://www.youtube.com/watch?v=kCUd-o1FbXg&t=420) (starts at 7:00)

#### Major Goals

- **Cluster to Browser**: Brought downstream exploratory analysis from exclusive, high barrier-to-entry cluster-based workflows to user-friendly, simple browser-based workflows

- **Days to Minutes**: Simplified exploratory analysis beyond the regular processing reports produced by the WISE Consortium from days waiting for requests served by few with data access to self-serving

- **Expert-only to Community Access**: Direct access to data allows for community access—not just those with data access and expert bioinformatics knowledge—to explore, catering for virologists and phylogeneticists

- **Isolated to Integrated**: First time wastewater data can be analyzed on the fly with the latest clinical variant definitions without manual work

### Current Features
- Loculus instance for easy data sharing and collaboration
- Multi-virus srSILO API for querying amplicon sequence data (SARS-CoV-2, RSV-A, RSV-B)

### Live Instances
- **Mutational Analysis Dashboards**: [genspectrum.org/swiss-wastewater](https://genspectrum.org/swiss-wastewater)
- **Swiss Wastewater Viral Alignment Database**: [db.wasap.genspectrum.org](https://db.wasap.genspectrum.org/)
- **API Swagger UI**: [COVID](https://lapis.wasap.genspectrum.org/covid/swagger-ui/index.html) | [RSV-A](https://lapis.wasap.genspectrum.org/rsva/swagger-ui/index.html) | [RSV-B](https://lapis.wasap.genspectrum.org/rsvb/swagger-ui/index.html)
- **External uptime check**: [UptimeRobot status page](https://stats.uptimerobot.com/EfH9UmhAYf)

#### Data Processing Pipeline (external)
Upstream data processing is handled separately:
- NGS processing via [V-pipe](https://github.com/cbg-ethz/V-pipe)
- Database ingestion with [sr2silo](https://github.com/cbg-ethz/sr2silo) from V-pipe alignment outputs

### Roadmap
- Expand to more viruses
- Raw data submission & V-pipe pipeline integration
- On-demand exploratory analysis tools

## Architecture

All components run on a single server and are managed by Ansible playbooks, which you run from your own machine (the control machine) over SSH.

```
                                  Internet
                                     │
        ┌────────────────────────────┴────────────────────────────┐
        │                  Nginx (SSL, routing)                   │  + firewalld
        └─────┬──────────────────────┬──────────────────────┬─────┘
              │ db.<domain>          │ lapis.<domain>/      │ rhydb.<domain>
              │ api.db.<domain>      │   <virus>/           │ silo.<domain>
              ▼                      ▼                      │
     ┌─────────────────┐   ┌─────── srSILO (Docker) ────────┼───────┐
     │     Loculus     │   │  ┌───────┐ ┌───────┐ ┌───────┐ │       │
     │  (Kubernetes)   │   │  │ LAPIS │ │ LAPIS │ │ LAPIS │ │       │
     │                 │   │  │ covid │ │ rsva  │ │ rsvb  │ │       │
     └────────┬────────┘   │  └───┬───┘ └───┬───┘ └───┬───┘ │       │
              │            │      ▼         ▼         ▼     ▼       │
              │            │  ┌──────────────────────────────────┐  │
              │            │  │ RhyDB (one instance), tables:    │  │
              │            │  │   data (covid), rsva, rsvb       │  │
              │            │  └─────────────────▲────────────────┘  │
              │            └────────────────────┼───────────────────┘
              │                                 │ new state
              │  fetch new data       ┌─────────┴──────────┐
              └──────────────────────►│   srsilo-updater   │
                                      │ (daily, systemd)   │
                                      └────────────────────┘
```

Monitoring (Prometheus/Grafana) runs on the host, scrapes the LAPIS instances and is only reachable from localhost (e.g. via SSH tunnel).

**Data flow:**

1. V-pipe produces BAM alignments.
2. [sr2silo](https://github.com/cbg-ethz/sr2silo) converts them and submits them to Loculus (which stores files in S3).
3. The srSILO pipeline ([`srsilo-updater/`](srsilo-updater/README.md)) runs daily (02:00, systemd timer), fetches and sorts new data from the Loculus API per virus, then, if any virus got new data, builds a fresh RhyDB (SILO) state from all viruses' sorted data. RhyDB picks up new states automatically; old ones are removed by a retention policy.
4. Each virus' LAPIS serves its table; nginx exposes it at `lapis.<domain>/<virus>/`, and RhyDB itself at `rhydb.<domain>` / `silo.<domain>`.

**Instances** (registry `srsilo_viruses` in `group_vars/srsilo/main.yml`, all in one docker-compose project `/opt/srsilo/docker-compose.yml`):

One RhyDB instance, `wise-rhydb` on port 8081, serves all viruses. Its tables are defined in `roles/srsilo/files/rhydb/schema.rhyql`. Each virus has its own LAPIS, which reads its table via `--rhydb.table`:

| Virus | Key | RhyDB table | LAPIS port | LAPIS container |
|-------|-----|-------------|-----------:|-----------------|
| SARS-CoV-2 | `covid` | `data` | 8083 | `wise-sarsCoV2-lapis` |
| RSV-A | `rsva` | `rsva` | 8084 | `wise-rsva-lapis` |
| RSV-B | `rsvb` | `rsvb` | 8085 | `wise-rsvb-lapis` |

Nginx routes `lapis.<domain>/<virus>/*` to these ports, stripping the `/<virus>` prefix (and setting `X-Forwarded-Prefix`). For backward compatibility, the root path `lapis.<domain>/` is proxied (not redirected, to keep CORS preflight working) to the COVID LAPIS. `rhydb.<domain>/`, `silo.<domain>/` and `silo.<domain>/covid/` serve the shared RhyDB instance. `silo.<domain>/rsva/` and `/rsvb/` still route to the old per-virus RSV SILO instances (ports 8082, 8086) during the transition (TODO: remove).

## Installation

### Prerequisites

- **Control machine:** Ansible 2.16+
- **Target server:** Ubuntu/Debian with Docker + Docker Compose, a Kubernetes cluster with kubectl configured (for Loculus), and a deploy user with SSH key access and sudo.
- **External services for Loculus** (not managed here): two PostgreSQL databases (Loculus + Keycloak) and an S3-compatible bucket. Credentials go in `group_vars/loculus/vault.yml`, bucket settings in `group_vars/loculus/main.yml`.

### Setup

```bash
git clone https://github.com/cbg-ethz/WisePulse.git
cd WisePulse

# Install Ansible collections/roles
ansible-galaxy collection install -r requirements.yml

# Point inventory.ini at your server
#   [wisepulse_server]
#   <target_host> ansible_user=<deploy_user>

# Vault password for group_vars/*/vault.yml (ask a maintainer; not in git)
echo '<vault password>' > .vault_pass && chmod 600 .vault_pass
```

Edit the configuration for your environment:

| File | Contents |
|------|----------|
| `group_vars/srsilo/main.yml` | Virus registry (organism, table, LAPIS port, `fetch_days` / `fetch_max_reads` / `chunk_size`), enabled viruses, memory limits, retention |
| `group_vars/loculus/main.yml`, `vault.yml` | Loculus app settings, S3 bucket; DB / Keycloak / S3 secrets |
| `group_vars/monitoring/main.yml`, `vault.yml` | LAPIS scrape targets, Prometheus retention; Grafana admin password |
| `roles/nginx/defaults/main.yml` | `nginx_domain_name`, backend ports, SSL certificate paths, enabled sites, `nginx_official_repo_channel` (`stable`/`mainline`) |

### Deploy

```bash
# Firewall (firewalld)
ansible-playbook playbooks/setup_firewall.yml -i inventory.ini --ask-become-pass

# Nginx reverse proxy (Let's Encrypt certs expected at the default certbot paths;
# add -e "generate_self_signed_certs=true" for test environments).
# nginx is installed from the official nginx.org apt repo. Hosts still on the distro
# package need a one-off manual switch first:
#   sudo apt install -o Dpkg::Options::=--force-confold nginx nginx-common-
ansible-playbook playbooks/setup_nginx.yml -i inventory.ini --ask-become-pass

# Loculus on Kubernetes
ansible-playbook playbooks/loculus/deploy-loculus.yml -i inventory.ini

# srSILO: user, directories, configs, Python venv, Rust tools, daily systemd timer
# (--tags timer: only the timer/service; --skip-tags build: skip the Rust build)
ansible-playbook playbooks/srsilo/setup.yml -i inventory.ini --ask-become-pass

# Monitoring: Prometheus, Grafana, Node Exporter, JSON Exporter
ansible-playbook playbooks/monitoring/full.yml -i inventory.ini --ask-become-pass
```

For a low-memory test server (~8 GB RAM), add `-e "@playbooks/srsilo/vars/test_vars.yml"` to the srSILO setup playbook; it writes a reduced-resource `pipeline.yml`.

Monitoring services listen on localhost only (Grafana `:3000`, Prometheus `:9090`, Node Exporter `:9100`, JSON Exporter `:7979`); use an SSH tunnel to reach them. Grafana login is `admin` with the password from the monitoring vault.

### Adding a new virus

1. Add its table (and references, prefixed `<virus>_`) to `roles/srsilo/files/rhydb/schema.rhyql`.
2. Add it to `srsilo_viruses` (organism, table, fetch settings) and `srsilo_enabled_viruses` in `group_vars/srsilo/main.yml`.
3. Only if it gets its own LAPIS: set `instance_name` and `lapis_port`, add `database_config.yaml` and `reference_genomes.json` under `roles/srsilo/files/viruses/<virus>/`, nginx routes in `roles/nginx/templates/sites-available/{lapis,silo}.j2` and the monitoring target in `group_vars/monitoring/main.yml`.
4. Re-run the srSILO setup playbook (and the nginx and monitoring playbooks for a new LAPIS).

## Operations

See **[docs/operations.md](docs/operations.md)** for logs, manual pipeline runs, recovery and troubleshooting.

The pipeline itself (phases, local development, config format) is documented in [`srsilo-updater/README.md`](srsilo-updater/README.md).
