# Loculus Deployment

Ansible role for deploying Loculus (pathogen sequence sharing platform) to Kubernetes.

## Prerequisites

- Kubernetes cluster with kubectl configured
- Helm chart for Loculus
- Ansible with community.general collection

## Configuration

### Variables

**Defaults** (`defaults/main.yml`):

| Variable | Description |
|----------|-------------|
| `loculus_temp_dir` | Temp directory for deployment files |
| `loculus_cleanup_temp_files` | Clean up temp files after deployment (default: `true`) |
| `loculus_kubeconfig_path` | Path to kubeconfig file |

**Configuration** (`group_vars/loculus/main.yml`):

- Application settings (name, host, organisms, URLs, etc.)
- See main configuration file for all available options

**Secrets** (`group_vars/loculus/vault.yml`):

- Database credentials
- Keycloak settings
- S3 credentials
- Service account passwords

## Deploy

```bash
# Configure
vim group_vars/loculus/main.yml
vim group_vars/loculus/vault.yml  # Encrypted secrets

# Deploy to Kubernetes
ansible-playbook playbooks/loculus/deploy-loculus.yml -i inventory.ini
```

## Verification

```bash
# Check pods
kubectl get pods -A | grep loculus

# Check services
kubectl get svc -A | grep loculus

# View logs
kubectl logs <pod-name> -n <namespace> -f
```

## See Also

- [Architecture Overview](../architecture/overview.md)
- [Nginx Configuration](nginx.md) for reverse proxy setup
