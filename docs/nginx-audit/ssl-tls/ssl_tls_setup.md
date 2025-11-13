# SSL/TLS Setup Documentation
*Generated: November 13, 2025*

## 1. Let's Encrypt Certificates Inventory

### Active Certificates

| Certificate Name | Domains Covered | Expiry Date | Days Valid | Key Type |
|-----------------|-----------------|-------------|------------|----------|
| db.wasap.genspectrum.org | db.wasap.genspectrum.org<br>api.db.wasap.genspectrum.org<br>auth.db.wasap.genspectrum.org | 2025-12-25 | 42 | ECDSA |
| lapis.wasap.genspectrum.org | lapis.wasap.genspectrum.org | 2026-01-10 | 58 | ECDSA |
| silo.wasap.genspectrum.org | silo.wasap.genspectrum.org | 2026-01-10 | 58 | ECDSA |
| wasap.genspectrum.org | wasap.genspectrum.org | 2026-01-02 | 50 | ECDSA |

### Certificate Details

**Total Certificates**: 4  
**Total Domains Covered**: 6  
**Key Algorithm**: ECDSA (Elliptic Curve) - Modern, more efficient than RSA  
**Certificate Authority**: Let's Encrypt  
**ACME Server**: https://acme-v02.api.letsencrypt.org/directory

### Multi-Domain Certificate
The `db.wasap.genspectrum.org` certificate is a **SAN (Subject Alternative Name)** certificate covering:
- db.wasap.genspectrum.org (main domain)
- api.db.wasap.genspectrum.org (API subdomain)
- auth.db.wasap.genspectrum.org (Auth subdomain)

This reduces the number of certificates needed and simplifies management.

---

## 2. Certificate Renewal Mechanism

### Automated Renewal via Systemd

**Status**: ✅ **Fully Automated**

#### Systemd Timer Configuration
```ini
# /usr/lib/systemd/system/certbot.timer
[Unit]
Description=Run certbot twice daily

[Timer]
OnCalendar=*-*-* 00,12:00:00      # Runs at midnight and noon daily
RandomizedDelaySec=43200           # Random delay up to 12 hours
Persistent=true                    # Catches up if system was off

[Install]
WantedBy=timers.target
```

**Schedule**: Runs twice daily (00:00 and 12:00) with random delay  
**Next Run**: Visible via `systemctl list-timers`

#### Systemd Service Configuration
```ini
# /usr/lib/systemd/system/certbot.service
[Unit]
Description=Certbot
Documentation=file:///usr/share/doc/python-certbot-doc/html/index.html
Documentation=https://certbot.eff.org/docs

[Service]
Type=oneshot
ExecStart=/usr/bin/certbot -q renew --no-random-sleep-on-renew
PrivateTmp=true
```

**Renewal Command**: `/usr/bin/certbot -q renew --no-random-sleep-on-renew`  
**Quiet Mode**: `-q` suppresses output unless there's an error  
**No Random Sleep**: Timer already provides randomization

### Renewal Parameters

From `/etc/letsencrypt/renewal/*.conf`:

```ini
renew_before_expiry = 30 days      # Renews when cert has 30 days left
version = 2.9.0                    # Certbot version
authenticator = nginx              # Uses nginx plugin
installer = nginx                  # Automatically updates nginx configs
server = https://acme-v02.api.letsencrypt.org/directory
key_type = ecdsa                   # Elliptic Curve keys
```

**Renewal Window**: Certificates are renewed 30 days before expiration  
**Authenticator**: nginx plugin (no need to stop nginx during renewal)  
**Auto-reload**: nginx automatically reloaded after successful renewal

### Renewal Monitoring

**Timer Status**: Check with `systemctl status certbot.timer`  
**Service Status**: Check with `systemctl status certbot.service`  
**Last Run**: Visible in `systemctl list-timers`  
**Logs**: `/var/log/letsencrypt/letsencrypt.log`

---

## 3. Custom SSL Configurations

### Global SSL Settings

File: `/etc/letsencrypt/options-ssl-nginx.conf`  
**Managed by**: Certbot (Mozilla SSL Configuration Generator)  
**Warning**: Manual edits will be overwritten by certbot updates

```nginx
# Session Cache
ssl_session_cache shared:le_nginx_SSL:10m;
ssl_session_timeout 1440m;         # 24 hours
ssl_session_tickets off;           # Disabled for forward secrecy

# Protocols
ssl_protocols TLSv1.2 TLSv1.3;     # Modern protocols only

# Cipher Configuration
ssl_prefer_server_ciphers off;     # Let client choose (TLS 1.3 best practice)

# Cipher Suites (Mozilla Intermediate Profile)
ssl_ciphers "ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305:DHE-RSA-AES128-GCM-SHA256:DHE-RSA-AES256-GCM-SHA384";
```

### DH Parameters

File: `/etc/letsencrypt/ssl-dhparams.pem`  
**Purpose**: Diffie-Hellman key exchange parameters  
**Managed by**: Certbot

### Certificate File Locations

