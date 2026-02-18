# Configuration Reference

## srSILO Configuration

### Enable Viruses

In `roles/srsilo/defaults/main.yml`:

```yaml
srsilo_enabled_viruses:
  - covid
  - rsva
```

### Per-Virus Configuration

In `group_vars/srsilo/main.yml`:

```yaml
srsilo_virus_config:
  covid:
    fetch_days: 90           # Days of data to fetch
    fetch_max_reads: 172500000  # Max reads to fetch
    chunk_size: 1000000      # Chunk size for processing
    docker_memory_limit: 340g  # Memory limit for SILO container
  rsva:
    fetch_days: 90
    fetch_max_reads: 50000000
    chunk_size: 500000
    docker_memory_limit: 340g
```

### Global Settings

```yaml
srsilo_retention_days: 3        # Delete indexes older than N days
srsilo_retention_min_keep: 2    # Always keep at least M indexes
```

### Virus Registry

In `roles/srsilo/defaults/main.yml`:

```yaml
srsilo_viruses:
  covid:
    organism: covid
    instance_name: wise-sarsCoV2
    lapis_port: 8083
    silo_port: 8081
  rsva:
    organism: rsva
    instance_name: wise-rsva
    lapis_port: 8084
    silo_port: 8082
```

## Loculus Configuration

### Application Settings

In `group_vars/loculus/main.yml`:

- Application name, host, organisms
- URL configurations
- Feature flags
- S3 bucket configuration

### Secrets

In `group_vars/loculus/vault.yml` (encrypted):

- Database credentials
- Keycloak settings
- S3 credentials
- Service account passwords

### Defaults

In `roles/loculus/defaults/main.yml`:

| Variable | Default | Description |
|----------|---------|-------------|
| `loculus_temp_dir` | `/tmp/loculus` | Temp directory for deployment |
| `loculus_cleanup_temp_files` | `true` | Clean up temp files after deployment |
| `loculus_kubeconfig_path` | `~/.kube/config` | Path to kubeconfig |

## Monitoring Configuration

### LAPIS Instances

In `group_vars/monitoring/main.yml`:

```yaml
lapis_instances:
  - name: covid
    url: https://lapis.wasap.genspectrum.org
  - name: rsva
    url: https://lapis.wasap.genspectrum.org/rsva
```

### Metrics to Collect

```yaml
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

## Nginx Configuration

### Domain Settings

In `roles/nginx/defaults/main.yml`:

| Variable | Default | Description |
|----------|---------|-------------|
| `nginx_domain_name` | `wasap.genspectrum.org` | Base domain name |

### Port Mappings

| Variable | Default | Description |
|----------|---------|-------------|
| `nginx_lapis_covid_port` | `8083` | COVID LAPIS port |
| `nginx_lapis_rsva_port` | `8084` | RSVA LAPIS port |
| `nginx_silo_covid_port` | `8081` | COVID SILO port |
| `nginx_silo_rsva_port` | `8082` | RSVA SILO port |

### SSL Certificates

| Variable | Description |
|----------|-------------|
| `nginx_ssl_certificate_path` | Main domain SSL cert |
| `nginx_ssl_certificate_key_path` | Main domain SSL key |
| `nginx_lapis_ssl_certificate_path` | LAPIS subdomain SSL cert |
| `nginx_silo_ssl_certificate_path` | SILO subdomain SSL cert |
| `nginx_loculus_ssl_certificate_path` | Loculus subdomain SSL cert |

### Sites

```yaml
nginx_sites:
  - name: wasap-scout
    template: wasap-scout.j2
  - name: lapis
    template: lapis.j2
  - name: silo
    template: silo.j2

nginx_enabled_sites:
  - wasap-scout
  - lapis
  - silo
```

## Test Configuration

For resource-constrained environments, use test variables:

```bash
ansible-playbook playbooks/srsilo/update-pipeline.yml -i inventory.ini \
  -e "@playbooks/srsilo/vars/test_vars.yml"
```

Test vars typically include:

```yaml
srsilo_docker_memory_limit: 8g
srsilo_fetch_max_reads: 5000000
srsilo_chunk_size: 30000
```
