# GitHub Copilot Agent Profiles

This directory contains custom agent profiles for GitHub Copilot to help maintain code quality, follow best practices, and provide expert guidance for the WisePulse project.

## Available Agents

### General Purpose

#### [agent.md](agent.md) - Main Agent Profile
**Purpose**: General guidance for all contributors working on WisePulse

**Use when**: 
- Starting work on any part of the project
- Need overview of project structure and standards
- Understanding development workflow
- General code quality questions

**Expertise**: Repository structure, coding standards, testing workflow, documentation requirements

---

### Domain-Specific Experts

#### [ansible-expert.md](ansible-expert.md) - Ansible Expert
**Purpose**: Ansible playbook and role development specialist

**Use when**:
- Writing or modifying Ansible playbooks
- Creating or updating roles
- Working with handlers, templates, or variables
- Implementing multi-phase pipelines
- Troubleshooting Ansible issues

**Expertise**: Ansible best practices, role design, variable management, error handling, idempotency

---

#### [devops-expert.md](devops-expert.md) - DevOps & Infrastructure Expert
**Purpose**: CI/CD, monitoring, and operational excellence specialist

**Use when**:
- Setting up or modifying CI/CD pipelines
- Configuring monitoring (Prometheus/Grafana)
- Working with Docker or Kubernetes
- Implementing deployment strategies
- Security and performance optimization
- Incident response and troubleshooting

**Expertise**: GitHub Actions, monitoring stack, container orchestration, security practices, production operations

---

#### [rust-expert.md](rust-expert.md) - Rust Expert
**Purpose**: Rust development specialist for srSILO tools

**Use when**:
- Writing or modifying Rust tools in `roles/srsilo/files/tools/`
- Performance optimization of data processing
- CLI tool development
- Error handling in Rust
- Testing Rust code

**Expertise**: Rust best practices, cargo, clippy, performance optimization, CLI development, integration with Ansible

---

#### [documentation-expert.md](documentation-expert.md) - Documentation Expert
**Purpose**: Technical documentation specialist

**Use when**:
- Creating or updating documentation
- Writing README files
- Documenting architecture decisions
- Creating troubleshooting guides
- Maintaining role documentation
- Writing API documentation

**Expertise**: Technical writing, markdown, documentation structure, examples, troubleshooting guides

---

#### [yaml-config-expert.md](yaml-config-expert.md) - YAML & Configuration Expert
**Purpose**: Configuration file and YAML specialist

**Use when**:
- Writing or modifying YAML files
- Managing Ansible variables
- Creating or updating GitHub Actions workflows
- Working with Docker Compose configurations
- Template development (Jinja2)
- Configuration file organization

**Expertise**: YAML syntax, Ansible variable management, Jinja2 templates, configuration best practices, secrets management

---

#### [software-engineering-expert.md](software-engineering-expert.md) - Software Engineering Expert
**Purpose**: General software engineering and code quality specialist

**Use when**:
- Code review
- Refactoring code
- Designing new features
- Managing technical debt
- Improving code quality
- Version control best practices

**Expertise**: SOLID principles, DRY/KISS, testing strategies, refactoring, code review, performance optimization

---

## How to Use These Agents

### With GitHub Copilot Chat

When using GitHub Copilot Chat, you can reference these agent profiles to get specialized assistance:

```
@workspace /new Using the guidance from .github/agents/ansible-expert.md, 
help me create a new Ansible role for managing backups
```

```
@workspace According to .github/agents/rust-expert.md, 
what's the best way to handle errors in this Rust tool?
```

### During Development

1. **Before starting**: Review `agent.md` for project overview
2. **While coding**: Reference the relevant expert agent for specific guidance
3. **During code review**: Use `software-engineering-expert.md` checklist
4. **When documenting**: Follow `documentation-expert.md` standards

### For Code Reviews

Reviewers can reference these agents when providing feedback:

```
Per .github/agents/ansible-expert.md, this task should be idempotent. 
Consider adding a check for existing state.
```

### For Learning

New contributors can use these agents to understand project standards:

1. Read `agent.md` for overall project understanding
2. Study domain-specific agents for areas you'll work on
3. Refer back when you have questions about best practices

## Agent Maintenance

### Updating Agents

When project standards or practices change:

1. Update the relevant agent profile(s)
2. Document the change in the agent file
3. Notify team of significant changes

### Creating New Agents

If you identify a need for a new specialized agent:

1. Create a new `.md` file in this directory
2. Follow the structure of existing agents
3. Add it to this README
4. Consider whether it should be temporary or permanent

## File Structure

```
.github/agents/
├── README.md                          # This file
├── agent.md                           # Main agent profile
├── ansible-expert.md                  # Ansible specialist
├── devops-expert.md                   # DevOps/Infrastructure specialist
├── rust-expert.md                     # Rust development specialist
├── documentation-expert.md            # Documentation specialist
├── yaml-config-expert.md             # YAML/Config specialist
└── software-engineering-expert.md     # General software engineering
```

## Quick Reference

| Task | Primary Agent | Supporting Agents |
|------|--------------|-------------------|
| New Ansible role | ansible-expert | software-engineering-expert, documentation-expert |
| Rust tool modification | rust-expert | software-engineering-expert |
| CI/CD pipeline | devops-expert | yaml-config-expert |
| Documentation update | documentation-expert | - |
| Config file changes | yaml-config-expert | ansible-expert (for Ansible configs) |
| Code review | software-engineering-expert | Domain-specific expert |
| Monitoring setup | devops-expert | yaml-config-expert |
| Playbook development | ansible-expert | yaml-config-expert |

## Best Practices

1. **Start broad, then specialize**: Begin with `agent.md`, then consult specialized agents
2. **Cross-reference**: Many tasks benefit from multiple agents' perspectives
3. **Stay updated**: Review agents periodically to stay current with standards
4. **Contribute improvements**: If you find gaps or errors, update the agents
5. **Share knowledge**: Reference agents when helping other team members

## Contributing

To improve these agent profiles:

1. Identify gaps or outdated information
2. Propose changes via pull request
3. Update related agents if changes affect multiple areas
4. Keep agents focused on their domain
5. Maintain consistent structure and style

## Questions?

If you're unsure which agent to consult:
- Start with `agent.md` for general guidance
- Check the "Quick Reference" table above
- When in doubt, multiple perspectives can help!
