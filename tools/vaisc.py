#!/usr/bin/env python3
"""New Vais compiler CLI bootstrap wrapper.

This is the user-facing `vaisc` command contract for New Vais. During the
transition it bootstraps through Legacy Vais, but the emitted LLVM IR is produced
by the New Vais self-host compiler in compiler/self/fixpoint_full.nl.
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FIXPOINT_FULL = ROOT / "compiler" / "self" / "fixpoint_full.nl"
EMBED_SELF_SOURCE = ROOT / "tools" / "embed_self_source.py"
TRANSPILER = ROOT / "compiler" / "transpiler" / "nl2vais.py"
DEFAULT_LEGACY_ROOT = Path("/Users/sswoo/study/projects/vais/compiler")


class CompileError(RuntimeError):
    pass


def run_checked(
    cmd: list[str],
    *,
    cwd: Path | None = None,
    stdout=None,
    stderr=None,
) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        cmd,
        cwd=str(cwd) if cwd else None,
        text=True,
        stdout=stdout,
        stderr=stderr,
    )
    if proc.returncode != 0:
        raise CompileError(
            f"command failed ({proc.returncode}): {' '.join(cmd)}"
        )
    return proc


def resolve_legacy_vaisc(legacy_root: Path) -> str:
    override = os.environ.get("LEGACY_VAISC")
    candidates: list[Path] = []
    if override:
        candidates.append(Path(override))
    candidates.extend(
        [
            legacy_root / "target" / "debug" / "vaisc",
            legacy_root / "target" / "release" / "vaisc",
            DEFAULT_LEGACY_ROOT / "target" / "debug" / "vaisc",
            DEFAULT_LEGACY_ROOT / "target" / "release" / "vaisc",
        ]
    )
    for candidate in candidates:
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return str(candidate)

    found = shutil.which("vaisc")
    if found:
        resolved = Path(found).resolve()
        repo_wrappers = {
            (ROOT / "scripts" / "vaisc").resolve(),
            Path(__file__).resolve(),
        }
        if resolved not in repo_wrappers:
            return found

    raise CompileError(
        "could not find Legacy Vais bootstrap compiler. "
        "Set LEGACY_VAISC=/path/to/legacy/vaisc or build "
        "/Users/sswoo/study/projects/vais/compiler."
    )


def tmpdir_from_args(args: argparse.Namespace) -> tuple[Path, tempfile.TemporaryDirectory[str] | None]:
    if args.keep_tmp:
        path = Path(tempfile.mkdtemp(prefix="vaisc-"))
        print(f"debug: keeping temp dir {path}", file=sys.stderr)
        return path, None
    holder = tempfile.TemporaryDirectory(prefix="vaisc-")
    return Path(holder.name), holder


def bootstrap_emit_ir(source: Path, ir_out: Path | None, args: argparse.Namespace) -> str | None:
    if not source.is_file():
        raise CompileError(f"source not found: {source}")

    tmp, holder = tmpdir_from_args(args)
    try:
        harness = tmp / "compiler_harness.nl"
        harness_vais = tmp / "compiler_harness.vais"
        stage0 = tmp / "stage0-vaisc"
        transpile_err = tmp / "transpile.err"
        build_log = tmp / "legacy-build.log"

        run_checked(
            [
                sys.executable,
                str(EMBED_SELF_SOURCE),
                str(FIXPOINT_FULL),
                str(source),
                str(harness),
            ]
        )
        with harness_vais.open("w") as out, transpile_err.open("w") as err:
            run_checked([sys.executable, str(TRANSPILER), str(harness)], stdout=out, stderr=err)

        legacy_root = Path(args.legacy_root).resolve()
        legacy_vaisc = resolve_legacy_vaisc(legacy_root)
        shutil.rmtree("/tmp/.vais-cache", ignore_errors=True)
        with build_log.open("w") as log:
            run_checked(
                [legacy_vaisc, "build", str(harness_vais), "-o", str(stage0)],
                cwd=legacy_root,
                stdout=log,
                stderr=subprocess.STDOUT,
            )

        if ir_out is None:
            proc = subprocess.run([str(stage0)], text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            if proc.returncode != 0:
                raise CompileError(f"generated New Vais compiler exited {proc.returncode}: {proc.stderr}")
            return proc.stdout

        ir_out.parent.mkdir(parents=True, exist_ok=True)
        with ir_out.open("w") as out:
            proc = subprocess.run([str(stage0)], text=True, stdout=out, stderr=subprocess.PIPE)
        if proc.returncode != 0:
            raise CompileError(f"generated New Vais compiler exited {proc.returncode}: {proc.stderr}")
        return None
    finally:
        if holder is not None:
            holder.cleanup()


def emit_ir(args: argparse.Namespace) -> int:
    source = Path(args.source).resolve()
    if args.output == "-":
        text = bootstrap_emit_ir(source, None, args)
        assert text is not None
        sys.stdout.write(text)
        return 0

    bootstrap_emit_ir(source, Path(args.output).resolve(), args)
    return 0


def build(args: argparse.Namespace) -> int:
    source = Path(args.source).resolve()
    out = Path(args.output).resolve()
    tmp, holder = tmpdir_from_args(args)
    try:
        ir_path = Path(args.ir_out).resolve() if args.ir_out else tmp / "out.ll"
        bootstrap_emit_ir(source, ir_path, args)
        out.parent.mkdir(parents=True, exist_ok=True)
        run_checked(
            [args.clang, "-Wno-override-module", "-o", str(out), str(ir_path)],
            stderr=subprocess.PIPE,
        )
        return 0
    finally:
        if holder is not None:
            holder.cleanup()


def run_program(args: argparse.Namespace) -> int:
    tmp, holder = tmpdir_from_args(args)
    try:
        bin_path = tmp / "a.out"
        build_args = argparse.Namespace(**vars(args))
        build_args.output = str(bin_path)
        build_args.ir_out = None
        build(build_args)
        proc = subprocess.run([str(bin_path)])
        return proc.returncode
    finally:
        if holder is not None:
            holder.cleanup()


def add_common_flags(parser: argparse.ArgumentParser) -> None:
    parser.add_argument(
        "--legacy-root",
        default=os.environ.get("VAIS_COMPILER_ROOT", str(DEFAULT_LEGACY_ROOT)),
        help="Legacy Vais compiler repo used only as bootstrap/oracle backend.",
    )
    parser.add_argument(
        "--keep-tmp",
        action="store_true",
        help="Keep the temporary bootstrap directory for debugging.",
    )


def make_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="vaisc",
        description="New Vais compiler CLI. Accepts .vais and transitional .nl sources.",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    emit = sub.add_parser("emit-ir", help="Compile New Vais source to LLVM IR.")
    emit.add_argument("source")
    emit.add_argument("-o", "--output", default="-", help="LLVM IR output path, or '-' for stdout.")
    add_common_flags(emit)
    emit.set_defaults(func=emit_ir)

    bld = sub.add_parser("build", help="Compile New Vais source to a native binary.")
    bld.add_argument("source")
    bld.add_argument("-o", "--output", required=True, help="Native binary output path.")
    bld.add_argument("--ir-out", help="Optional path to also keep emitted LLVM IR.")
    bld.add_argument("--clang", default=os.environ.get("CLANG", "clang"))
    add_common_flags(bld)
    bld.set_defaults(func=build)

    run = sub.add_parser("run", help="Compile and run New Vais source, returning the program exit code.")
    run.add_argument("source")
    run.add_argument("--clang", default=os.environ.get("CLANG", "clang"))
    add_common_flags(run)
    run.set_defaults(func=run_program)

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = make_parser()
    args = parser.parse_args(argv)
    try:
        return args.func(args)
    except CompileError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
