#!/usr/bin/env python3
"""Count LLM tokens for each language implementation using tiktoken (cl100k_base, used by GPT-4/Claude)."""
import os
import tiktoken

BENCH_DIR = os.path.dirname(os.path.abspath(__file__))
LANGUAGES = ["vais", "rust", "go", "c", "python"]
PROGRAMS = ["fibonacci", "quicksort", "http_types", "linked_list"]
EXTENSIONS = {"vais": ".vais", "rust": ".rs", "go": ".go", "c": ".c", "python": ".py"}

enc = tiktoken.get_encoding("cl100k_base")

def count_file(path):
    with open(path, "r") as f:
        content = f.read()
    tokens = enc.encode(content)
    chars = len(content)
    lines = content.count("\n") + (1 if content and not content.endswith("\n") else 0)
    return len(tokens), chars, lines

def main():
    results = {}  # (program, lang) -> (tokens, chars, lines)

    for prog in PROGRAMS:
        for lang in LANGUAGES:
            ext = EXTENSIONS[lang]
            path = os.path.join(BENCH_DIR, lang, f"{prog}{ext}")
            if os.path.exists(path):
                results[(prog, lang)] = count_file(path)

    # Print per-program table
    print("=" * 80)
    print("LLM TOKEN EFFICIENCY BENCHMARK — Vais vs Rust vs Go vs C vs Python")
    print("Tokenizer: cl100k_base (GPT-4 / Claude compatible)")
    print("=" * 80)

    for prog in PROGRAMS:
        print(f"\n### {prog}")
        print(f"{'Language':<10} {'Tokens':>8} {'Chars':>8} {'Lines':>6} {'vs Vais':>10}")
        print("-" * 50)
        vais_tokens = results.get((prog, "vais"), (0, 0, 0))[0]
        for lang in LANGUAGES:
            key = (prog, lang)
            if key in results:
                tokens, chars, lines = results[key]
                if lang == "vais":
                    ratio = "baseline"
                elif vais_tokens > 0:
                    ratio = f"{((tokens - vais_tokens) / vais_tokens * 100):+.1f}%"
                else:
                    ratio = "N/A"
                print(f"{lang:<10} {tokens:>8} {chars:>8} {lines:>6} {ratio:>10}")

    # Print summary
    print("\n" + "=" * 80)
    print("SUMMARY — Total tokens across all programs")
    print("=" * 80)
    totals = {}
    for lang in LANGUAGES:
        total_tokens = sum(results.get((p, lang), (0, 0, 0))[0] for p in PROGRAMS)
        total_chars = sum(results.get((p, lang), (0, 0, 0))[1] for p in PROGRAMS)
        total_lines = sum(results.get((p, lang), (0, 0, 0))[2] for p in PROGRAMS)
        totals[lang] = (total_tokens, total_chars, total_lines)

    vais_total = totals["vais"][0]
    print(f"{'Language':<10} {'Tokens':>8} {'Chars':>8} {'Lines':>6} {'vs Vais':>10} {'Tokens/Line':>12}")
    print("-" * 60)
    for lang in LANGUAGES:
        tokens, chars, lines = totals[lang]
        if lang == "vais":
            ratio = "baseline"
        elif vais_total > 0:
            ratio = f"{((tokens - vais_total) / vais_total * 100):+.1f}%"
        else:
            ratio = "N/A"
        tpl = f"{tokens/lines:.1f}" if lines > 0 else "N/A"
        print(f"{lang:<10} {tokens:>8} {chars:>8} {lines:>6} {ratio:>10} {tpl:>12}")

    # Save savings percentage
    print("\n--- Token Savings vs Vais ---")
    for lang in LANGUAGES:
        if lang == "vais":
            continue
        saving = (totals[lang][0] - vais_total) / totals[lang][0] * 100
        print(f"Vais saves {saving:.1f}% tokens compared to {lang}")

if __name__ == "__main__":
    main()
