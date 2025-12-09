# Incident Report: Service Outage After System Upgrade

**Date:** November 20, 2025  
**Server:** WisePulse Production (Hetzner)  
**Duration:** ~10-15 minutes  
**Severity:** Medium  

## Summary

After a routine `apt upgrade` and reboot, SILO/LAPIS and V-Pipe Scout failed to auto-restart. Loculus (k3d) recovered automatically. 

**Root cause:** Operator error - did not anticipate that upgrading `containerd.io` (1.7.27→2.1.5) would restart Docker daemon, stopping all containers. Combined with `restart: unless-stopped` policy (which doesn't restart manually-stopped containers), this prevented auto-recovery after reboot.

## Timeline

| Time | Event |
|------|-------|
| 15:23 | `apt upgrade` started (44 packages including Docker/containerd) |
| 15:26 | Upgrade completed |
| 15:27 | Server rebooted (new kernel 6.8.0-88) |
| 15:31 | Server back online (boot time: 3m 43s) |
| 15:31 | Loculus ✅ auto-started, SILO/LAPIS ❌ down, V-Pipe Scout ❌ down |
| 15:40 | SILO/LAPIS manually restarted |
| 15:41 | V-Pipe Scout manually restarted |

## Root Cause

**Operator Error:** Ran `apt upgrade` without checking which packages would be upgraded and their impact.

### What Happened
1. **Unexpected Docker restart:** `containerd.io` major version upgrade (1.7.27→2.1.5) triggered Docker daemon restart
2. **Docker stopped all containers** gracefully during the restart
3. **Containers marked "manually stopped"** by Docker's shutdown logic
4. **Reboot occurred** (for new kernel)
5. **`restart: unless-stopped` policy** on SILO/LAPIS skipped "manually stopped" containers
6. **`restart: no` policy** on V-Pipe Scout never auto-restarts anyway

### Why This Wasn't Obvious
- `restart: unless-stopped` seems like it should mean "restart after reboot" - but it doesn't
- Package upgrades don't always show which services will restart
- No staging environment to test upgrade impact

### Lessons Learned
- **Check package list before upgrading:** `apt list --upgradable` to spot Docker/containerd
- **Expect Docker restarts** when upgrading containerd, docker-ce, or docker-compose-plugin
- **Use `restart: always`** for production services that must survive reboots/restarts
- **Test upgrades in staging** before production

---

## Key Packages Upgraded

- **containerd.io:** 1.7.27 → 2.1.5 (major upgrade, caused Docker restart)
- **docker-ce:** 5:28.3.2 → 5:29.0.2
- **openssh-server:** Security update (SSH config conflict handled correctly)
- **linux-image-generic:** 6.8.0-87 → 6.8.0-88 (new kernel, required reboot)
- Total: 44 packages

## Notes

- ✅ SSH config preserved correctly (password auth still disabled)
- ✅ Boot time 3m43s is normal (2m41s firmware init)
- ✅ No data loss
- ✅ All services recovered successfully

## Preventive Measures

### Before Next Upgrade

**Pre-upgrade checklist:**
```bash
# 1. Check what will be upgraded
apt list --upgradable | grep -E 'docker|containerd|systemd|kernel'

# 2. If Docker/containerd is upgrading:
#    - Expect Docker daemon restart → all containers stop
#    - Plan for manual service restart after upgrade
#    - OR fix restart policies first (see below)

# 3. Timing considerations
#    - Off-peak hours
#    - Announce maintenance window to users
#    - Have rollback plan
```

## Action Items

### 🔴 High Priority (1 week)

1. **Fix SILO/LAPIS restart policy**
   ```yaml
   # /opt/WisePulse/roles/srsilo/templates/docker-compose.yml.j2
   services:
     lapisOpen:
       restart: always  # Change from unless-stopped
     silo:
       restart: always  # Change from unless-stopped
   ```

2. **Fix V-Pipe Scout restart policy**
   ```yaml
   # /opt/v-pipe-scout/docker-compose.yml
   services:
     redis:
       restart: always  # Add
     streamlit:
       restart: always  # Add
     worker:
       restart: always  # Add
   ```

### 🟡 Medium Priority (1 month)

3. **Migrate V-Pipe Scout to Ansible** - Consistent management with other services
4. **Audit all Docker restart policies** - Run: `docker inspect --format '{{.Name}}: {{.HostConfig.RestartPolicy.Name}}' $(docker ps -aq)`

### 🟢 Low Priority (3 months)

5. **Implement staging environment** - Test upgrades before production
6. **Add service monitoring** - Prometheus/Grafana alerting for failures
7. **Document update procedures** - Create `/opt/WisePulse/docs/SERVER_MAINTENANCE.md`

## Quick Reference

**Manual restart commands:**
```bash
# SILO/LAPIS
cd /opt/srsilo/tools && LAPIS_PORT=8083 docker compose up -d

# V-Pipe Scout  
cd /opt/v-pipe-scout && docker compose up -d

# Check status
kubectl get pods -A  # Loculus
docker ps -a         # All containers
```
