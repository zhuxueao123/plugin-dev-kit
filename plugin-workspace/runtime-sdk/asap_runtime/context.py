"""Execution context models shared with plug-ins."""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Callable, Dict, Iterable, Iterator, Optional

from .model import wrap_model
from . import paths
from .plugin_i18n import PluginI18nCatalog, PluginTranslator

@dataclass
class TenantInfo:
    id: Optional[str] = None
    code: Optional[str] = None
    name: Optional[str] = None
    extra: Dict[str, Any] = field(default_factory=dict)


@dataclass
class UserInfo:
    id: Optional[str] = None
    account: Optional[str] = None
    display_name: Optional[str] = None
    roles: list[str] = field(default_factory=list)
    permissions: list[str] = field(default_factory=list)
    extra: Dict[str, Any] = field(default_factory=dict)


@dataclass
class FeatureInfo:
    code: Optional[str] = None
    name: Optional[str] = None
    entity_code: Optional[str] = None
    extra: Dict[str, Any] = field(default_factory=dict)


@dataclass
class ScenarioInfo:
    code: Optional[str] = None
    scenario_type: Optional[str] = None
    mode: Optional[str] = None
    extra: Dict[str, Any] = field(default_factory=dict)


@dataclass
class ActionInfo:
    code: Optional[str] = None
    display_name: Optional[str] = None
    action_type: Optional[str] = None
    extra: Dict[str, Any] = field(default_factory=dict)


@dataclass
class SelectionInfo:
    row: Any = None
    rows: list[Any] = field(default_factory=list)
    ids: list[str] = field(default_factory=list)
    extra: Dict[str, Any] = field(default_factory=dict)

    @property
    def count(self) -> int:
        return len(self.rows)

    @property
    def has_selection(self) -> bool:
        return self.count > 0


@dataclass
class InvocationInfo:
    trace_id: Optional[str] = None
    span_id: Optional[str] = None
    correlation_id: Optional[str] = None
    channel: Optional[str] = None
    locale: Optional[str] = None
    timezone: Optional[str] = None
    environment: Optional[str] = None
    extra: Dict[str, Any] = field(default_factory=dict)


@dataclass
class ContextMeta:
    tenant: TenantInfo = field(default_factory=TenantInfo)
    user: UserInfo = field(default_factory=UserInfo)
    feature: FeatureInfo = field(default_factory=FeatureInfo)
    scenario: ScenarioInfo = field(default_factory=ScenarioInfo)
    action: ActionInfo = field(default_factory=ActionInfo)
    invocation: InvocationInfo = field(default_factory=InvocationInfo)
    tags: Dict[str, Any] = field(default_factory=dict)


class ConfigView:
    """Lazy configuration accessor exposed to plug-ins."""

    def __init__(self, loader: Optional[callable] = None, snapshot: Optional[Dict[str, Any]] = None) -> None:
        self._loader = loader
        self._snapshot = snapshot or {}

    def bind_loader(self, loader: Callable[[], Dict[str, Any]]) -> None:
        self._loader = loader

    def set_snapshot(self, snapshot: Dict[str, Any]) -> None:
        self._snapshot = dict(snapshot)

    def _ensure_loaded(self) -> None:
        if self._loader and not self._snapshot:
            self._snapshot = dict(self._loader())

    def get(self, key: str, default: Any = None) -> Any:
        self._ensure_loaded()
        return self._snapshot.get(key, default)

    def require(self, key: str) -> Any:
        self._ensure_loaded()
        if key not in self._snapshot:
            raise KeyError(f"Configuration key '{key}' not found")
        return self._snapshot[key]

    def as_dict(self) -> Dict[str, Any]:
        self._ensure_loaded()
        return dict(self._snapshot)


class ContextData:
    """Mutable data section shared with plug-ins."""

    def __init__(
        self,
        *,
        root_name: Optional[str] = None,
        datasets: Optional[Dict[str, Any]] = None,
        descriptors: Optional[Dict[str, Any]] = None,
        raw_blobs: Optional[Dict[str, bytes]] = None,
    ) -> None:
        self._root_name = root_name
        self._datasets: Dict[str, Any] = datasets or {}
        self._descriptors = descriptors or {}
        self._raw_blobs = raw_blobs or {}

    def set_root_name(self, name: Optional[str]) -> None:
        self._root_name = name

    def add_dataset(self, name: str, data: Any, descriptor: Optional[Any] = None, raw: Optional[bytes] = None) -> None:
        self._datasets[name] = data
        if descriptor is not None:
            self._descriptors[name] = descriptor
        if raw is not None:
            self._raw_blobs[name] = raw

    @property
    def root(self) -> Any:
        if not self._root_name:
            raise KeyError("root dataset not specified")
        if self._root_name not in self._datasets:
            raise KeyError(f"dataset '{self._root_name}' not found")
        return self._datasets[self._root_name]

    @property
    def collections(self) -> Dict[str, Any]:
        return {name: value for name, value in self._datasets.items() if isinstance(value, list)}

    def get(self, name: str, default: Any = None) -> Any:
        return self._datasets.get(name, default)

    def iter_datasets(self) -> Iterable[tuple[str, Any]]:
        return self._datasets.items()

    def stream(self, name: str) -> Iterator[Any]:
        if name in self._raw_blobs:
            yield self._raw_blobs[name]
            return
        if name not in self._datasets:
            raise KeyError(f"dataset '{name}' not found")
        value = self._datasets[name]
        if isinstance(value, list):
            for item in value:
                yield item
        else:
            yield value


