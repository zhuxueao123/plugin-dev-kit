"""Plugin application helper for registering capability handlers."""
from __future__ import annotations

from typing import Any, Callable, Dict, Union

from .context import PluginContext
from .result import ExecutionResult

CapabilityHandler = Callable[
    [PluginContext, bytes],
    Union[ExecutionResult, Dict[str, Any], bytes, str, None],
]


class PluginApp:
    """Lightweight registry used by plug-in authors to register handlers."""

    def __init__(self) -> None:
        self._handlers: Dict[str, CapabilityHandler] = {}

    def capability(self, capability_code: str) -> Callable[[CapabilityHandler], CapabilityHandler]:
        """Decorator used to register capability handlers."""

        def decorator(func: CapabilityHandler) -> CapabilityHandler:
            if capability_code in self._handlers:
                raise ValueError(f"Duplicate capability registration for '{capability_code}'")
            self._handlers[capability_code] = func
            return func

        return decorator

    @property
    def handlers(self) -> Dict[str, CapabilityHandler]:
        return dict(self._handlers)
