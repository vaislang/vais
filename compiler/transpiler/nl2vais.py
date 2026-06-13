#!/usr/bin/env python3
"""Compatibility wrapper for the legacy New Vais bootstrap adapter."""

from __future__ import annotations

import importlib.util
from pathlib import Path


_TARGET = Path(__file__).with_name("legacy_vais_bootstrap.py")
_SPEC = importlib.util.spec_from_file_location("legacy_vais_bootstrap", _TARGET)
if _SPEC is None or _SPEC.loader is None:
    raise ImportError(f"cannot load {_TARGET}")

_MODULE = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(_MODULE)

for _name, _value in vars(_MODULE).items():
    if not _name.startswith("__"):
        globals()[_name] = _value


if __name__ == "__main__":
    _MODULE.main()
