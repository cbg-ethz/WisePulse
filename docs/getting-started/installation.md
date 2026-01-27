# Installation

## Prerequisites

- **Ansible** 2.16+
- **Docker** and **Docker Compose**
- **Kubernetes** (for Loculus), kubectl configured
- **Linux** (Ubuntu 20.04+, Debian 11+)

## Setup

### 1. Clone the Repository

```bash
git clone https://github.com/cbg-ethz/WisePulse.git
cd WisePulse
```

### 2. Configure Inventory

```bash
cp inventory.ini.example inventory.ini
vim inventory.ini  # Set your target hosts
```

### 3. Install Ansible Collections

```bash
ansible-galaxy collection install -r requirements.yml
```

### 4. Configure Variables

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

### 5. Run Setup Playbook

```bash
# One-time setup (creates user, directories, builds tools)
ansible-playbook playbooks/srsilo/setup.yml -i inventory.ini --become --ask-become-pass
```

## Next Steps

- [Quick Start Guide](quick-start.md) – Run your first commands
- [Configuration Reference](../configuration/reference.md) – Detailed configuration options
