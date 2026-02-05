window.BENCHMARK_DATA = {
  "lastUpdate": 1770277616298,
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
      },
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
          "id": "aa7d3dfa490386017a5ffb2616bdcfd084d0f737",
          "message": "feat: implement smart C runtime linking for vaisc build\n\nAdd module-based automatic C runtime detection and linking:\n- get_runtime_for_module(): 20+ std modules mapped to C runtime files\n- extract_used_modules(): Extract imports from AST (supports std/x and std::x formats)\n- find_runtime_file(): Generic runtime file discovery\n- Selective linking: only link runtimes actually used by the program\n- Auto-detect pthread requirements and system libraries (-lssl, -lz, etc.)\n- Legacy fallback for backwards compatibility\n\nPhase 37 Stage 2 partial completion.\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-05T15:46:56+09:00",
          "tree_id": "e9ca495d327409d13e4a0a9f60e364fddaea0938",
          "url": "https://github.com/vaislang/vais/commit/aa7d3dfa490386017a5ffb2616bdcfd084d0f737"
        },
        "date": 1770274432179,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2734,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5321,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5936,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11143,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 18285,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34076,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29976,
            "range": "± 654",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 67027,
            "range": "± 455",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 268310,
            "range": "± 6907",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 408691,
            "range": "± 2503",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 101963,
            "range": "± 688",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 704652,
            "range": "± 1930",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 150858,
            "range": "± 570",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 178852,
            "range": "± 842",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 187841,
            "range": "± 916",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 227969,
            "range": "± 977",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 486506,
            "range": "± 1629",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 684695,
            "range": "± 2351",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 382336,
            "range": "± 1494",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1091188,
            "range": "± 12441",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37665,
            "range": "± 282",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 195376,
            "range": "± 649",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 374761,
            "range": "± 3849",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1819330,
            "range": "± 9452",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "ef8504ed4a04e718b11b9a7dd3840386d9cf8e24",
          "message": "chore: update outdated dependencies to latest compatible versions\n\nMajor version updates (compatible):\n- dashmap 5.5 → 6.1\n- libloading 0.8 → 0.9\n- notify 6.1 → 8.2\n- toml 0.8 → 0.9\n- gimli 0.28 → 0.33\n- object 0.32 → 0.38\n- pyo3 0.25 → 0.28\n- napi/napi-derive 2.16 → 3.x\n- thiserror 1.0 → 2.0\n- rustyline 13.0/14.0 → 17.0\n- colored 2.1 → 3.0\n- dirs 5.0 → 6.0\n- criterion 0.5 → 0.8\n- config 0.14 → 0.15\n\nMinor/Patch updates:\n- clap 4.4 → 4.5\n- regex 1.10 → 1.12\n- wasmtime 41.0 → 41.0.3\n- inferno 0.11 → 0.12\n\nNot updated (breaking changes):\n- cranelift (API changes)\n- axum/tower (middleware signatures)\n- ureq (complete API rewrite)\n- rand (argon2 compatibility)\n\nPhase 37 Stage 3 complete.\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-05T16:16:56+09:00",
          "tree_id": "fa151464472b2427c405c5f72524a5a6ac6ed386",
          "url": "https://github.com/vaislang/vais/commit/ef8504ed4a04e718b11b9a7dd3840386d9cf8e24"
        },
        "date": 1770276241842,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2549,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5764,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6631,
            "range": "± 289",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11605,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 18771,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 35797,
            "range": "± 319",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 32390,
            "range": "± 228",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 68539,
            "range": "± 3969",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 263336,
            "range": "± 1378",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 400411,
            "range": "± 2623",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 97309,
            "range": "± 667",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 694404,
            "range": "± 4371",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 149471,
            "range": "± 1257",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 175843,
            "range": "± 5257",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 183124,
            "range": "± 965",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 224317,
            "range": "± 2289",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 484846,
            "range": "± 2276",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 683583,
            "range": "± 2874",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 376887,
            "range": "± 2159",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1092107,
            "range": "± 15675",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 40766,
            "range": "± 684",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 206805,
            "range": "± 784",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 404871,
            "range": "± 1289",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1989187,
            "range": "± 8485",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "1b6b6e9e708ecd378dbf1e626897bc86a8fb9359",
          "message": "feat: prepare registry server for Fly.io deployment\n\n- Add fly.toml and Dockerfile.fly for Fly.io deployment\n- Update Dockerfile to use Rust 1.85+ (edition2024 support)\n- Add PORT env var support for Fly.io compatibility\n- Add root-level /health endpoint for health checks\n- Create fly-deploy.sh script for deployment workflow\n- Create publish-packages.sh for batch package publishing\n- Update ROADMAP with Phase 37 Stage 4 progress\n\nAll 10 seed packages tested locally with Docker:\n- cli-args, color, csv, dotenv, env\n- math-ext, retry, toml-parser, validate, cache\n\nE2E verified: publish → search → download roundtrip\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-05T16:40:05+09:00",
          "tree_id": "d335318509820b0a83d8377a458be4858bb1fd99",
          "url": "https://github.com/vaislang/vais/commit/1b6b6e9e708ecd378dbf1e626897bc86a8fb9359"
        },
        "date": 1770277615600,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 1962,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 4953,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5794,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10207,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 16599,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 32472,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29150,
            "range": "± 283",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 63365,
            "range": "± 383",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 250064,
            "range": "± 905",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 372036,
            "range": "± 1002",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 106688,
            "range": "± 756",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 635463,
            "range": "± 2484",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 153373,
            "range": "± 393",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 181281,
            "range": "± 449",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 190767,
            "range": "± 905",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 233064,
            "range": "± 4548",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 481796,
            "range": "± 15013",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 671682,
            "range": "± 15969",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 384596,
            "range": "± 1326",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1051179,
            "range": "± 34179",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38241,
            "range": "± 182",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 183725,
            "range": "± 1424",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 356977,
            "range": "± 2412",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1751895,
            "range": "± 14492",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}