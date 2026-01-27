# Troubleshooting

## Common Issues

### srSILO OOM During Preprocessing

**Symptoms:** Pipeline fails during SILO preprocessing phase with out-of-memory errors.

**Solutions:**

1. Lower `srsilo_chunk_size` in group_vars:
   ```yaml
   srsilo_virus_config:
     covid:
       chunk_size: 500000  # Reduce from 1000000
   ```

2. Increase `srsilo_docker_memory_limit` if RAM available:
   ```yaml
   srsilo_virus_config:
     covid:
       docker_memory_limit: 400g
   ```

3. Use test_vars.yml for resource-constrained environments:
   ```bash
   ansible-playbook playbooks/srsilo/update-pipeline.yml -i inventory.ini \
     -e "@playbooks/srsilo/vars/test_vars.yml"
   ```

### API Won't Start

**Symptoms:** LAPIS/SILO containers fail to start or immediately exit.

**Diagnosis:**

```bash
# Check Docker logs (use actual container name for your virus)
# COVID: wise-sarsCoV2-lapis / wise-sarsCoV2-silo
# RSV-A: wise-rsva-lapis / wise-rsva-silo
docker logs wise-sarsCoV2-lapis

# Verify index exists (replace <virus> with covid, rsva, etc.)
ls -la /opt/srsilo/<virus>/output/<timestamp>/

# Check permissions
ls -ld /opt/srsilo/<virus>/output
```

**Common causes:**

- Missing or corrupted index
- Permission issues on output directory
- Port already in use

### Timer Not Running

**Symptoms:** Automated pipeline runs aren't happening.

**Diagnosis:**

```bash
# Check status
systemctl status srsilo-update.timer

# View schedule
systemctl list-timers

# Check service logs
journalctl -u srsilo-update.service -n 100
```

**Solutions:**

1. Enable the timer:
   ```bash
   sudo systemctl enable srsilo-update.timer
   sudo systemctl start srsilo-update.timer
   ```

2. Re-run setup:
   ```bash
   ansible-playbook playbooks/srsilo/setup-timer.yml -i inventory.ini
   ```

### Containers Not Auto-Restarting After Reboot

**Symptoms:** SILO/LAPIS down after server reboot.

**Note:** The default `restart: unless-stopped` policy *does* restart containers after reboot, unless they were manually stopped. If containers aren't restarting, check:

1. Docker daemon status: `systemctl status docker`
2. Container was manually stopped before reboot
3. Container crashed during startup (check logs)

**To change restart policy:** Edit the Ansible template at `roles/srsilo/templates/docker-compose.yml.j2` and re-run the playbook. Direct edits to `/opt/srsilo/<virus>/config/docker-compose.yml` will be overwritten.

See [2025-11-20 Service Outage](../incidents/2025-11-20-service-outage.md) for detailed analysis.

### Pipeline Stuck / Orphaned State

**Symptoms:** Pipeline refuses to run, reports "preprocessing in progress".

**Diagnosis:**

```bash
# Check for orphan marker (replace <virus> with covid, rsva, etc.)
ls -la /opt/srsilo/<virus>/output/.preprocessing_in_progress
```

**Solution:**

```bash
# Clean failed run artifacts for a specific virus
sudo rm -rf /opt/srsilo/<virus>/sorted_chunks/*
sudo rm -rf /opt/srsilo/<virus>/tmp/*
sudo rm /opt/srsilo/<virus>/output/.preprocessing_in_progress
```

## Diagnostic Commands

### Check All Services

```bash
# Docker containers
docker ps -a | grep -E 'silo|lapis'

# Kubernetes pods (Loculus)
kubectl get pods -A

# Systemd timers
systemctl list-timers --all | grep srsilo
```

### Resource Usage

```bash
# Memory usage
free -h

# Disk usage
df -h /opt/srsilo

# Docker resource usage
docker stats --no-stream
```

### Network Connectivity

```bash
# API endpoints
curl -s http://localhost:8083/sample/info | head -3
curl -s http://localhost:8084/sample/info | head -3

# External connectivity
curl -s https://lapis.wasap.genspectrum.org/covid/sample/info | head -3
```

## See Also

- [Recovery Procedures](recovery.md)
- [Logging Guide](logging.md)
