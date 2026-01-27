# Quick Start

After completing [installation](installation.md), use these commands to get started.

## srSILO Pipeline

### Run Pipeline

```bash
# Update all enabled viruses (production)
ansible-playbook playbooks/srsilo/update-all-viruses.yml -i inventory.ini --become --ask-become-pass

# Or update a single virus
ansible-playbook playbooks/srsilo/update-pipeline.yml -i inventory.ini -e "srsilo_virus=rsva"

# Test with reduced resources (8GB RAM)
ansible-playbook playbooks/srsilo/update-all-viruses.yml -i inventory.ini \
  -e "@playbooks/srsilo/vars/test_vars.yml"
```

### Setup Automation

```bash
# Setup daily automated runs at 2 AM
ansible-playbook playbooks/srsilo/setup-timer.yml -i inventory.ini
```

### Check API Status

```bash
curl http://localhost:8083/sample/info  # COVID
curl http://localhost:8084/sample/info  # RSV-A
```

**API Swagger UI:**

- COVID: `http://localhost:8083/swagger-ui/index.html`
- RSV-A: `http://localhost:8084/swagger-ui/index.html`

## Loculus

```bash
# Deploy W-ASAP Loculus to Kubernetes
ansible-playbook playbooks/loculus/deploy-loculus.yml -i inventory.ini
```

## Monitoring Stack

```bash
# Deploy Prometheus + Grafana
ansible-playbook playbooks/monitoring/full.yml -i inventory.ini --ask-become-pass
```

## Nginx Reverse Proxy

```bash
# Deploy Nginx with SSL termination
ansible-playbook playbooks/setup_nginx.yml -i inventory.ini --ask-become-pass
```

## Next Steps

- [Architecture Overview](../architecture/overview.md) – Understand the system design
- [Operations Guide](../operations/logging.md) – Monitor and troubleshoot
