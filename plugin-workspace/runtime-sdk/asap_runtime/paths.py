"""Common filesystem paths used by runtime."""
from __future__ import annotations

import os
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
RUNTIME_DIR = BASE_DIR.parent
REPO_ROOT = RUNTIME_DIR.parent


def _resolve_plugins_dir() -> Path:
    """Locate the plugins directory with environment override support."""

    env_dir = os.environ.get("ASAP_RUNTIME_PLUGINS_DIR")
    if env_dir:
        return Path(env_dir).expanduser().resolve()

    search_roots = [REPO_ROOT, RUNTIME_DIR]
    for root in search_roots:
        candidate = (root / "plugins").resolve()
        if candidate.exists():
            return candidate

    # Fallback to repository-level assumption even if the directory is not present yet.
    return (search_roots[0] / "plugins").resolve()


PLUGINS_DIR = _resolve_plugins_dir()
PLUGIN_BACKEND_DIR = PLUGINS_DIR / "backend"
PLUGIN_FRONTEND_DIR = PLUGINS_DIR / "frontend"
CACHE_DIR = BASE_DIR / "cache"
CONFIG_CACHE_DIR = CACHE_DIR / "config"


def plugin_backend_manifest(plugin_code: str) -> Path:
    return PLUGIN_BACKEND_DIR / plugin_code / "manifest.json"


def plugin_frontend_manifest(plugin_code: str) -> Path:
    return PLUGIN_FRONTEND_DIR / plugin_code / "manifest.json"
