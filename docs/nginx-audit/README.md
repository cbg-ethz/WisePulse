# Nginx Audit Documentation

This directory contains comprehensive documentation of the nginx configuration on the production server, including the initial audit and subsequent improvements.

## Structure

```
nginx-audit/
├── inventory.md                    # Main summary document (updated)
├── IMPROVEMENTS.md                 # Detailed documentation of all improvements made
├── before-improvements/            # Original configuration (Nov 13, 2025)
│   ├── nginx.conf
│   ├── conf.d/wasap.conf
│   └── sites-available/
│       ├── default
│       ├── monitoring
│       └── wasap-scout
├── after-improvements/             # Current configuration (Nov 20, 2025)
│   ├── nginx.conf
│   ├── snippets/ssl-params.conf   # NEW: Shared SSL/TLS config
│   ├── conf.d/wasap.conf          # UPDATED: Upstreams only
│   └── sites-available/
│       ├── default
│       ├── monitoring
│       ├── wasap-scout
│       ├── lapis                  # NEW: Extracted from conf.d
│       ├── silo                   # NEW: Extracted from conf.d
│       └── loculus                # NEW: Moved from sites-enabled
├── diagnostics/
│   ├── before/                    # Initial diagnostic outputs
│   │   ├── compile_time_options.txt
│   │   ├── full_running_conf.txt
│   │   ├── sites_available.txt
│   │   ├── sites_enabled.txt
│   │   └── systemctl_status.txt
│   └── after/                     # Post-improvement diagnostics
│       ├── full_running_conf.txt
│       ├── sites_available.txt
│       └── sites_enabled.txt
└── ssl-tls/                       # SSL/TLS documentation and cert info
    ├── ssl_tls_setup.md
    ├── certbot_certificates.txt
    └── ...
```

## Quick Start

### Understanding the Current Configuration
1. **Start here**: Read [inventory.md](inventory.md) for high-level overview
2. **See what changed**: Read [IMPROVEMENTS.md](IMPROVEMENTS.md)
3. **Compare configs**: Diff `before-improvements/` vs `after-improvements/`

### Key Improvements Made
- ✅ HTTP/2 enabled on all 6 vhosts
- ✅ SSL/TLS settings consolidated into shared snippet
- ✅ HSTS headers added for security
- ✅ Vhost organization standardized (Debian/Ubuntu conventions)

## Files Guide

### Main Documentation
- **inventory.md** - Comprehensive inventory of all nginx configuration
  - All subdomains/vhosts
  - SSL certificates
  - Reverse proxy targets
  - Static content locations
  - Nginx version and modules

- **IMPROVEMENTS.md** - Detailed documentation of improvements
  - What changed and why
  - Before/after comparisons
  - Testing and validation
  - Performance impact
  - Rollback procedures

### Configuration Files

#### Before Improvements (Nov 13, 2025)
Original state captured during initial audit.

**Key characteristics:**
- Vhosts mixed between `conf.d/` and `sites-enabled/`
- HTTP/2 only on 2/6 vhosts
- No HSTS headers
- SSL settings duplicated in each vhost

#### After Improvements (Nov 20, 2025)
Current production state with all improvements applied.

**Key characteristics:**
- All vhosts in `sites-available/`, symlinked from `sites-enabled/`
- HTTP/2 on all 6 vhosts
- HSTS headers on all vhosts
- Shared SSL configuration in `snippets/ssl-params.conf`
- Clean separation: domains vs upstreams

### Diagnostic Outputs

#### Before
Initial diagnostic data:
- `full_running_conf.txt` - Complete nginx -T output
- `sites_available.txt` - Files in sites-available/
- `sites_enabled.txt` - Files in sites-enabled/
- `compile_time_options.txt` - nginx -V output
- `systemctl_status.txt` - Service status

#### After
Post-improvement diagnostic data:
- `full_running_conf.txt` - Updated nginx -T output
- `sites_available.txt` - Updated file listing
- `sites_enabled.txt` - Updated symlink listing

### SSL/TLS Documentation
- **ssl_tls_setup.md** - Comprehensive SSL/TLS documentation
  - Certificate inventory
  - Automated renewal mechanism (certbot)
  - SSL configuration details
  - Security assessment

## Common Tasks

### Compare Before and After
```bash
# Compare main config
diff before-improvements/nginx.conf after-improvements/nginx.conf

# Compare wasap.conf (now only upstreams)
diff before-improvements/conf.d/wasap.conf after-improvements/conf.d/wasap.conf

# See new files
ls after-improvements/sites-available/
# lapis, silo, loculus are new
```

### Verify Current State Matches Documentation
```bash
# Check production matches after-improvements/
diff /etc/nginx/nginx.conf after-improvements/nginx.conf
diff /etc/nginx/conf.d/wasap.conf after-improvements/conf.d/wasap.conf
diff /etc/nginx/snippets/ssl-params.conf after-improvements/snippets/ssl-params.conf
```

### Review Full Running Config
```bash
# Before improvements
less diagnostics/before/full_running_conf.txt

# After improvements
less diagnostics/after/full_running_conf.txt
```

## Timeline

- **November 13, 2025** - Initial audit performed
  - Documented existing configuration
  - Identified improvement opportunities
  
- **November 20, 2025** - Improvements implemented
  - Enabled HTTP/2 on all vhosts
  - Consolidated SSL/TLS settings
  - Added HSTS headers
  - Standardized vhost organization
  - Updated documentation

## Related Issues

- Initial audit: #116
- HTTP/2 improvements: #117
- Configuration improvements: #118

## Next Steps

For future work, see "Future Recommendations" in [IMPROVEMENTS.md](IMPROVEMENTS.md):
- OCSP Stapling
- Additional security headers
- HSTS preload submission
