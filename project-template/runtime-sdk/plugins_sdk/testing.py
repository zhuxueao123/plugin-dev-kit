"""Testing helpers for plug-in authors."""
from __future__ import annotations

from pathlib import Path
from typing import Any, Dict, Optional

from asap_runtime import paths
from .context import PluginContext, ContextData, ContextMeta, TenantInfo, UserInfo, FeatureInfo, ScenarioInfo, ActionInfo
from asap_runtime.plugin_i18n import PluginI18nCatalog, PluginTranslator


class PluginContextFactory:
    """Factory helpers to create PluginContext instances for unit tests."""

    @staticmethod
    def from_fixture(
        *,
        plugin_code: str = "test-plugin",
        capability_code: str = "default",
        request_id: str = "req-test",
        meta: Optional[ContextMeta] = None,
        params: Optional[Dict[str, Any]] = None,
        datasets: Optional[Dict[str, Any]] = None,
        core: Optional[Any] = None,
        locale: Optional[str] = None,
    ) -> PluginContext:
        context_meta = meta or ContextMeta(
            tenant=TenantInfo(id="tenant-test"),
            user=UserInfo(id="user-test"),
            feature=FeatureInfo(code="feature"),
            scenario=ScenarioInfo(code="scenario"),
            action=ActionInfo(code="action"),
        )
        if locale:
            context_meta.invocation.locale = locale
        data_section = ContextData()
        if datasets:
            for name, value in datasets.items():
                data_section.add_dataset(name, value)
            if datasets:
                first_key = next(iter(datasets.keys()))
                data_section.set_root_name(first_key)
        translator = None
        locales_dir = paths.PLUGINS_DIR / "locales"
        if not locales_dir.exists():
            # In the standalone dev kit, runtime-sdk and locales are siblings
            # below plugin-workspace instead of living under a runtime tree.
            workspace_locales = Path(__file__).resolve().parents[2] / "locales"
            if workspace_locales.exists():
                locales_dir = workspace_locales
        if locales_dir.exists():
            translator = PluginTranslator(
                PluginI18nCatalog(locales_dir),
                plugin_code,
                locale=context_meta.invocation.locale,
            )
        context = PluginContext(
            request_id=request_id,
            plugin_code=plugin_code,
            capability_code=capability_code,
            meta=context_meta,
            data=data_section,
            params=dict(params or {}),
            core=core,
            i18n=translator,
        )
        return context


__all__ = ["PluginContextFactory"]
