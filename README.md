# WisePulse

<div align="center">

<img src="roles/loculus/files/images/wasap-logo.png" alt="WisePulse Logo" width="150"/>

### WISE Loculus with V-Pipe – Infrastructure for start-to-end viral wastewater analysis

![Status: Public Beta](https://img.shields.io/badge/status-public%20beta-blue)
![Platform: Linux](https://img.shields.io/badge/platform-linux-lightgrey)
![License: MIT](https://img.shields.io/badge/license-MIT-green)

</div>

---

## About

**WisePulse** is an end-to-end infrastructure for viral wastewater surveillance, combining [Loculus](https://loculus.org) for sequence data management with [LAPIS-SILO](https://github.com/GenSpectrum/LAPIS-SILO), a high-performance genomic database – here deployed for the first time to **S**hort-**R**ead / amplicon sequences. It enables real-time querying and visualization of viral genomic data from wastewater samples.

Built with [Ansible](https://www.ansible.com/) for infrastructure-as-code, enabling quick redeployment to other institutions and environments.

**Demo**: [Watch on YouTube](https://www.youtube.com/watch?v=kCUd-o1FbXg&t=420) (starts at 7:00)

#### Major Goals

- **Cluster to Browser**: Brought downstream exploratory analysis from exclusive, high barrier-to-entry cluster-based workflows to user-friendly, simple browser-based workflows

- **Days to Minutes**: Simplified exploratory analysis beyond the regular processing reports produced by the WISE Consortium from days waiting for requests served by few with data access to self-serving

- **Expert-only to Community Access**: Direct access to data allows for community access—not just those with data access and expert bioinformatics knowledge—to explore, catering for virologists and phylogeneticists

- **Isolated to Integrated**: First time wastewater data can be analyzed on the fly with the latest clinical variant definitions without manual work

### Current Features
- Loculus instance for easy data sharing and collaboration
- Multi-virus srSILO API for querying amplicon sequence data (SARS-CoV-2, RSV-A)

### Live Instances
- **Mutational Analysis Dashboards**: [genspectrum.org/swiss-wastewater](https://genspectrum.org/swiss-wastewater)
- **Swiss Wastewater Viral Alignment Database**: [db.wasap.genspectrum.org](https://db.wasap.genspectrum.org/)
- **API Swagger UI**: [COVID](https://lapis.wasap.genspectrum.org/covid/swagger-ui/index.html) | [RSV-A](https://lapis.wasap.genspectrum.org/rsva/swagger-ui/index.html)

#### Data Processing Pipeline (external)
Upstream data processing is handled separately:
- NGS processing via [V-pipe](https://github.com/cbg-ethz/V-pipe)
- Database ingestion with [sr2silo](https://github.com/cbg-ethz/sr2silo) from V-pipe alignment outputs

### Roadmap
- Expand to more viruses
- Raw data submission & V-pipe pipeline integration
- On-demand exploratory analysis tools

## Config structure

TODO - this needs to be documented.
I'd like to see a section about: "This is how you set up your config with your viruses etc. And now run the setup script."

## Quick Start

```bash
# One-time setup
ansible-playbook playbooks/srsilo/setup.yml -i inventory.ini --become --ask-become-pass

# Run pipeline
ansible-playbook playbooks/srsilo/update-all-viruses.yml -i inventory.ini --become --ask-become-pass
```

## Documentation

See the **[full documentation](https://cbg-ethz.github.io/WisePulse/)** for installation, configuration, and operations guides.
