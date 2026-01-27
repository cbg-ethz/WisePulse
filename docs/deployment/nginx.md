# Nginx Deployment

Nginx reverse proxy with SSL termination for all WisePulse services.

## Deploy

```bash
ansible-playbook playbooks/setup_nginx.yml -i inventory.ini --ask-become-pass
```

## Multi-Virus Path-Based Routing

LAPIS and SILO support multiple virus instances via path-based routing:

| URL Pattern | Backend | Port |
|-------------|---------|------|
| `lapis.{domain}/covid/*` | LAPIS COVID | 8083 |
| `lapis.{domain}/rsva/*` | LAPIS RSVA | 8084 |
| `silo.{domain}/covid/*` | SILO COVID | 8081 |
| `silo.{domain}/rsva/*` | SILO RSVA | 8082 |

### Path Stripping

The `/covid` and `/rsva` prefixes are stripped before proxying to backends:

- `https://lapis.wasap.genspectrum.org/covid/sample/info` → backend receives `/sample/info`

This is achieved using nginx's `proxy_pass` with a trailing slash:

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

**Key implementation details:**

- **Trailing slash in `proxy_pass`**: When both the location and proxy_pass end with `/`, nginx strips the location prefix
- **Upstream blocks**: Defined in `conf.d/wasap.conf.j2` for cleaner configuration and future load balancing support
- **X-Forwarded-Prefix header**: Tells backends the original path prefix
- **Trailing slash redirects**: Requests to `/covid` return 301 to `/covid/`

### Backward Compatibility

For backward compatibility with existing API consumers, requests to root paths are proxied directly to the COVID backend (not redirected). This preserves CORS preflight behavior for cross-origin requests:

- `https://lapis.wasap.genspectrum.org/sample/info` → proxies to COVID LAPIS backend

Using proxy instead of redirect avoids breaking CORS preflight (OPTIONS) requests, which don't follow redirects properly.

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `nginx_domain_name` | `wasap.genspectrum.org` | Base domain name. Subdomains (db., lapis., silo.) are derived from this. |
| `nginx_lapis_covid_port` | `8083` | COVID LAPIS port |
| `nginx_lapis_rsva_port` | `8084` | RSVA LAPIS port |
| `nginx_silo_covid_port` | `8081` | COVID SILO port |
| `nginx_silo_rsva_port` | `8082` | RSVA SILO port |
| `nginx_ssl_certificate_path` | `/etc/letsencrypt/live/{{ nginx_domain_name }}/fullchain.pem` | SSL certificate path |
| `nginx_ssl_certificate_key_path` | `/etc/letsencrypt/live/{{ nginx_domain_name }}/privkey.pem` | SSL key path |
| `nginx_dhparam_path` | `/etc/letsencrypt/ssl-dhparams.pem` | Diffie-Hellman parameters file |
| `nginx_sites` | List of all sites | Site templates to deploy to `sites-available` |
| `nginx_enabled_sites` | List of active sites | Sites to symlink to `sites-enabled` |

## SSL Configuration

Uses Let's Encrypt certificates:

- **Wasap Scout (Main Domain):** `nginx_ssl_certificate_path` and `nginx_ssl_certificate_key_path`
- **Lapis:** `nginx_lapis_ssl_certificate_path` and `nginx_lapis_ssl_certificate_key_path`
- **Silo:** `nginx_silo_ssl_certificate_path` and `nginx_silo_ssl_certificate_key_path`
- **Loculus (db, auth.db, api.db):** `nginx_loculus_ssl_certificate_path` and `nginx_loculus_ssl_certificate_key_path`

All subdomain certificate paths default to the standard Let's Encrypt structure (`/etc/letsencrypt/live/<subdomain>.{{ nginx_domain_name }}/`).

### Production

In production, certificates are expected to be managed by Certbot at the default Let's Encrypt paths.

```yaml
- hosts: webservers
  roles:
    - nginx
```

### Staging / Testing

For environments without real certificates:

```bash
ansible-playbook -i inventory.ini playbooks/setup_nginx.yml -e "generate_self_signed_certs=true"
```

## Testing

### Verify Path-Based Routing

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

```bash
# Test COVID endpoint (new path)
curl -s https://lapis.wasap.genspectrum.org/covid/sample/info | head -3

# Test RSVA endpoint (new path)
curl -s https://lapis.wasap.genspectrum.org/rsva/sample/info | head -3

# Test backward compatibility (proxied to COVID, not redirected)
curl -s https://lapis.wasap.genspectrum.org/sample/info | head -3
```

## Local Testing Considerations

1. **DNS Resolution**: Public domain names resolve to production IPs. Use `/etc/hosts` overrides or test via `127.0.0.1` with appropriate `Host` headers.

2. **SSL/SNI**: Self-signed certificates may not match expected hostnames.

3. **Server Block Priority**: If testing via `127.0.0.1`, ensure no other server block listens specifically on `127.0.0.1:443`.

## Role Structure

```
roles/nginx/
├── tasks/       # Main installation and configuration tasks
├── templates/   # Nginx configuration templates
├── handlers/    # Service reload/restart handlers
└── defaults/    # Default variable definitions
```