Standard Let's Encrypt directory structure:

```
/etc/letsencrypt/
├── live/                          # Symlinks to current certificates
│   ├── db.wasap.genspectrum.org/
│   │   ├── fullchain.pem         # Certificate + intermediate chain
│   │   ├── privkey.pem           # Private key
│   │   ├── cert.pem              # Certificate only
│   │   └── chain.pem             # Intermediate chain only
│   ├── lapis.wasap.genspectrum.org/
│   ├── silo.wasap.genspectrum.org/
│   └── wasap.genspectrum.org/
├── archive/                       # Actual certificate files (versioned)
├── renewal/                       # Renewal configuration files
│   ├── db.wasap.genspectrum.org.conf
│   ├── lapis.wasap.genspectrum.org.conf
│   ├── silo.wasap.genspectrum.org.conf
│   └── wasap.genspectrum.org.conf
├── options-ssl-nginx.conf        # SSL configuration (Mozilla profile)
└── ssl-dhparams.pem              # DH parameters
```

### Per-VHost SSL Configuration

All vhosts include the same SSL settings:

```nginx
listen 443 ssl;  # or `listen 443 ssl http2;` for HTTP/2
ssl_certificate /etc/letsencrypt/live/<domain>/fullchain.pem;
ssl_certificate_key /etc/letsencrypt/live/<domain>/privkey.pem;
include /etc/letsencrypt/options-ssl-nginx.conf;
ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem;
```

**HTTP/2 Support**:
- ✅ Enabled: lapis.wasap.genspectrum.org, silo.wasap.genspectrum.org
- ❌ Not enabled: wasap.genspectrum.org, db.wasap.genspectrum.org, auth.db.wasap.genspectrum.org, api.db.wasap.genspectrum.org

---

## 4. Security Assessment

### ✅ Strong Security Posture

- **Modern TLS Only**: TLSv1.2 and TLSv1.3 (no deprecated protocols)
- **Strong Ciphers**: AEAD ciphers (GCM, ChaCha20-Poly1305)
- **Forward Secrecy**: ECDHE and DHE key exchange
- **ECDSA Certificates**: More efficient than RSA
- **Automated Renewal**: No risk of expired certificates
- **Session Ticket Disabled**: Enhanced privacy/security
- **Mozilla Intermediate Profile**: Good balance of security and compatibility

### 🔧 Potential Improvements

1. **Enable HTTP/2 on all vhosts** (currently only 2/6 have it)
2. **Consider HSTS headers** (HTTP Strict Transport Security)
3. **OCSP Stapling** (faster certificate validation)
4. **Certificate Transparency monitoring** (via ct.cloudflare.com or similar)

---

## 5. SSL/TLS Testing

### Recommended Tests

```bash
# Test SSL configuration
openssl s_client -connect wasap.genspectrum.org:443 -servername wasap.genspectrum.org

# Check certificate expiry
echo | openssl s_client -connect wasap.genspectrum.org:443 -servername wasap.genspectrum.org 2>/dev/null | openssl x509 -noout -dates

# Test all cipher suites
nmap --script ssl-enum-ciphers -p 443 wasap.genspectrum.org
```

### Online Tools
- **SSL Labs**: https://www.ssllabs.com/ssltest/analyze.html?d=wasap.genspectrum.org
- **Certificate Transparency Logs**: https://crt.sh/?q=wasap.genspectrum.org

---

## Summary

| Aspect | Status | Details |
|--------|--------|---------|
| **Certificates** | ✅ Active | 4 certificates, 6 domains, all valid |
| **Renewal** | ✅ Automated | Systemd timer, twice daily checks |
| **Protocol** | ✅ Modern | TLSv1.2 + TLSv1.3 only |
| **Ciphers** | ✅ Strong | Mozilla Intermediate profile |
| **Key Type** | ✅ Modern | ECDSA (not RSA) |
| **Automation** | ✅ Full | No manual intervention needed |
| **Documentation** | ✅ Complete | This file! |

---

## Ansible Migration Considerations

When migrating to Ansible:

1. **Install certbot + nginx plugin**: `certbot python3-certbot-nginx`
2. **Enable systemd timer**: `systemctl enable --now certbot.timer`
3. **Initial certificate generation**:
   ```bash
   certbot --nginx -d domain.com -d www.domain.com
   ```
4. **Verify renewal works**: `certbot renew --dry-run`
5. **Monitor renewal timer**: Include in monitoring/alerting

### Template Variables for Ansible

```yaml
ssl_certificate_email: "admin@genspectrum.org"
ssl_domains:
  - name: wasap.genspectrum.org
    domains: [wasap.genspectrum.org]
  - name: lapis.wasap.genspectrum.org
    domains: [lapis.wasap.genspectrum.org]
  - name: silo.wasap.genspectrum.org
    domains: [silo.wasap.genspectrum.org]
  - name: db.wasap.genspectrum.org
    domains:
      - db.wasap.genspectrum.org
      - api.db.wasap.genspectrum.org
      - auth.db.wasap.genspectrum.org
```
