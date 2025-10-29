# Loculus Role

Ansible role for deploying Loculus (pathogen sequence sharing platform) to Kubernetes.

## Role Variables

**Defaults** (`defaults/main.yml`):
- `loculus_temp_dir`: Temp directory for deployment files
- `loculus_cleanup_temp_files`: Clean up temp files after deployment (default: `true`)
- `loculus_kubeconfig_path`: Path to kubeconfig file

**Required** (set in `host_vars/` or `group_vars/`):
- `kubectl_context`: kubectl context name
- `helm_chart_path`: Path to Loculus Helm chart

**Configuration** (`group_vars/loculus/main.yml`):
- Application settings (name, host, organisms, URLs, etc.)
- See main configuration file for all available options

**Secrets** (`group_vars/loculus/vault.yml`):
- Database credentials
- Keycloak settings
- S3 credentials
- Service account passwords

## Usage

See parent `ansible/README.md` for deployment instructions.



