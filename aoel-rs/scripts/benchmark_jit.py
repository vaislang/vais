#!/usr/bin/env python3
"""
Vais JIT vs 인터프리터 성능 비교 벤치마크

JIT 컴파일이 활성화된 경우와 그렇지 않은 경우의 성능을 비교합니다.
"""

import subprocess
import time
import os
import sys

# 테스트할 Vais 프로그램들
BENCHMARKS = {
    "add": {
        "source": "add(a, b) = a + b",
        "func": "add",
        "args": "[100, 200]",
        "iterations": 1000,
    },
    "mul_complex": {
        "source": "calc(a, b) = (a + b) * (a - b)",
        "func": "calc",
        "args": "[50, 30]",
        "iterations": 1000,
    },
    "arithmetic": {
        "source": "math(x) = ((x * 2 + 3) * 4 - 5) / 2",
        "func": "math",
        "args": "[100]",
        "iterations": 1000,
    },
}

def write_temp_file(source: str) -> str:
    """임시 Vais 파일 생성"""
    temp_path = "/tmp/vais_bench.vais"
    with open(temp_path, "w") as f:
        f.write(source)
    return temp_path

def run_benchmark(name: str, source: str, func: str, args: str, iterations: int, use_jit: bool) -> float:
    """벤치마크 실행 및 시간 측정"""
    temp_path = write_temp_file(source)

    cmd = ["cargo", "run", "--release", "-p", "vais-cli"]
    if use_jit:
        cmd.extend(["--features", "jit"])
    cmd.extend(["--", "run", temp_path, "-f", func, "-a", args])
    if use_jit:
        cmd.append("--jit")

    # Warmup
    for _ in range(3):
        subprocess.run(cmd, capture_output=True, cwd="/Users/sswoo/study/projects/vais/vais-rs")

    # Benchmark
    start = time.perf_counter()
    for _ in range(iterations):
        result = subprocess.run(cmd, capture_output=True, cwd="/Users/sswoo/study/projects/vais/vais-rs")
        if result.returncode != 0:
            print(f"Error running {name}: {result.stderr.decode()}")
            return -1
    end = time.perf_counter()

    return (end - start) / iterations * 1_000_000  # microseconds per iteration

def main():
    print("=" * 80)
    print("Vais JIT vs 인터프리터 성능 비교")
    print("=" * 80)
    print()
    print("⚠️  참고: 프로세스 시작 오버헤드가 포함됨 (순수 VM 성능은 Rust 벤치마크 참고)")
    print()

    results = []

    for name, bench in BENCHMARKS.items():
        print(f"📌 {name}...")

        # 인터프리터 실행
        interp_time = run_benchmark(
            name,
            bench["source"],
            bench["func"],
            bench["args"],
            bench["iterations"],
            use_jit=False
        )

        # JIT 실행
        jit_time = run_benchmark(
            name,
            bench["source"],
            bench["func"],
            bench["args"],
            bench["iterations"],
            use_jit=True
        )

        if interp_time > 0 and jit_time > 0:
            speedup = interp_time / jit_time
            results.append({
                "name": name,
                "interp_us": interp_time,
                "jit_us": jit_time,
                "speedup": speedup,
            })
            print(f"  인터프리터: {interp_time:.0f} µs, JIT: {jit_time:.0f} µs, 속도 향상: {speedup:.2f}x")
        else:
            print(f"  벤치마크 실패")

    print()
    print("=" * 80)
    print("📊 결과 요약")
    print("=" * 80)
    print()
    print(f"  {'벤치마크':<20} {'인터프리터 (µs)':>15} {'JIT (µs)':>15} {'속도 향상':>12}")
    print(f"  {'-'*20} {'-'*15} {'-'*15} {'-'*12}")

    for r in results:
        print(f"  {r['name']:<20} {r['interp_us']:>15.0f} {r['jit_us']:>15.0f} {r['speedup']:>11.2f}x")

    if results:
        avg_speedup = sum(r['speedup'] for r in results) / len(results)
        print()
        print(f"  평균 속도 향상: {avg_speedup:.2f}x")

if __name__ == "__main__":
    main()
