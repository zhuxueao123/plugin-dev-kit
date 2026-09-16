#!/usr/bin/env python3
"""Verify the local AsapFlow Plugin Dev Kit development environment."""
from __future__ import annotations

import json
import sys
from pathlib import Path

KIT_ROOT = Path(__file__).resolve().parent
WORKSPACE = KIT_ROOT / "plugin-workspace"
SDK_ROOT = WORKSPACE / "runtime-sdk"


def require(relative_path: str) -> Path:
    path = KIT_ROOT / relative_path
    if not path.exists():
        raise RuntimeError(f"missing required Dev Kit file: {relative_path}")
    return path


def verify_manifests() -> None:
    backend_plugins = ("supplier_guard", "supplier_runtime_rules")
    for plugin_code in backend_plugins:
        manifest_path = require(
            f"plugin-workspace/examples/backend/{plugin_code}/manifest.json"
        )
        manifest = json.loads(manifest_path.read_text(encoding="utf-8-sig"))
        if manifest.get("pluginCode") != plugin_code:
            raise RuntimeError(f"unexpected pluginCode in {manifest_path}")
        require(f"plugin-workspace/examples/backend/{plugin_code}/main.py")

    frontend_manifest_path = require(
        "plugin-workspace/examples/frontend/supplier_portal/manifest.json"
    )
    frontend_manifest = json.loads(
        frontend_manifest_path.read_text(encoding="utf-8-sig")
    )
    if frontend_manifest.get("pluginCode") != "supplier_portal":
        raise RuntimeError(f"unexpected pluginCode in {frontend_manifest_path}")
    for page in frontend_manifest.get("pages", []):
        require(
            "plugin-workspace/examples/frontend/supplier_portal/"
            + page["entry"]
        )


def verify_sdk() -> None:
    sys.path.insert(0, str(SDK_ROOT))
    import asap_runtime
    from asap_runtime import PluginContext, wrap_model
    from plugins_sdk import ExecutionResultBuilder, PluginContextFactory

    if asap_runtime.PluginContext is not PluginContext:
        raise RuntimeError("asap_runtime public import smoke check failed")
    if wrap_model({"ready": True}).ready is not True:
        raise RuntimeError("asap_runtime model import smoke check failed")

    context = PluginContextFactory.from_fixture(
        datasets={"order": {"amount": 100}}
    )
    if context.model.amount != 100:
        raise RuntimeError("PluginContextFactory smoke check failed")
    result = ExecutionResultBuilder().status("ok").payload({"ready": True}).build()
    if result.status != "ok" or result.payload != {"ready": True}:
        raise RuntimeError("ExecutionResultBuilder smoke check failed")


def main() -> None:
    verify_manifests()
    verify_sdk()
    print("Plugin Dev Kit verification passed.")


if __name__ == "__main__":
    main()
