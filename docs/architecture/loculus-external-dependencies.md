# Loculus External Dependencies

Loculus relies on external, unmanaged components that must be provisioned separately from the Ansible-managed infrastructure.

## PostgreSQL Database

Loculus requires two PostgreSQL databases:

- **Main Database**: Stores sequence metadata, user submissions, and application data
- **Keycloak Database**: Manages authentication and user identity data

These databases are **not managed** by WisePulse's Ansible playbooks and must be set up externally.

### Configuration

Database credentials are stored in `group_vars/loculus/vault.yml` (encrypted).

## S3-Compatible Object Storage

Loculus uses S3-compatible object storage for storing sequence data files (BAM alignments, nucleotide sequences).

### Configuration

Configured in `group_vars/loculus/main.yml`:

```yaml
s3:
  enabled: true
  bucket:
    region: eu-central
    endpoint: https://fsn1.your-objectstorage.com/
    bucket: wasap
```

S3 credentials (access key, secret key) are stored in `group_vars/loculus/vault.yml`.

### Storage Requirements

The S3 bucket stores:
- Uploaded nucleotide alignments
- srSILO read files
- Preprocessing artifacts

Storage size depends on the number of samples and file sizes.

## See Also

- [Loculus Deployment](../deployment/loculus.md) for deployment instructions
- [Configuration Reference](../configuration/reference.md) for all configuration options
