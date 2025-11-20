# Nginx Configuration Inventory
*Initial Audit: November 13, 2025*  
*Updated After Improvements: November 20, 2025*

> **Note**: This inventory reflects the **improved** configuration state.  
> For original audit state, see `before-improvements/` folder.  
> For detailed changes, see [IMPROVEMENTS.md](IMPROVEMENTS.md).

## 1. All Subdomains/Virtual Hosts

### Active HTTPS Endpoints
1. **wasap.genspectrum.org** - Main Scout Application
2. **lapis.wasap.genspectrum.org** - LAPIS API Service
3. **silo.wasap.genspectrum.org** - Silo Service
4. **db.wasap.genspectrum.org** - Loculus Main Interface
5. **auth.db.wasap.genspectrum.org** - Loculus Authentication (Keycloak)
6. **api.db.wasap.genspectrum.org** - Loculus API Backend

### HTTP to HTTPS Redirects
- Default server (catch-all) on port 80 → redirects to HTTPS
- Individual redirects for each named subdomain

---

## 2. SSL Certificates Coverage

### Certificate Locations
All certificates managed by Let's Encrypt/Certbot:

| Domain | Certificate Path | Private Key Path |
|--------|------------------|------------------|
| wasap.genspectrum.org | `/etc/letsencrypt/live/wasap.genspectrum.org/fullchain.pem` | `/etc/letsencrypt/live/wasap.genspectrum.org/privkey.pem` |
| lapis.wasap.genspectrum.org | `/etc/letsencrypt/live/lapis.wasap.genspectrum.org/fullchain.pem` | `/etc/letsencrypt/live/lapis.wasap.genspectrum.org/privkey.pem` |
| silo.wasap.genspectrum.org | `/etc/letsencrypt/live/silo.wasap.genspectrum.org/fullchain.pem` | `/etc/letsencrypt/live/silo.wasap.genspectrum.org/privkey.pem` |
| db.wasap.genspectrum.org | `/etc/letsencrypt/live/db.wasap.genspectrum.org/fullchain.pem` | `/etc/letsencrypt/live/db.wasap.genspectrum.org/privkey.pem` |

### SSL Configuration
- **Protocols**: TLSv1.2, TLSv1.3 (recommended by Mozilla SSL Config)
- **Session Cache**: Shared cache (10MB)
- **Session Timeout**: 1440 minutes (24 hours)
- **Session Tickets**: Disabled
- **Cipher Suites**: Modern Mozilla configuration
- **DH Params**: `/etc/letsencrypt/ssl-dhparams.pem`
- **Options File**: `/etc/letsencrypt/options-ssl-nginx.conf`

### Certificate Domains in `/etc/letsencrypt/live/`
- db.wasap.genspectrum.org (shared by db, auth.db, and api.db)
- lapis.wasap.genspectrum.org
- silo.wasap.genspectrum.org
- wasap.genspectrum.org

---

## 3. Reverse Proxy Targets

| Subdomain | Backend Service | Local Port | Service Type |
|-----------|----------------|------------|--------------|
| wasap.genspectrum.org | Streamlit (Scout) | 127.0.0.1:9001 | Web Application |
| lapis.wasap.genspectrum.org | LAPIS API | 127.0.0.1:8083 | API Service |
| silo.wasap.genspectrum.org | Silo Service | 127.0.0.1:8081 | API Service |
| db.wasap.genspectrum.org | Loculus Frontend | 127.0.0.1:9000 | Web Application |
| auth.db.wasap.genspectrum.org | Keycloak | 127.0.0.1:9083 | Authentication |
| api.db.wasap.genspectrum.org | Loculus Backend | 127.0.0.1:9080 | API Service |
| api.db.wasap.genspectrum.org/backend/ | Loculus Backend Alt | 127.0.0.1:9079 | API Service |

### Upstream Definitions
```nginx
upstream lapis-wasap {
    server 127.0.0.1:8083;
}

upstream silo-wasap {
    server 127.0.0.1:8081;
}
```

### Special Proxy Configurations

#### Streamlit (wasap.genspectrum.org)
- WebSocket support enabled
- HTTP/1.1 with Upgrade headers
- Connection: "upgrade"
- Read timeout: 86400 seconds (24 hours)
- Proxy buffering: off

#### Keycloak (auth.db.wasap.genspectrum.org)
- Increased buffer sizes:
  - proxy_buffer_size: 16k
  - proxy_buffers: 4 16k
  - proxy_busy_buffers_size: 32k

