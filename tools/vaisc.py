#!/usr/bin/env python3
"""New Vais compiler CLI bootstrap wrapper.

This is the user-facing `vaisc` command contract for New Vais. During the
transition it bootstraps through Legacy Vais, but the emitted LLVM IR is produced
by the New Vais self-host compiler in compiler/self/fixpoint_full.nl.
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FIXPOINT_FULL = ROOT / "compiler" / "self" / "fixpoint_full.nl"
EMBED_SELF_SOURCE = ROOT / "tools" / "embed_self_source.py"
TRANSPILER = ROOT / "compiler" / "transpiler" / "nl2vais.py"
DEFAULT_LEGACY_ROOT = Path("/Users/sswoo/study/projects/vais/compiler")


class CompileError(RuntimeError):
    pass


class FrontContractError(CompileError):
    pass


@dataclass(frozen=True)
class FrontIssue:
    line: int
    col: int
    message: str
    help: str


def code_only(line: str) -> str:
    out: list[str] = []
    string_delim: str | None = None
    escaped = False
    for ch in line:
        if escaped:
            out.append(" ")
            escaped = False
            continue
        if string_delim == '"' and ch == "\\":
            out.append(" ")
            escaped = True
            continue
        if string_delim is None and ch == "#":
            out.append(" " * (len(line) - len(out)))
            break
        if ch in ('"', "`"):
            if string_delim is None:
                string_delim = ch
            elif string_delim == ch:
                string_delim = None
            out.append(" ")
            continue
        out.append(" " if string_delim is not None else ch)
    return "".join(out)


def issue_from_match(match: re.Match[str], line_no: int, message: str, help_text: str) -> FrontIssue:
    return FrontIssue(line_no, match.start() + 1, message, help_text)


FRONT_UNSUPPORTED_RULES: list[tuple[re.Pattern[str], str, str]] = [
    (
        re.compile(r"\bstruct\b"),
        "struct declarations are not in the New Vais native day-1 front subset yet",
        "use Int values/functions for this slice, or keep structs on the Legacy bootstrap path until NV-C4.",
    ),
    (
        re.compile(r"\benum\b"),
        "enum declarations are not in the New Vais native day-1 front subset yet",
        "model this day-1 case with Int tags, or keep enums on the Legacy bootstrap path until NV-C4.",
    ),
    (
        re.compile(r"\bmatch\b"),
        "`match` is not in the New Vais native day-1 front subset yet",
        "use if/else for day-1 native sources, or keep match-based code on the Legacy bootstrap path.",
    ),
    (
        re.compile(r"\bfor\b"),
        "`for` loops are not in the New Vais native day-1 front subset yet",
        "use `while` with an explicit mutable index for now.",
    ),
    (
        re.compile(r"\b(print|putchar)\s*\("),
        "printing is not in the New Vais native day-1 front subset yet",
        "return an Int from `main` for this slice; IO stays on the bootstrap path for now.",
    ),
    (
        re.compile(r"\b(Str|Char|Bool)\b"),
        "only Int scalar typing is in the New Vais native day-1 front subset",
        "use Int parameters/locals for this slice; string, char, and bool surface types come later.",
    ),
    (
        re.compile(r"\b(true|false)\b"),
        "boolean literals are not in the New Vais native day-1 front subset yet",
        "use Int 0/1 values or comparisons in this slice.",
    ),
    (
        re.compile(r"\b(List|Map|Option|Result)\s*<|\blist\s*\("),
        "collections and sum-result types are not in the New Vais native day-1 front subset yet",
        "keep this source on the Legacy bootstrap path until the native parity gate grows to it.",
    ),
    (
        re.compile(r"\b(break|continue)\b"),
        "loop control statements are not in the New Vais native day-1 front subset yet",
        "rewrite the loop with an explicit condition variable for now.",
    ),
    (
        re.compile(r"\b(trait|impl)\b"),
        "traits and impl blocks are not in the New Vais native day-1 front subset yet",
        "use plain functions for this slice.",
    ),
    (
        re.compile(r"\|[^|]*\|"),
        "closures are not in the New Vais native day-1 front subset yet",
        "write a named function for this slice; closure object parity is tracked after the front contract.",
    ),
    (
        re.compile(r"\["),
        "list/array literals and indexing are not in the New Vais native day-1 front subset yet",
        "use scalar Int locals and function calls for this slice.",
    ),
    (
        re.compile(r"\.[A-Za-z_][A-Za-z0-9_]*\s*\("),
        "method calls are not in the New Vais native day-1 front subset yet",
        "use a plain function call for this slice.",
    ),
    (
        re.compile(r"\?"),
        "error propagation with `?` is not in the New Vais native day-1 front subset yet",
        "return an Int status code explicitly in this slice.",
    ),
]


FRONT_HELP_RULES: list[tuple[re.Pattern[str], str, str]] = [
    (
        re.compile(r"&&"),
        "logical AND uses the word `and`, not `&&`",
        "replace `&&` with `and`.",
    ),
    (
        re.compile(r"\|\|"),
        "logical OR uses the word `or`, not `||`",
        "replace `||` with `or`.",
    ),
    (
        re.compile(r"(?<![<>=!])!(?!=)(?=\s*\w|\s*\()"),
        "logical NOT uses the word `not`, not `!`",
        "replace `!expr` with `not expr`.",
    ),
    (
        re.compile(r"\bas\s+([A-Za-z_]\w*)"),
        "type conversion is explicit `Type(x)`, not `x as Type`",
        "write `Type(expr)` instead of `expr as Type`.",
    ),
    (
        re.compile(r"\b[A-Z]\w*::"),
        "enum/path access uses `.`, not `::`",
        "replace `::` with `.`.",
    ),
    (
        re.compile(r"\bVec\s*<|\bvec!\s*\["),
        "the New Vais list spelling is `List<T>` and `[a, b]`, not Rust Vec syntax",
        "use `List<T>`/list literals on the Legacy path for now; day-1 native front is scalar-only.",
    ),
    (
        re.compile(r"\bHashMap\b"),
        "the New Vais map spelling is `Map<K,V>`, not `HashMap<K,V>`",
        "use `Map<K,V>` on the Legacy path for now; day-1 native front is scalar-only.",
    ),
    (
        re.compile(r"\bString\b"),
        "the string type is `Str`, not `String`",
        "use `Str` on the Legacy path for now; day-1 native front is scalar-only.",
    ),
    (
        re.compile(r"[+\-*/%]=(?!=)"),
        "compound assignment is not New Vais syntax",
        "write it out, e.g. `x = x + 1`.",
    ),
]


FRONT_FN_HEADER = re.compile(
    r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)\s*(?:->\s*([A-Za-z_][A-Za-z0-9_]*))?"
)
FRONT_INT_PARAM = re.compile(r"[A-Za-z_][A-Za-z0-9_]*\s*:\s*Int\Z")


def check_function_contracts(code: str, line_no: int) -> tuple[list[FrontIssue], bool, bool]:
    issues: list[FrontIssue] = []
    has_main = False
    has_bad_main = False

    for match in FRONT_FN_HEADER.finditer(code):
        name = match.group(1)
        params = match.group(2).strip()
        return_type = match.group(3)

        if name == "main":
            if not params and return_type == "Int":
                has_main = True
            else:
                has_bad_main = True
            continue

        if return_type != "Int":
            issues.append(
                FrontIssue(
                    line_no,
                    match.start() + 1,
                    "New Vais native day-1 helper functions must return `Int`",
                    "write helpers as `fn name(a: Int, ...) -> Int { ... }`.",
                )
            )

        if params and any(not FRONT_INT_PARAM.fullmatch(part.strip()) for part in params.split(",")):
            issues.append(
                FrontIssue(
                    line_no,
                    match.start(2) + 1,
                    "New Vais native day-1 helper parameters must be `name: Int`",
                    "use Int-typed helper parameters in this slice, e.g. `fn add(a: Int, b: Int) -> Int`.",
                )
            )

    return issues, has_main, has_bad_main


def check_front_contract(source: Path) -> None:
    lines = source.read_text().splitlines()
    issues: list[FrontIssue] = []
    has_main = False
    has_bad_main = False

    for line_no, raw in enumerate(lines, 1):
        code = code_only(raw)
        fn_issues, line_has_main, line_has_bad_main = check_function_contracts(code, line_no)
        issues.extend(fn_issues)
        has_main = has_main or line_has_main
        has_bad_main = has_bad_main or line_has_bad_main

        for pattern, message, help_text in FRONT_HELP_RULES:
            match = pattern.search(code)
            if match:
                issues.append(issue_from_match(match, line_no, message, help_text))
                break
        else:
            for pattern, message, help_text in FRONT_UNSUPPORTED_RULES:
                match = pattern.search(code)
                if match:
                    issues.append(issue_from_match(match, line_no, message, help_text))
                    break

    if has_bad_main and not has_main:
        message = "New Vais native day-1 front requires `fn main() -> Int` exactly"
        help_text = "write the entrypoint as `fn main() -> Int { ... }`."
        issues.insert(0, FrontIssue(1, 1, message, help_text))
    elif not has_main:
        message = "New Vais native day-1 front requires `fn main() -> Int`"
        help_text = "add `fn main() -> Int { return <int> }` as the program entrypoint."
        issues.insert(0, FrontIssue(1, 1, message, help_text))

    if not issues:
        return

    formatted: list[str] = []
    for issue in issues:
        raw = lines[issue.line - 1] if 0 <= issue.line - 1 < len(lines) else ""
        formatted.extend(
            [
                f"error: {issue.message}",
                f"  --> {source}:{issue.line}:{issue.col}",
                f"  {raw}",
                f"  help: {issue.help}",
                "",
            ]
        )
    raise FrontContractError("\n".join(formatted).rstrip())


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
    check_front_contract(source)

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
        description=(
            "New Vais compiler CLI. Accepts .vais and transitional .nl sources "
            "that fit the native day-1 front contract."
        ),
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
    except FrontContractError as exc:
        print(str(exc), file=sys.stderr)
        return 1
    except CompileError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
