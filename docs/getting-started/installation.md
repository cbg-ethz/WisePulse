# Installation

Ansible runs from your own machine (the control machine) and configures the target server
over SSH — it is not run on the server itself.

## Prerequisites

On your control machine:
- **Ansible** 2.16+

On the target server:
- **Docker** and **Docker Compose**
- **Kubernetes** (for Loculus), kubectl configured
- **Linux** (Ubuntu 20.04+, Debian 11+)
- A deploy user with SSH access and passwordless sudo (or sudo via `--ask-become-pass`)
- Your control machine's SSH key in that user's `~/.ssh/authorized_keys`

Verify SSH access before proceeding:

```bash
ssh <deploy_user>@<target_host>
```

## Setup

### 1. Clone the Repository

```bash
git clone https://github.com/cbg-ethz/WisePulse.git
cd WisePulse
```

### 2. Configure Inventory

Edit `inventory.ini` to point at your target host and deploy user:

```ini
[wisepulse_server]
<target_host> ansible_user=<deploy_user>
```

### 3. Install Ansible Collections

```bash
ansible-galaxy collection install -r requirements.yml
```

### 4. Set Up the Vault Password

`group_vars/loculus/vault.yml` and `group_vars/monitoring/vault.yml` hold secrets encrypted
with Ansible Vault. Decrypting them (and running any playbook) requires a `.vault_pass` file
in the repo root containing the vault password.

This file is not checked into git. Ask a project maintainer for the password, then create it:

```bash
echo '<vault password>' > .vault_pass
chmod 600 .vault_pass
```

### 5. Configure Variables

Edit the group variables for your environment:

```bash
# srSILO configuration
vim group_vars/srsilo/main.yml

# Loculus configuration
vim group_vars/loculus/main.yml
vim group_vars/loculus/vault.yml  # Encrypted secrets

# Monitoring configuration
vim group_vars/monitoring/main.yml
```

### 6. Run Setup Playbook

```bash
# One-time setup (creates user, directories, builds tools)
ansible-playbook playbooks/srsilo/setup.yml -i inventory.ini --become --ask-become-pass
```

## Next Steps

- [Quick Start Guide](quick-start.md) – Run your first commands
- [Configuration Reference](../configuration/reference.md) – Detailed configuration options
