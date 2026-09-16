"""Execution result helpers for plug-in authors."""
from __future__ import annotations

from typing import Any, Dict, Optional

from asap_runtime.context import ExecutionResult


class ExecutionResultBuilder:
    """Mutable builder used when handlers need to append metadata step by step."""

    def __init__(self) -> None:
        self._status = "ok"
        self._payload: Any = None
        self._error_message: Optional[str] = None
        self._metadata: Dict[str, Any] = {}

    def status(self, value: str) -> "ExecutionResultBuilder":
        self._status = value
        return self

    def payload(self, value: Any) -> "ExecutionResultBuilder":
        self._payload = value
        return self

    def error(self, message: str) -> "ExecutionResultBuilder":
        self._error_message = message
        return self

    def metadata(self, **entries: Any) -> "ExecutionResultBuilder":
        self._metadata.update(entries)
        return self

    def build(self) -> ExecutionResult:
        result = ExecutionResult(status=self._status, payload=self._payload, error_message=self._error_message)
        if self._metadata:
            if isinstance(result.payload, dict):
                combined = dict(result.payload)
                combined.update(self._metadata)
                result.payload = combined
            elif result.payload is None:
                result.payload = dict(self._metadata)
        return result


def ok(payload: Any = None) -> ExecutionResult:
    return ExecutionResult(status="ok", payload=payload)


def reject(message: str, payload: Any = None) -> ExecutionResult:
    return ExecutionResult(status="reject", payload=payload, error_message=message)


def error(message: str, payload: Any = None, *, status: str = "failed") -> ExecutionResult:
    return ExecutionResult(status=status, payload=payload, error_message=message)


__all__ = [
    "ExecutionResult",
    "ExecutionResultBuilder",
    "ok",
    "reject",
    "error",
]
