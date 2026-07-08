import logging
import subprocess
import time

import requests

from pipeline.config import VirusConfig, VirusPaths

log = logging.getLogger(__name__)

_HEALTH_RETRIES = 12
_HEALTH_DELAY = 5


def start(virus_name: str, virus: VirusConfig, paths: VirusPaths) -> None:
    log.info("Starting SILO API for %s", virus_name)
    subprocess.run(
        ["docker", "compose", "-p", virus_name, "up", "-d"],
        cwd=paths.config,
        check=True,
    )
    _wait_for_ready(virus.lapis_port, virus_name)


def _wait_for_ready(port: int, virus_name: str) -> None:
    url = f"http://localhost:{port}/sample/info"
    for attempt in range(1, _HEALTH_RETRIES + 1):
        try:
            r = requests.get(url, timeout=10)
            if r.status_code == 200:
                log.info("API for %s is ready", virus_name)
                return
        except requests.RequestException:
            pass
        log.info("Waiting for API (%s) — attempt %d/%d", virus_name, attempt, _HEALTH_RETRIES)
        time.sleep(_HEALTH_DELAY)
    log.warning("API for %s did not become healthy within timeout — continuing anyway", virus_name)
