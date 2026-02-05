window.BENCHMARK_DATA = {
  "lastUpdate": 1770271758578,
  "repoUrl": "https://github.com/vaislang/vais",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "sswoowkd@gmail.com",
            "name": "sswoo",
            "username": "sswoo88"
          },
          "committer": {
            "email": "sswoowkd@gmail.com",
            "name": "sswoo",
            "username": "sswoo88"
          },
          "distinct": true,
          "id": "b0b8b5e7663629b3075bd03e597d5d612a96370c",
          "message": "fix: add write permissions to benchmark workflow\n\nGitHub Actions bot needs explicit write permissions to push\nbenchmark results to gh-pages branch.\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-05T15:02:23+09:00",
          "tree_id": "b2caa398df85d70292feb0d707df95d7bb64a6a3",
          "url": "https://github.com/vaislang/vais/commit/b0b8b5e7663629b3075bd03e597d5d612a96370c"
        },
        "date": 1770271758173,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2696,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5079,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5929,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11134,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17674,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33903,
            "range": "± 198",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29847,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66256,
            "range": "± 716",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 268279,
            "range": "± 969",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 408728,
            "range": "± 3647",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 101359,
            "range": "± 656",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 706562,
            "range": "± 2302",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 151266,
            "range": "± 729",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 178480,
            "range": "± 643",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 187681,
            "range": "± 707",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 228577,
            "range": "± 1740",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 483297,
            "range": "± 4514",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 680895,
            "range": "± 2071",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 376786,
            "range": "± 3006",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1086653,
            "range": "± 16236",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37427,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 195088,
            "range": "± 379",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 377412,
            "range": "± 750",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1821848,
            "range": "± 13676",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}