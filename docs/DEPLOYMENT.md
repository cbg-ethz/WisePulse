# WisePulse Deployment Guide

## Prerequisites

- Ansible 2.16+, Docker, Docker Compose
- Kubernetes (for Loculus), kubectl configured
- Linux (Ubuntu 20.04+, Debian 11+)

## srSILO Pipeline

### Setup

```bash
# Clone and configure
git clone https://github.com/cbg-ethz/WisePulse.git
cd WisePulse
cp inventory.ini.example inventory.ini
vim group_vars/srsilo/main.yml

# One-time setup (creates user, builds tools, deploys configs)
ansible-playbook playbooks/srsilo/setup.yml -i inventory.ini --become  --ask-become-pass
```

### Operations

```bash
# Manual run (7-phase pipeline)
ansible-playbook playbooks/srsilo/update-pipeline.yml -i inventory.ini  --become --ask-become-pass 

# Test mode (8GB RAM, 5M reads, 30k chunks)
ansible-playbook playbooks/srsilo/update-pipeline.yml -i inventory.ini \
  --become --ask-become-pass \
  -e "@playbooks/srsilo/vars/test_vars.yml"

# Enable daily automation (2 AM systemd timer)
ansible-playbook playbooks/srsilo/setup-timer.yml -i inventory.ini --become

# View logs
journalctl -t srsilo-update -n 100 --no-pager
journalctl -u srsilo-update.service -f  # Follow timer runs
```

### Recovery

```bash
# Check API status
docker ps | grep silo
curl http://localhost:8083/sample/info

# Restart API with latest index
cd /opt/srsilo/tools
docker-compose down
docker-compose up -d

# Restart with specific index
cd /opt/srsilo/tools
docker-compose down
# Edit docker-compose.yml, change DATA_DIR to /opt/srsilo/output/<timestamp>
docker-compose up -d

# Clean failed run artifacts
sudo rm -rf /opt/srsilo/sorted_chunks/*
sudo rm -rf /opt/srsilo/tmp/*
sudo rm /opt/srsilo/output/.preprocessing_in_progress
```

### Logging

```bash
# Timer status
systemctl status srsilo-update.timer
systemctl list-timers srsilo-update.timer

# Pipeline logs
journalctl -t srsilo-split -n 50        # Chunk splitting
journalctl -t srsilo-merge -n 50        # Merging
journalctl -t srsilo-preprocess -n 50   # SILO preprocessing

```

## Loculus

### Deploy

```bash
# Configure
vim group_vars/loculus/main.yml
vim group_vars/loculus/vault.yml  # Encrypted secrets

# Deploy to Kubernetes
ansible-playbook playbooks/loculus/deploy-loculus.yml -i inventory.ini
```


## Monitoring Stack

See [MONITORING.md](MONITORING.md) for deployment and access instructions.

## Common Issues

**srSILO OOM during preprocessing**:
- Lower `srsilo_chunk_size` in group_vars
- Increase `srsilo_docker_memory_limit` if RAM available
- Use test_vars.yml for resource-constrained environments

**API won't start**:
- Check Docker logs: `docker logs silo-lapis`
- Verify index exists: `ls -la /opt/srsilo/output/<timestamp>/`
- Check permissions: `ls -ld /opt/srsilo/output`

**Timer not running**:
- Check status: `systemctl status srsilo-update.timer`
- View schedule: `systemctl list-timers`
- Check service logs: `journalctl -u srsilo-update.service -n 100`
