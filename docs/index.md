# WisePulse

<p align="center">
  <img src="assets/wasap-logo.png" alt="WisePulse Logo" width="150">
</p>

<p align="center">
  <strong>WISE Loculus with V-Pipe – Infrastructure for start-to-end viral wastewater analysis</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/status-public%20beta-blue" alt="Status: Public Beta">
  <img src="https://img.shields.io/badge/platform-linux-lightgrey" alt="Platform: Linux">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License: MIT">
</p>

---

## About

**WisePulse** is an end-to-end infrastructure for viral wastewater surveillance, combining [Loculus](https://loculus.org) for sequence data management with [LAPIS-SILO](https://github.com/GenSpectrum/LAPIS-SILO), a high-performance genomic database – here deployed for the first time to **S**hort-**R**ead / amplicon sequences. It enables real-time querying and visualization of viral genomic data from wastewater samples.

**Demo**: [Watch on YouTube](https://www.youtube.com/watch?v=kCUd-o1FbXg&t=420) (starts at 7:00)

### Major Goals

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

#### Data Processing Pipeline (external)

Upstream data processing is handled separately:

- NGS processing via [V-pipe](https://github.com/cbg-ethz/V-pipe)
- Database ingestion with [sr2silo](https://github.com/cbg-ethz/sr2silo) from V-pipe alignment outputs

### Roadmap

- Expand to more viruses
- Raw data submission & V-pipe pipeline integration
- On-demand exploratory analysis tools

## Quick Links

- [Installation Guide](getting-started/installation.md)
- [Quick Start](getting-started/quick-start.md)
- [Architecture Overview](architecture/overview.md)
- [Configuration Reference](configuration/reference.md)

## API Documentation

| Service | Swagger UI |
|---------|-----|
| **LAPIS COVID** | [lapis.wasap.genspectrum.org/covid/swagger-ui/index.html](https://lapis.wasap.genspectrum.org/covid/swagger-ui/index.html) |
| **LAPIS RSV-A** | [lapis.wasap.genspectrum.org/rsva/swagger-ui/index.html](https://lapis.wasap.genspectrum.org/rsva/swagger-ui/index.html) |
