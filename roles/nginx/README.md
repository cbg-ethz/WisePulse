# Nginx Role

This role installs and configures Nginx as a reverse proxy for the WisePulse application suite (Loculus, Lapis, Silo, Wasap-Scout).

## Multi-Virus Path-Based Routing

LAPIS and SILO support multiple virus instances via path-based routing:

| URL Pattern | Backend | Port |
|-------------|---------|------|
| `lapis.{domain}/covid/*` | LAPIS COVID | 8083 |
| `lapis.{domain}/rsva/*` | LAPIS RSVA | 8084 |
| `silo.{domain}/covid/*` | SILO COVID | 8081 |
| `silo.{domain}/rsva/*` | SILO RSVA | 8082 |

### Path Stripping

The `/covid` and `/rsva` prefixes are stripped before proxying to backends. For example:
- `https://lapis.wasap.genspectrum.org/covid/sample/info` → backend receives `/sample/info`

This is achieved using nginx's `proxy_pass` with a trailing slash, which automatically strips the matched location prefix:

```nginx
# Upstream definitions (in conf.d/wasap.conf)
upstream lapis-covid {
    server 127.0.0.1:8083;
}

# Location block (in sites-available/lapis.j2)
location /covid/ {
    proxy_pass http://lapis-covid/;  # trailing slash strips /covid/ prefix
    proxy_set_header X-Forwarded-Prefix /covid/;
}
```

Key implementation details:
- **Trailing slash in `proxy_pass`**: When both the location and proxy_pass end with `/`, nginx strips the location prefix from the request URI
- **Upstream blocks**: Defined in `conf.d/wasap.conf.j2` for cleaner configuration and future load balancing support
- **X-Forwarded-Prefix header**: Tells backends the original path prefix for proper URL generation in responses
- **Trailing slash redirects**: Requests to `/covid` (without trailing slash) return 301 to `/covid/` for consistent behavior

### Backward Compatibility

For backward compatibility with existing API consumers, requests to root paths are proxied directly to the COVID backend (not redirected). This preserves CORS preflight behavior for cross-origin requests:
- `https://lapis.wasap.genspectrum.org/sample/info` → proxies to COVID LAPIS backend

Using proxy instead of redirect avoids breaking CORS preflight (OPTIONS) requests, which don't follow redirects properly.

## Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `nginx_domain_name` | `wasap.genspectrum.org` | The base domain name for the deployment. Subdomains (db., lapis., silo.) are derived from this. |
| `nginx_lapis_covid_port` | `8083` | Port for COVID LAPIS instance |
| `nginx_lapis_rsva_port` | `8084` | Port for RSVA LAPIS instance |
| `nginx_silo_covid_port` | `8081` | Port for COVID SILO instance |
| `nginx_silo_rsva_port` | `8082` | Port for RSVA SILO instance |
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

## Testing

### Local Testing Considerations

When testing nginx configurations locally (e.g., on a VM or development machine), be aware of these complexities:

1. **DNS Resolution**: Public domain names (e.g., `lapis.wasap.genspectrum.org`) resolve to production IPs, not your local machine. Use `/etc/hosts` overrides or test via `127.0.0.1` with appropriate `Host` headers.

2. **SSL/SNI**: Self-signed certificates in staging may not match the expected hostnames. When multiple server blocks listen on port 443, nginx uses SNI to select the correct block. Mismatched certificates can cause requests to hit the wrong server block.

3. **Server Block Priority**: If testing via `127.0.0.1`, ensure no other server block (e.g., monitoring) listens specifically on `127.0.0.1:443`, as nginx prefers more specific listeners.

### Verifying Path-Based Routing

To test that path stripping works correctly without SSL complications:

```bash
# Create a test server on an unused port
cat > /tmp/test.conf << 'EOF'
upstream test-backend {
    server 127.0.0.1:8083;
}

server {
    listen 9999;

    location = /covid {
        return 301 /covid/;
    }

    location /covid/ {
        proxy_pass http://test-backend/;
        proxy_set_header X-Forwarded-Prefix /covid/;
    }
}
EOF

sudo cp /tmp/test.conf /etc/nginx/sites-available/test
sudo ln -sf /etc/nginx/sites-available/test /etc/nginx/sites-enabled/test
sudo nginx -t && sudo systemctl reload nginx

# Test - should return valid JSON from LAPIS backend
curl -s http://127.0.0.1:9999/covid/sample/info | head -3

# Cleanup
sudo rm /etc/nginx/sites-enabled/test
sudo systemctl reload nginx
```

### Production Verification

After deploying to the actual production/staging server:

```bash
# Test COVID endpoint (new path)
curl -s https://lapis.wasap.genspectrum.org/covid/sample/info | head -3

# Test RSVA endpoint (new path)
curl -s https://lapis.wasap.genspectrum.org/rsva/sample/info | head -3

# Test backward compatibility (proxied to COVID, not redirected)
curl -s https://lapis.wasap.genspectrum.org/sample/info | head -3
# Expected: HTTP/2 200, JSON response from COVID LAPIS
```