class TransportContext:
    """Transport-level helpers (deadline, cancellation placeholders)."""

    def __init__(self, deadline: Optional[float] = None, cancelled: Optional[callable] = None) -> None:
        self._deadline = deadline
        self._cancelled = cancelled

    @property
    def deadline(self) -> Optional[float]:
        return self._deadline

    def is_cancelled(self) -> bool:
        if self._cancelled:
            try:
                return bool(self._cancelled())
            except Exception:  # pragma: no cover - defensive
                return False
        return False


@dataclass
class PluginContext:
    request_id: str
    plugin_code: str
    capability_code: str
    meta: ContextMeta = field(default_factory=ContextMeta)
    config: ConfigView = field(default_factory=ConfigView)
    data: ContextData = field(default_factory=ContextData)
    selection: SelectionInfo = field(default_factory=SelectionInfo)
    params: Dict[str, Any] = field(default_factory=dict)
    metadata: Dict[str, Any] = field(default_factory=dict)
    raw_context: Dict[str, Any] = field(default_factory=dict)
    core: Any | None = None
    i18n: Any | None = None
    transport: TransportContext = field(default_factory=TransportContext)

    @property
    def tenant(self) -> TenantInfo:
        return self.meta.tenant

    @property
    def user(self) -> UserInfo:
        return self.meta.user

    @property
    def feature(self) -> FeatureInfo:
        return self.meta.feature

    @property
    def scenario(self) -> ScenarioInfo:
        return self.meta.scenario

    @property
    def action(self) -> ActionInfo:
        return self.meta.action

    @property
    def invocation(self) -> InvocationInfo:
        return self.meta.invocation

    @property
    def locale(self) -> Optional[str]:
        return self.meta.invocation.locale

    @property
    def available_locales(self) -> list[str]:
        if self.i18n is None:
            return []
        return list(getattr(self.i18n, "available_locales", []) or [])

    @property
    def model(self) -> Any:
        try:
            root = self.data.root
        except KeyError:
            return None
        return wrap_model(root)

    def t(self, key: str, params: Optional[Dict[str, Any]] = None, default: Optional[str] = None) -> str:
        if self.i18n is None:
            if default is not None:
                return default
            return key
        return self.i18n.t(key, params=params, default=default, locale=self.locale)


class ExecutionContext(PluginContext):
    """Backward-compatible initializer for legacy plug-in handlers."""

    def __init__(
        self,
        *,
        request_id: str,
        plugin_code: str,
        capability_code: str,
        tenant: Optional[TenantInfo] = None,
        user: Optional[UserInfo] = None,
        feature: Optional[FeatureInfo] = None,
        scenario: Optional[ScenarioInfo] = None,
        action: Optional[ActionInfo] = None,
        meta: Optional[ContextMeta] = None,
        params: Optional[Dict[str, Any]] = None,
        metadata: Optional[Dict[str, Any]] = None,
        raw_context: Optional[Dict[str, Any]] = None,
        config: Optional[ConfigView] = None,
        data: Optional[ContextData] = None,
        selection: Optional[SelectionInfo] = None,
        core: Optional[Any] = None,
        i18n: Optional[Any] = None,
        transport: Optional[TransportContext] = None,
    ) -> None:
        meta_obj = meta or ContextMeta(
            tenant=tenant or TenantInfo(),
            user=user or UserInfo(),
            feature=feature or FeatureInfo(),
            scenario=scenario or ScenarioInfo(),
            action=action or ActionInfo(),
        )
        super().__init__(
            request_id=request_id,
            plugin_code=plugin_code,
            capability_code=capability_code,
            meta=meta_obj,
            config=config or ConfigView(),
            data=data or ContextData(),
            selection=selection or SelectionInfo(),
            params=params or {},
            metadata=metadata or {},
            raw_context=raw_context or {},
            core=core,
            i18n=i18n or self._create_default_translator(plugin_code, meta_obj.invocation.locale),
            transport=transport or TransportContext(),
        )

    @staticmethod
    def _create_default_translator(plugin_code: str, locale: Optional[str]) -> Any | None:
        locales_dir = paths.PLUGINS_DIR / "locales"
        if not locales_dir.exists():
            return None
        return PluginTranslator(
            PluginI18nCatalog(locales_dir),
            plugin_code,
            locale=locale,
        )


@dataclass
class ExecutionResult:
    """Standardized execution result structure."""

    status: str = "ok"
    payload: Any = None
    error_message: Optional[str] = None

    def to_response_bytes(self) -> bytes:
        if self.payload is None:
            return b""
        if isinstance(self.payload, (bytes, bytearray)):
            return bytes(self.payload)
        if isinstance(self.payload, str):
            return self.payload.encode("utf-8")

        import json

        return json.dumps(self.payload, ensure_ascii=False).encode("utf-8")
