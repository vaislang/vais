#!/usr/bin/env python3
"""Normalize source-position-derived LLVM globals for stage IR comparison."""

from __future__ import annotations

import re
import sys
from pathlib import Path


GLOBAL_RE = re.compile(r"@\.(?:s|fmt)\d+")


def normalize(text: str) -> str:
    names: dict[str, str] = {}
    counts = {".s": 0, ".fmt": 0}

    def replace(match: re.Match[str]) -> str:
        key = match.group(0)
        if key not in names:
            prefix = ".fmt" if key.startswith("@.fmt") else ".s"
            names[key] = f"@{prefix}$N{counts[prefix]}"
            counts[prefix] += 1
        return names[key]

    return GLOBAL_RE.sub(replace, text)


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: normalize_stage_ir.py IN.ll OUT.ll", file=sys.stderr)
        return 2
    src = Path(sys.argv[1])
    dst = Path(sys.argv[2])
    dst.write_text(normalize(src.read_text()))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
