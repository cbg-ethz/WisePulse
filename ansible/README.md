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
├── playbooks/
│   └── deploy.yml                # Deployment playbook
└── secrets/                      # Legacy directory (can be removed)
    └── my-values.yaml            # Original monolithic config
```

## Security Model

- **Encrypted Secrets**: All sensitive data is stored in `group_vars/all/vault.yml` using Ansible Vault
- **Clear Configuration**: Non-sensitive configuration is in `group_vars/all/main.yml`
- **Host-specific**: Host-specific variables are in `host_vars/localhost/main.yml`

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

## Migration from Legacy Setup

The original `secrets/my-values.yaml` has been split into:

1. **Non-sensitive config** → `group_vars/all/main.yml`
2. **Sensitive data** → `group_vars/all/vault.yml` (encrypted)
3. **Host-specific** → `host_vars/localhost/main.yml`
4. **Template** → `templates/values.yaml.j2`

## Adding New Secrets

1. Edit the vault file:
   ```bash
   ansible-vault edit group_vars/all/vault.yml
   ```

2. Add the secret under the appropriate section (e.g., `vault_secrets.new_service.password`)

3. Reference it in the template: `{{ vault_secrets.new_service.password }}`
