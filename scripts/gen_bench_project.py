#!/usr/bin/env python3
"""Generate multi-file Vais and C projects for incremental compilation benchmarks."""
import sys
import os

def gen_vais(target_lines, out_dir):
    os.makedirs(out_dir, exist_ok=True)
    funcs_per_mod = 20
    structs_per_mod = 5
    lines_per_mod = funcs_per_mod * 6 + structs_per_mod * 6
    num_mods = max(2, target_lines // lines_per_mod)

    for m in range(num_mods):
        code = []
        for s in range(structs_per_mod):
            code.append(f"S Mod{m}S{s} {{")
            code.append(f"    a: i64,")
            code.append(f"    b: i64,")
            code.append(f"    c: bool")
            code.append(f"}}")
            code.append("")
        for f in range(funcs_per_mod):
            fn_id = m * funcs_per_mod + f
            kind = f % 5
            if kind == 0:
                code.append(f"F m{m}_f{fn_id}(x: i64, y: i64) -> i64 {{")
                code.append(f"    a := x * {fn_id+1} + y")
                code.append(f"    b := a - {fn_id%7+1} * x")
                code.append(f"    R a + b")
                code.append(f"}}")
            elif kind == 1:
                code.append(f"F m{m}_f{fn_id}(n: i64) -> i64 {{")
                code.append(f"    I n <= 1 {{ R {fn_id%3+1} }}")
                code.append(f"    R n * @(n - 1)")
                code.append(f"}}")
            elif kind == 2:
                code.append(f"F m{m}_f{fn_id}(x: i64) -> i64 {{")
                code.append(f"    I x < {fn_id*3} {{ R x * 2 }}")
                code.append(f"    I x < {fn_id*10} {{ R x + {fn_id} }}")
                code.append(f"    R x")
                code.append(f"}}")
            elif kind == 3:
                code.append(f"F m{m}_f{fn_id}(n: i64) -> i64 {{")
                code.append(f"    sum := mut 0")
                code.append(f"    i := mut 0")
                code.append(f"    L {{")
                code.append(f"        I i >= n {{ R sum }}")
                code.append(f"        sum = sum + i * {fn_id%5+1}")
                code.append(f"        i = i + 1")
                code.append(f"    }}")
                code.append(f"}}")
            else:
                code.append(f"F m{m}_f{fn_id}(a: i64, b: i64, c: i64) -> i64 {{")
                code.append(f"    x := a * {fn_id} + b")
                code.append(f"    y := b * {fn_id%4+1} - c")
                code.append(f"    z := x + y + c * {fn_id%3}")
                code.append(f"    R x + y + z")
                code.append(f"}}")
            code.append("")
        with open(os.path.join(out_dir, f"mod{m}.vais"), "w") as fh:
            fh.write("\n".join(code) + "\n")

    imports = "\n".join(f"U mod{m}" for m in range(num_mods))
    with open(os.path.join(out_dir, "main.vais"), "w") as fh:
        fh.write(f"{imports}\n\nF main() -> i64 {{\n    result := m0_f0(1, 2)\n    R result\n}}\n")
    print(num_mods)

def gen_c(target_lines, out_dir):
    os.makedirs(out_dir, exist_ok=True)
    funcs_per_mod = 20
    lines_per_mod = funcs_per_mod * 6 + 30
    num_mods = max(2, target_lines // lines_per_mod)

    header = ["#pragma once", "#include <stdint.h>"]
    for m in range(num_mods):
        lines = ["#include <stdint.h>", '#include "funcs.h"']
        for f in range(funcs_per_mod):
            fn_id = m * funcs_per_mod + f
            lines.append(f"int64_t m{m}_f{fn_id}(int64_t a, int64_t b) {{")
            lines.append(f"    int64_t x = a * {fn_id+1} + b;")
            lines.append(f"    int64_t y = x - {fn_id%7+1} * a;")
            lines.append(f"    return x + y;")
            lines.append(f"}}")
            header.append(f"int64_t m{m}_f{fn_id}(int64_t a, int64_t b);")
        with open(os.path.join(out_dir, f"mod{m}.c"), "w") as fh:
            fh.write("\n".join(lines) + "\n")

    with open(os.path.join(out_dir, "funcs.h"), "w") as fh:
        fh.write("\n".join(header) + "\n")
    with open(os.path.join(out_dir, "main.c"), "w") as fh:
        fh.write('#include "funcs.h"\nint main() { return (int)m0_f0(1, 2); }\n')

    objs = " ".join(f"mod{m}.o" for m in range(num_mods))
    mk = f"CC=clang\nCFLAGS=-O2\nOBJS=main.o {objs}\n\nbench: $(OBJS)\n\t$(CC) $(CFLAGS) -o bench $(OBJS)\n\n%.o: %.c funcs.h\n\t$(CC) $(CFLAGS) -c $< -o $@\n\nclean:\n\trm -f *.o bench\n"
    with open(os.path.join(out_dir, "Makefile"), "w") as fh:
        fh.write(mk)
    print(num_mods)

if __name__ == "__main__":
    lang = sys.argv[1]  # "vais" or "c"
    target = int(sys.argv[2])
    out = sys.argv[3]
    if lang == "vais":
        gen_vais(target, out)
    else:
        gen_c(target, out)
