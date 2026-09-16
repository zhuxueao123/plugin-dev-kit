"""Plugin i18n support for runtime-executed plug-ins."""
from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Optional


def _flatten_messages(prefix: str, value: Any, target: Dict[str, str]) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            next_prefix = f"{prefix}.{key}" if prefix else str(key)
            _flatten_messages(next_prefix, child, target)
        return
    if value is None:
        return
    target[prefix] = str(value)


def _lookup_nested(mapping: Dict[str, Any], dotted_key: str) -> Optional[Any]:
    if dotted_key in mapping:
        return mapping[dotted_key]
    current: Any = mapping
    for part in dotted_key.split("."):
        if not isinstance(current, dict) or part not in current:
            return None
        current = current[part]
    return current


class PluginI18nCatalog:
    """Loads locale dictionaries from plugins/locales."""

    def __init__(self, locales_dir: Path, fallback_locale: str = "zh-CN") -> None:
        self._locales_dir = Path(locales_dir)
        self._fallback_locale = fallback_locale
        self._messages: Dict[str, Dict[str, Dict[str, str]]] = {}
        self.reload()

    @property
    def fallback_locale(self) -> str:
        return self._fallback_locale

    def reload(self) -> None:
        messages: Dict[str, Dict[str, Dict[str, str]]] = {}
        if self._locales_dir.exists():
            for path in sorted(self._locales_dir.glob("*.json")):
                locale = path.stem
                payload = json.loads(path.read_text(encoding="utf-8-sig"))
                locale_messages: Dict[str, Dict[str, str]] = {}
                if isinstance(payload, dict):
                    for plugin_code, plugin_payload in payload.items():
                        if not isinstance(plugin_payload, dict):
                            continue
                        flattened: Dict[str, str] = {}
                        _flatten_messages("", plugin_payload, flattened)
                        locale_messages[str(plugin_code)] = flattened
                messages[locale] = locale_messages
        self._messages = messages

    def available_locales(self, plugin_code: Optional[str] = None) -> list[str]:
        if not plugin_code:
            return sorted(self._messages.keys())
        result = []
        for locale, locale_payload in self._messages.items():
            if plugin_code in locale_payload:
                result.append(locale)
        return sorted(result)

    def translate(
        self,
        plugin_code: str,
        key: str,
        *,
        locale: Optional[str] = None,
        params: Optional[Dict[str, Any]] = None,
        default: Optional[str] = None,
        fallback_locale: Optional[str] = None,
    ) -> str:
        normalized_key = str(key or "").strip()
        if not normalized_key:
            return default or ""

        preferred_locale = str(locale or "").strip() or None
        fallback = str(fallback_locale or self._fallback_locale or "").strip() or None
        locale_chain = []
        if preferred_locale:
            locale_chain.append(preferred_locale)
        if fallback and fallback not in locale_chain:
            locale_chain.append(fallback)

        plugin_namespace, scoped_key = self._split_key(plugin_code, normalized_key)

        for candidate_locale in locale_chain:
            locale_payload = self._messages.get(candidate_locale) or {}
            plugin_payload = locale_payload.get(plugin_namespace) or {}
            raw = plugin_payload.get(scoped_key)
            if raw is None:
                raw = plugin_payload.get(normalized_key)
            if raw is not None:
                return self._format(raw, params)

        return self._format(default if default is not None else normalized_key, params)

    def exists(self, plugin_code: str, key: str, *, locale: Optional[str] = None) -> bool:
        normalized_key = str(key or "").strip()
        if not normalized_key:
            return False
        plugin_namespace, scoped_key = self._split_key(plugin_code, normalized_key)
        locales = [str(locale).strip()] if locale else list(self._messages.keys())
        for candidate_locale in locales:
            locale_payload = self._messages.get(candidate_locale) or {}
            plugin_payload = locale_payload.get(plugin_namespace) or {}
            if scoped_key in plugin_payload or normalized_key in plugin_payload:
                return True
        return False

    @staticmethod
    def _split_key(plugin_code: str, key: str) -> tuple[str, str]:
        prefix = f"{plugin_code}."
        if key.startswith(prefix):
            return plugin_code, key[len(prefix):]
        return plugin_code, key

    @staticmethod
    def _format(template: str, params: Optional[Dict[str, Any]]) -> str:
        if not params:
            return template
        try:
            return template.format(**params)
        except Exception:
            return template


@dataclass
class PluginTranslator:
    """Bound translator exposed to plugin code through context."""

    catalog: PluginI18nCatalog
    plugin_code: str
    locale: Optional[str] = None
    fallback_locale: str = "zh-CN"

    def t(
        self,
        key: str,
        params: Optional[Dict[str, Any]] = None,
        default: Optional[str] = None,
        locale: Optional[str] = None,
    ) -> str:
        return self.catalog.translate(
            self.plugin_code,
            key,
            locale=locale or self.locale,
            params=params,
            default=default,
            fallback_locale=self.fallback_locale,
        )

    def exists(self, key: str, locale: Optional[str] = None) -> bool:
        return self.catalog.exists(self.plugin_code, key, locale=locale or self.locale)

    @property
    def available_locales(self) -> list[str]:
        return self.catalog.available_locales(self.plugin_code)
