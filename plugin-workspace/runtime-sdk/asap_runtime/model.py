"""Lightweight wrappers for dict/list models with attribute access."""
from __future__ import annotations

from typing import Any, Iterator


def _unwrap(value: Any) -> Any:
    if isinstance(value, AttrModel):
        return value.to_dict()
    if isinstance(value, AttrList):
        return value.to_list()
    return value


def wrap_model(value: Any) -> Any:
    if isinstance(value, (AttrModel, AttrList)):
        return value
    if isinstance(value, dict):
        return AttrModel(value)
    if isinstance(value, list):
        return AttrList(value)
    return value


class AttrModel:
    """Dict wrapper that returns None for missing attributes."""

    def __init__(self, data: dict) -> None:
        object.__setattr__(self, "_data", data)

    def __getattr__(self, name: str) -> Any:
        data = object.__getattribute__(self, "_data")
        if name in data:
            return wrap_model(data[name])
        return None

    def __setattr__(self, name: str, value: Any) -> None:
        data = object.__getattribute__(self, "_data")
        data[name] = _unwrap(value)

    def __getitem__(self, key: str) -> Any:
        data = object.__getattribute__(self, "_data")
        if key in data:
            return wrap_model(data[key])
        return None

    def __setitem__(self, key: str, value: Any) -> None:
        data = object.__getattribute__(self, "_data")
        data[key] = _unwrap(value)

    def get(self, key: str, default: Any = None) -> Any:
        data = object.__getattribute__(self, "_data")
        if key in data:
            return wrap_model(data[key])
        return default

    def to_dict(self) -> dict:
        return object.__getattribute__(self, "_data")

    def items(self):
        return self.to_dict().items()

    def keys(self):
        return self.to_dict().keys()

    def values(self):
        return self.to_dict().values()

    def __repr__(self) -> str:
        return f"AttrModel({self.to_dict()!r})"


class AttrList:
    """List wrapper that returns wrapped elements."""

    def __init__(self, data: list) -> None:
        object.__setattr__(self, "_data", data)

    def __getitem__(self, index: int) -> Any:
        data = object.__getattribute__(self, "_data")
        return wrap_model(data[index])

    def __setitem__(self, index: int, value: Any) -> None:
        data = object.__getattribute__(self, "_data")
        data[index] = _unwrap(value)

    def __len__(self) -> int:
        data = object.__getattribute__(self, "_data")
        return len(data)

    def __iter__(self) -> Iterator[Any]:
        data = object.__getattribute__(self, "_data")
        for item in data:
            yield wrap_model(item)

    def append(self, value: Any) -> None:
        data = object.__getattribute__(self, "_data")
        data.append(_unwrap(value))

    def extend(self, values: list) -> None:
        data = object.__getattribute__(self, "_data")
        data.extend(_unwrap(value) for value in values)

    def to_list(self) -> list:
        return object.__getattribute__(self, "_data")

    def __repr__(self) -> str:
        return f"AttrList({self.to_list()!r})"
