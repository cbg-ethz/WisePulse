# srSILO Deployment

## Setup

```bash
# Clone and configure
git clone https://github.com/cbg-ethz/WisePulse.git
cd WisePulse
cp inventory.ini.example inventory.ini
vim group_vars/srsilo/main.yml

# One-time setup (creates user, builds tools, deploys configs)
ansible-playbook playbooks/srsilo/setup.yml -i inventory.ini --become --ask-become-pass
```

## Operations

### Manual Run

```bash
# Full 7-phase pipeline
ansible-playbook playbooks/srsilo/update-pipeline.yml -i inventory.ini --become --ask-become-pass

# Test mode (8GB RAM, 5M reads, 30k chunks)
ansible-playbook playbooks/srsilo/update-pipeline.yml -i inventory.ini \
  --become --ask-become-pass \
  -e "@playbooks/srsilo/vars/test_vars.yml"
```

### Automation

```bash
# Enable daily automation (2 AM systemd timer)
ansible-playbook playbooks/srsilo/setup-timer.yml -i inventory.ini --become

# View logs
journalctl -t srsilo-update -n 100 --no-pager
journalctl -u srsilo-update.service -f  # Follow timer runs
```

### Update All Viruses (Production)

```bash
ansible-playbook playbooks/srsilo/update-all-viruses.yml -i inventory.ini --become --ask-become-pass
```

### Update Single Virus

```bash
ansible-playbook playbooks/srsilo/update-pipeline.yml -i inventory.ini -e "srsilo_virus=rsva"
```

## Verification

```bash
# Check API status
docker ps | grep silo
curl http://localhost:8083/sample/info  # COVID
curl http://localhost:8084/sample/info  # RSV-A

# Timer status
systemctl status srsilo-update.timer
systemctl list-timers srsilo-update.timer
```

## See Also

- [srSILO Architecture](../architecture/srsilo-pipeline.md)
- [Configuration Reference](../configuration/reference.md)
- [Recovery Procedures](../operations/recovery.md)
- [Troubleshooting](../operations/troubleshooting.md)