#### Loculus API (api.db.wasap.genspectrum.org)
- Dual backend routing:
  - `/backend/` → 127.0.0.1:9079 (with X-Forwarded-Prefix)
  - `/` → 127.0.0.1:9080

---

## 4. Static Content Locations

### `/var/www/html/static/`
- **Served by**: db.wasap.genspectrum.org
- **URL Path**: `/static/`
- **Cache Policy**: 
  - Expires: 1 year
  - Cache-Control: "public, immutable"
- **Error Handling**: 404 if not found

---

## 5. Custom Nginx Modules or Non-Standard Configurations

### Global HTTP Settings
```nginx
proxy_http_version 1.1;  # Set globally in wasap.conf
```

### Worker Configuration
- **User**: www-data
- **Worker Processes**: auto
- **Worker Connections**: 768 per worker
- **PID File**: /run/nginx.pid

### Gzip Compression
- **Enabled**: Yes
- **Compression Level**: 6
- **Proxied**: any
- **Vary**: on
- **Types**: text/plain, text/css, application/json, application/javascript, text/xml, application/xml, application/xml+rss, text/javascript

### HTTP/2
- **Enabled**: Yes (on lapis and silo vhosts)
- **Format**: `listen 443 ssl http2;`

### Configuration File Structure
```
/etc/nginx/
├── nginx.conf (main)
├── conf.d/
│   └── wasap.conf (upstreams + lapis/silo vhosts)
├── sites-available/
│   ├── default
│   ├── monitoring
│   └── wasap-scout
└── sites-enabled/
    ├── loculus (db.wasap configs)
    └── wasap-scout → /etc/nginx/sites-available/wasap-scout
```

### Warning in Configuration
```
protocol options redefined for 0.0.0.0:443 in /etc/nginx/sites-enabled/loculus:13
```
This indicates SSL parameters are being redefined across multiple server blocks (expected with Certbot management).

---

## 6. Current Nginx Version and Compile Options

### Version Information
- **Version**: nginx/1.24.0 (Ubuntu)
- **Built with**: OpenSSL 3.0.13 (30 Jan 2024)
- **TLS SNI Support**: Enabled

### Core Configuration Paths
- **Prefix**: /usr/share/nginx
- **Config Path**: /etc/nginx/nginx.conf
- **HTTP Log**: /var/log/nginx/access.log
- **Error Log**: stderr
- **PID Path**: /run/nginx.pid
- **Modules Path**: /usr/lib/nginx/modules

### Compiled Modules (Built-in)
- http_ssl_module
- http_stub_status_module
- http_realip_module
- http_auth_request_module
- http_v2_module
- http_dav_module
- http_slice_module
- http_addition_module
- http_flv_module
- http_gunzip_module
- http_gzip_static_module
- http_mp4_module
- http_random_index_module
- http_secure_link_module
- http_sub_module
- mail_ssl_module
- stream_ssl_module
- stream_ssl_preread_module
- stream_realip_module

### Dynamic Modules (Loadable)
- http_geoip_module=dynamic
- http_image_filter_module=dynamic
- http_perl_module=dynamic
- http_xslt_module=dynamic
- mail=dynamic
- stream=dynamic
- stream_geoip_module=dynamic

### Build Configuration
- **Compiler Optimizations**: -O2, LTO enabled
- **Security Features**: 
  - Stack protector strong
  - Stack clash protection
  - Format string protection
  - Control flow protection
- **PCRE**: JIT compilation enabled
- **Threads**: Enabled
- **Debug**: Enabled
- **Compatibility Mode**: Enabled

### Temp Paths
- Client Body: /var/lib/nginx/body
- FastCGI: /var/lib/nginx/fastcgi
- Proxy: /var/lib/nginx/proxy
- SCGI: /var/lib/nginx/scgi
- uWSGI: /var/lib/nginx/uwsgi

---

## Summary Statistics

- **Total Subdomains**: 6
- **SSL Certificates**: 4 distinct certificates
- **Reverse Proxy Targets**: 7 backend services
- **Static Content Locations**: 1
- **Configuration Files**: 3 (nginx.conf, wasap.conf, loculus, wasap-scout)
- **Nginx Version**: 1.24.0 (Ubuntu official package)
- **HTTP/2**: Enabled on 2 of 6 vhosts
- **WebSocket Support**: 1 vhost (wasap.genspectrum.org)
