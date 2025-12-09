# Nginx Role

This role installs and configures Nginx as a reverse proxy for the WisePulse application suite (Loculus, Lapis, Silo, Wasap-Scout).

## Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `nginx_domain_name` | `wasap.genspectrum.org` | The base domain name for the deployment. Subdomains (db., lapis., silo.) are derived from this. |
| `nginx_ssl_certificate_path` | `/etc/letsencrypt/live/{{ nginx_domain_name }}/fullchain.pem` | Path to the SSL certificate. |
| `nginx_ssl_certificate_key_path` | `/etc/letsencrypt/live/{{ nginx_domain_name }}/privkey.pem` | Path to the SSL private key. |
| `nginx_dhparam_path` | `/etc/letsencrypt/ssl-dhparams.pem` | Path to the Diffie-Hellman parameters file. |
| `nginx_sites` | List of all sites | List of site templates to deploy to `sites-available`. |
| `nginx_enabled_sites` | List of active sites | List of sites to symlink to `sites-enabled`. |

## SSL Configuration

The role is configured to use Let's Encrypt certificates.

*   **Wasap Scout (Main Domain):** Uses `nginx_ssl_certificate_path` and `nginx_ssl_certificate_key_path`.
*   **Lapis:** Uses `nginx_lapis_ssl_certificate_path` and `nginx_lapis_ssl_certificate_key_path`.
*   **Silo:** Uses `nginx_silo_ssl_certificate_path` and `nginx_silo_ssl_certificate_key_path`.
*   **Loculus (db, auth.db, api.db):** Uses `nginx_loculus_ssl_certificate_path` and `nginx_loculus_ssl_certificate_key_path`.

All subdomain certificate paths default to the standard Let's Encrypt structure (`/etc/letsencrypt/live/<subdomain>.{{ nginx_domain_name }}/`).

## Usage

### Production
In production, certificates are expected to be managed by Certbot at the default Let's Encrypt paths.

```yaml
- hosts: webservers
  roles:
    - nginx
```

### Staging / Testing
For staging environments where real certificates might not exist, you can use the `setup_nginx.yml` playbook with `generate_self_signed_certs=true`. This will generate self-signed certificates for the configured domains before running the role.

```bash
ansible-playbook -i inventory.ini playbooks/setup_nginx.yml -e "generate_self_signed_certs=true"
```

## Structure
- `tasks/`: Main installation and configuration tasks.
- `templates/`: Nginx configuration templates (sites, snippets, conf.d).
- `handlers/`: Service reload/restart handlers.
- `defaults/`: Default variable definitions.
