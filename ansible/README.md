# Ansible Setup Documentation

## Directory Structure

This Ansible setup follows best practices with proper separation of concerns:

```
ansible/
├── ansible.cfg                     # Ansible configuration
├── inventory.ini                   # Inventory definition
├── .vault_pass                    # Vault password file (not in git)
├── group_vars/
│   └── all/
│       ├── main.yml              # Non-sensitive configuration
│       └── vault.yml             # Encrypted secrets (vault)
├── host_vars/
│   └── localhost/
│       └── main.yml              # Host-specific configuration
├── templates/
│   └── values.yaml.j2            # Jinja2 template for Kubernetes values
└── playbooks/
    └── deploy.yml                # Deployment playbook
```

## Usage

### Deploy to Kubernetes
```bash
cd ansible
ansible-playbook playbooks/deploy.yml
```

### Edit Secrets
```bash
cd ansible
ansible-vault edit group_vars/all/vault.yml
```

### View Encrypted Secrets (without editing)
```bash
cd ansible
ansible-vault view group_vars/all/vault.yml
```

## Adding New Secrets

1. Edit the vault file:
   ```bash
   ansible-vault edit group_vars/all/vault.yml
   ```

2. Add the secret under the appropriate section (e.g., `vault_secrets.new_service.password`)

3. Reference it in the template: `{{ vault_secrets.new_service.password }}`
