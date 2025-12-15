# WisePulse AI Agent Profile

## Repository Context

WisePulse is an infrastructure project for viral wastewater surveillance that combines:
- **Ansible** for infrastructure automation (roles, playbooks)
- **Rust** tools for high-performance data processing (srSILO pipeline)
- **DevOps** practices (CI/CD, monitoring, deployment)
- **Docker & Kubernetes** for containerized services
- Components: Loculus, srSILO, Prometheus/Grafana monitoring, Nginx

## Code Quality Standards

### Ansible Best Practices
- Follow ansible-lint rules defined in `.ansible-lint`
- Use handlers for service restarts
- Implement idempotent tasks
- Tag tasks appropriately for selective execution
- Use variables from `defaults/` and override in `vars/` when needed
- Keep roles focused and single-purpose
- Document roles in `roles/*/README.md`

### Rust Standards
- Format code with `cargo fmt`
- Pass `cargo clippy` without warnings
- Write unit tests for all tools
- Handle errors properly (no unwrap in production code)
- Use descriptive error messages

### YAML/Configuration
- Maintain consistent indentation (2 spaces)
- Use meaningful variable names with role prefixes
- Comment complex configurations
- Keep secrets in vault files

## Development Workflow

### Before Making Changes
1. Check existing CI workflows in `.github/workflows/`
2. Review relevant role documentation in `docs/`
3. Understand the multi-phase srSILO pipeline architecture

### Testing Changes
- Ansible: `ansible-playbook --syntax-check` and `ansible-lint`
- Rust: `cargo test`, `cargo clippy`, `cargo fmt --check`
- Run relevant playbooks with `--check` mode when possible

### File Organization
- Playbooks: `playbooks/[component]/[action].yml`
- Roles: `roles/[component]/[tasks|handlers|defaults|templates]/`
- Docs: `docs/[component]/`
- Tools: `roles/srsilo/files/tools/src/[tool_name]/`

## Key Architecture Patterns

### srSILO Pipeline
- 7-phase automated update pipeline
- Self-healing with automatic rollback
- Low-downtime API management
- Smart execution (exits early if no new data)
- Retention policy for old indexes

### Infrastructure as Code
- Declarative Ansible playbooks
- Idempotent operations
- Environment-specific variables
- Vault-encrypted secrets

## Common Tasks

### Adding a New Ansible Role
1. Create role structure: `ansible-galaxy init roles/[name]`
2. Define defaults in `defaults/main.yml`
3. Implement tasks in `tasks/main.yml`
4. Add handlers if needed
5. Document in role's `README.md`
6. Create playbook in `playbooks/[component]/`

### Modifying srSILO Tools
1. Navigate to `roles/srsilo/files/tools/src/[tool]/`
2. Make changes to Rust code
3. Run `cargo fmt && cargo clippy`
4. Write/update tests
5. Update role tasks if tool interface changes

### Updating CI/CD
1. Modify workflows in `.github/workflows/`
2. Keep builds fast and focused
3. Use caching for dependencies
4. Test syntax and quality checks

## Documentation Requirements

- Update relevant docs in `docs/` when changing architecture
- Keep README.md current with new features
- Document configuration changes in role documentation
- Add inline comments for complex logic

## Security Considerations

- Never commit vault passwords or secrets
- Use Ansible vault for sensitive data
- Review Docker memory limits and resource constraints
- Validate input data in Rust tools
- Keep dependencies updated

## Performance Guidelines

- srSILO: Optimize for high-memory production environments (377GB RAM)
- Provide test configurations for constrained resources
- Use chunking for large data processing
- Monitor resource usage with Prometheus/Grafana

## When to Seek Help

- Changing core pipeline architecture
- Modifying multi-virus support plans
- Major role refactoring
- Breaking changes to APIs or interfaces
- Security-sensitive modifications
