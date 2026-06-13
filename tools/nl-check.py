#!/usr/bin/env python3
"""Compatibility wrapper for the old nl-check command.

Use tools/vais-check.py for the canonical New Vais lint command.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path


TARGET = Path(__file__).with_name("vais-check.py")
SPEC = importlib.util.spec_from_file_location("vais_check", TARGET)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"could not load {TARGET}")
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


if __name__ == "__main__":
    raise SystemExit(MODULE.main())
