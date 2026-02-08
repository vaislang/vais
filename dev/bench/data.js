window.BENCHMARK_DATA = {
  "lastUpdate": 1770515058081,
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
          "id": "8b2278866195ad1dc30f0efe2690637663f328dc",
          "message": "feat: deploy vais-registry to Fly.io production\n\n- Deploy to https://vais-registry.fly.dev (Tokyo region)\n- Publish 10 seed packages to production registry\n- Fix Dockerfile.fly to copy workspace members (examples/plugins, benches)\n- Add .registry-credentials.md to .gitignore\n- Mark Phase 37 Stage 4 as complete\n\nE2E verified:\n- Health check: /health returns ok\n- Search: /api/v1/search?q=csv returns results\n- Download: /api/v1/packages/csv/1.0.0 returns tar.gz\n- 10 packages available: cli-args, color, csv, dotenv, env,\n  math-ext, retry, toml-parser, validate, cache\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-05T16:53:27+09:00",
          "tree_id": "5a909222a570cc0239079389043b9f4b471e5b58",
          "url": "https://github.com/vaislang/vais/commit/8b2278866195ad1dc30f0efe2690637663f328dc"
        },
        "date": 1770278414725,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2390,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5568,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6146,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11233,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17829,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34032,
            "range": "± 253",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29843,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65967,
            "range": "± 586",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 268255,
            "range": "± 1165",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 409073,
            "range": "± 1617",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100807,
            "range": "± 656",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 707623,
            "range": "± 2546",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 150529,
            "range": "± 600",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 178200,
            "range": "± 700",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 185613,
            "range": "± 3921",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 226912,
            "range": "± 2580",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 483122,
            "range": "± 1772",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 683786,
            "range": "± 4018",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 378296,
            "range": "± 2529",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1092805,
            "range": "± 9104",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38915,
            "range": "± 432",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 193772,
            "range": "± 1310",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 374161,
            "range": "± 2896",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1811737,
            "range": "± 19405",
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
          "id": "ff197d8cf5db860094a6d1f0438c311d60412e1b",
          "message": "feat(selfhost): add keywords and escape decoding to selfhost lexer\n\n- Add self/Self/as/const keyword recognition to lexer.vais\n- Add D/O/N/G/Y single-letter keywords for defer/union/extern/global/await\n- Implement string escape sequence decoding: \\n \\t \\r \\\\ \\\" \\0 \\xHH\n- Add global helper functions (token_simple, token_int_lit, etc.) to token.vais\n- Add 9 new token constants for the additional keywords\n\nThis advances selfhost towards 75% completion (Phase 37 Stage 5).\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-05T17:12:49+09:00",
          "tree_id": "930a6a78e1bb0573a70738e28b774a2ada1b4ecd",
          "url": "https://github.com/vaislang/vais/commit/ff197d8cf5db860094a6d1f0438c311d60412e1b"
        },
        "date": 1770279588032,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2420,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5150,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5986,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11760,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17716,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33795,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29693,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66597,
            "range": "± 1504",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 269445,
            "range": "± 940",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 412012,
            "range": "± 5306",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100430,
            "range": "± 546",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 712057,
            "range": "± 2575",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 152001,
            "range": "± 783",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 179027,
            "range": "± 1571",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 187540,
            "range": "± 1112",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 230672,
            "range": "± 1422",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 486306,
            "range": "± 1998",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 690609,
            "range": "± 10947",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 382048,
            "range": "± 1982",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1096718,
            "range": "± 7344",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38139,
            "range": "± 272",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 196956,
            "range": "± 1494",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 382325,
            "range": "± 3006",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1850314,
            "range": "± 14419",
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
          "id": "7e5371267f9b3bad4a39c120a76f99cdd9ce00cd",
          "message": "feat(selfhost): add lexer token equivalence verification tests\n\nAdd comprehensive tests to verify selfhost lexer produces equivalent\noutput to Rust lexer across all examples/ files:\n\n- Add token ID mapping table (66 critical tokens mapped 1:1)\n- Add examples/ coverage test: 145 files, 45640 tokens, 99.8% supported\n- Add token sequence verification for key syntax patterns\n- Add new keyword tests (self, Self, as, const)\n- Add string escape sequence decoding tests\n\nTest results: 114 selfhost tests passing (13 new tests added)\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-05T17:26:56+09:00",
          "tree_id": "f41ab84a1204ab6e5df3c1d7d041710cc48eda30",
          "url": "https://github.com/vaislang/vais/commit/7e5371267f9b3bad4a39c120a76f99cdd9ce00cd"
        },
        "date": 1770280448150,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2379,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5339,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5965,
            "range": "± 260",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11442,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17763,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33708,
            "range": "± 363",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30321,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66637,
            "range": "± 791",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 268601,
            "range": "± 3422",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 408991,
            "range": "± 2541",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100463,
            "range": "± 940",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 709197,
            "range": "± 2360",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 149506,
            "range": "± 698",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 177054,
            "range": "± 1131",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 186048,
            "range": "± 1786",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 226578,
            "range": "± 13645",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 479904,
            "range": "± 1671",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 678014,
            "range": "± 2029",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 374169,
            "range": "± 1165",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1082810,
            "range": "± 8020",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37255,
            "range": "± 294",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 194763,
            "range": "± 1194",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 377988,
            "range": "± 4152",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1808558,
            "range": "± 17783",
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
          "id": "536f258a97f3ff4ae2b1531e5e3de2c1a124d31d",
          "message": "feat(selfhost): achieve 100% token coverage for all examples\n\nAdd support for remaining tokens to achieve full lexer parity:\n\nSelfhost lexer additions:\n- Keywords: spawn, macro, comptime, dyn, linear, affine, move, consume, lazy, force\n- Operators: |> (PipeArrow), ... (Ellipsis), $ (Dollar), #[ (HashBracket)\n- Lifetime identifiers: 'a, 'static, etc.\n- SIMD types: Vec2f32, Vec4f32, Vec8f32, Vec2f64, Vec4f64, Vec4i32, Vec8i32, Vec2i64, Vec4i64\n\nTest results:\n- 145 .vais files: 100% lexing success\n- 45,640 tokens: 100% selfhost support (up from 99.8%)\n- 114 selfhost tests passing\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-05T17:36:00+09:00",
          "tree_id": "8b8f8c51b435496be48cab9d12e5a2205454bbfc",
          "url": "https://github.com/vaislang/vais/commit/536f258a97f3ff4ae2b1531e5e3de2c1a124d31d"
        },
        "date": 1770280968661,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2414,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5498,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6067,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11120,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17755,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34361,
            "range": "± 832",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30371,
            "range": "± 265",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66756,
            "range": "± 413",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 269899,
            "range": "± 960",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 410147,
            "range": "± 2311",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100849,
            "range": "± 481",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 713043,
            "range": "± 10710",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 152414,
            "range": "± 705",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 180138,
            "range": "± 731",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 188167,
            "range": "± 2880",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 230529,
            "range": "± 2324",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 486818,
            "range": "± 10412",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 685414,
            "range": "± 2719",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 378128,
            "range": "± 4809",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1091680,
            "range": "± 9580",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 39245,
            "range": "± 273",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 197423,
            "range": "± 1259",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 378369,
            "range": "± 5954",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1833032,
            "range": "± 12433",
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
          "id": "57e62479d32bf8df3f6fd4edcd6a8cc4cd030eba",
          "message": "docs(roadmap): mark Phase 37 Stage 5 selfhost 75% as completed\n\n- Lexer 100% token equivalence achieved (145 files, 45,640 tokens)\n- Parser implementation deferred to Phase 38\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-05T17:46:16+09:00",
          "tree_id": "baad11d145abc824b3e33c37dbe4bcbda413cbf1",
          "url": "https://github.com/vaislang/vais/commit/57e62479d32bf8df3f6fd4edcd6a8cc4cd030eba"
        },
        "date": 1770281583226,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2394,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5191,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5994,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10935,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17979,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33959,
            "range": "± 377",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30066,
            "range": "± 410",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66112,
            "range": "± 383",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 270804,
            "range": "± 926",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 410056,
            "range": "± 1633",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100557,
            "range": "± 683",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 706252,
            "range": "± 5550",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 151803,
            "range": "± 711",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 179197,
            "range": "± 794",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 187110,
            "range": "± 1041",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 229204,
            "range": "± 4330",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 485650,
            "range": "± 1750",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 685714,
            "range": "± 3797",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 378145,
            "range": "± 1341",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1093097,
            "range": "± 13366",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38259,
            "range": "± 252",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 195805,
            "range": "± 1455",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 376245,
            "range": "± 3140",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1812836,
            "range": "± 19549",
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
          "id": "cb6fc733d0355fc339a77eab95addef47d3c9c57",
          "message": "docs(roadmap): add Phase 38 selfhosting 100% roadmap\n\nAdd comprehensive roadmap for achieving 100% self-hosting:\n- Stage 1: Parser completion (65% → 100%)\n- Stage 2: AST completion (85% → 100%)\n- Stage 3: Type Checker implementation (40% → 100%)\n- Stage 4: Codegen completion (70% → 100%)\n- Stage 5: MIR introduction (0% → 100%)\n- Stage 6: Bootstrapping test\n- Stage 7: Tool support (optional)\n\nCurrent selfhost progress: 75% (17,871 LOC)\nTarget: 100% (~31,000 LOC)\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-05T21:36:16+09:00",
          "tree_id": "149526ca3e9a672d34210e6e0ecb39623cfe031a",
          "url": "https://github.com/vaislang/vais/commit/cb6fc733d0355fc339a77eab95addef47d3c9c57"
        },
        "date": 1770295383858,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 1970,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 4957,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5815,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10097,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 16921,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 32248,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29226,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 62615,
            "range": "± 277",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 249481,
            "range": "± 2721",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 373737,
            "range": "± 7199",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 106919,
            "range": "± 599",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 636329,
            "range": "± 2150",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 154219,
            "range": "± 696",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 182847,
            "range": "± 525",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 192567,
            "range": "± 767",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 233533,
            "range": "± 1125",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 466570,
            "range": "± 3799",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 650394,
            "range": "± 4625",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 387484,
            "range": "± 1221",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1031464,
            "range": "± 18620",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37528,
            "range": "± 242",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 182968,
            "range": "± 1579",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 355532,
            "range": "± 1906",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1749108,
            "range": "± 17381",
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
          "id": "519784e45e53a25d2cba5e46034eebce3ab9ea83",
          "message": "feat(selfhost): add generics parsing to selfhost parser\n\n- Add parse_generics() helper function to parse <T, U, V> type parameters\n- Update parse_function, parse_struct, parse_enum, parse_impl to call parse_generics\n- Add GenericParam, Variant, EnumDef structs to parser.vais\n- Implement full enum parsing with tuple variants support\n- Update parser_s1.vais with same generics parsing capability\n- Mark Generics parsing as complete in ROADMAP.md\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-05T21:49:19+09:00",
          "tree_id": "b50bee1878ab00e35d9155d048909c66197379ad",
          "url": "https://github.com/vaislang/vais/commit/519784e45e53a25d2cba5e46034eebce3ab9ea83"
        },
        "date": 1770296170537,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2405,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5515,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6144,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10857,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17660,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33393,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29851,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66811,
            "range": "± 402",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 269319,
            "range": "± 5411",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 409950,
            "range": "± 2156",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100097,
            "range": "± 770",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 708988,
            "range": "± 6014",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 150197,
            "range": "± 735",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 177507,
            "range": "± 1979",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 184894,
            "range": "± 773",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 231845,
            "range": "± 7894",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 481945,
            "range": "± 39279",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 678207,
            "range": "± 2223",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 372967,
            "range": "± 4807",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1085501,
            "range": "± 26859",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38235,
            "range": "± 296",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 193083,
            "range": "± 2345",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 374987,
            "range": "± 2991",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1811359,
            "range": "± 21307",
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
          "id": "4a476c629cd204862783d6437b7949b8a7ad585e",
          "message": "feat(selfhost): add trait system parsing to selfhost parser\n\n- Add parse_trait() function to parse W (trait) definitions\n- Add parse_super_traits() for parsing `: Trait1 + Trait2`\n- Add parse_trait_bounds() for generic parameter bounds `T: Clone`\n- Extend parse_generics() to support trait bounds\n- Extend parse_impl() to support trait impl `X Type: Trait { ... }`\n- Add TraitDef struct definition to parser.vais\n\nAll trait-related tests pass. Parser progress: 65% → 70%\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-05T22:07:48+09:00",
          "tree_id": "a2086051bf13eae9d2b29f3bb0ea600fa33dd930",
          "url": "https://github.com/vaislang/vais/commit/4a476c629cd204862783d6437b7949b8a7ad585e"
        },
        "date": 1770297273176,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2440,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5205,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6090,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11589,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17843,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33693,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30222,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 67013,
            "range": "± 622",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 269341,
            "range": "± 1273",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 411253,
            "range": "± 3476",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100214,
            "range": "± 1042",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 715542,
            "range": "± 4034",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 151745,
            "range": "± 687",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 178665,
            "range": "± 1003",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 186896,
            "range": "± 1136",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 229999,
            "range": "± 1192",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 485173,
            "range": "± 8456",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 687606,
            "range": "± 2681",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 378957,
            "range": "± 1399",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1095773,
            "range": "± 13407",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37631,
            "range": "± 325",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 196173,
            "range": "± 4763",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 378610,
            "range": "± 2976",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1818242,
            "range": "± 17857",
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
          "id": "3ef9e2a422402a30a5e40809634d164d14868467",
          "message": "feat(selfhost): add complete pattern matching parsing\n\nImplement comprehensive pattern matching support in selfhost parser:\n- Wildcard pattern (_)\n- Integer literal patterns\n- Identifier binding patterns\n- Or patterns (A | B | C)\n- Range patterns (0..5, 10..=20)\n- Guard patterns (pattern I cond =>)\n- Enum variant patterns (Some(v), None)\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T04:21:09+09:00",
          "tree_id": "fb42ea490a36bac5469ff3608829c0594e0ad7f4",
          "url": "https://github.com/vaislang/vais/commit/3ef9e2a422402a30a5e40809634d164d14868467"
        },
        "date": 1770319676529,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2397,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5281,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6111,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10722,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17650,
            "range": "± 239",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33594,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29568,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65261,
            "range": "± 574",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 270584,
            "range": "± 975",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 410682,
            "range": "± 4241",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 99759,
            "range": "± 655",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 710724,
            "range": "± 2170",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 150174,
            "range": "± 797",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 177663,
            "range": "± 2704",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 185440,
            "range": "± 1263",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 228596,
            "range": "± 9186",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 486631,
            "range": "± 20946",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 684691,
            "range": "± 2373",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 375432,
            "range": "± 1710",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1091703,
            "range": "± 13312",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38036,
            "range": "± 259",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 197406,
            "range": "± 1524",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 380342,
            "range": "± 2613",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1821581,
            "range": "± 20037",
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
          "id": "053511d797f7552aeb44506ec9d7bf3a55883833",
          "message": "feat(selfhost): add closure, attribute, and async/await parsing\n\n- Closure parsing: |x| expr, |x: i64| -> i64 { }, |a, b| expr, move |x|\n- Attribute parsing: #[inline], #[derive(Clone, Debug)], #[custom(arg)]\n- Async/await parsing: A F foo() -> T, expr.await\n- Add test file demonstrating new features\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T04:29:03+09:00",
          "tree_id": "09b97c6d8a53f8a0f60c69c3b852dd0b71ca60c8",
          "url": "https://github.com/vaislang/vais/commit/053511d797f7552aeb44506ec9d7bf3a55883833"
        },
        "date": 1770320158941,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2401,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5322,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6025,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10759,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17776,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33726,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29587,
            "range": "± 1258",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65089,
            "range": "± 485",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 268443,
            "range": "± 1266",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 407394,
            "range": "± 1984",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 99186,
            "range": "± 741",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 704055,
            "range": "± 3523",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 149800,
            "range": "± 613",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 176992,
            "range": "± 696",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 184798,
            "range": "± 716",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 226991,
            "range": "± 1249",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 481262,
            "range": "± 2542",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 679524,
            "range": "± 3989",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 374729,
            "range": "± 2283",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1085316,
            "range": "± 21568",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38239,
            "range": "± 3998",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 194585,
            "range": "± 1235",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 375619,
            "range": "± 2595",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1814067,
            "range": "± 15865",
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
          "id": "c4710095099acd20b8cc5b9cc1688592b5a3f20f",
          "message": "feat(selfhost): complete AST with trait bounds, where clause, and utilities\n\nStage 2: AST 완성 (85% → 100%)\n- Add TraitBound node for trait constraints (T: Display + Clone)\n- Add WhereClause node for where constraints\n- Add AST printer utilities (print_expr, print_binop, etc.)\n- Add Visitor pattern foundation (visit_module_items, visit_expr_tree)\n- Update documentation with complete AST coverage\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T04:33:56+09:00",
          "tree_id": "670e897eaa31e5c9c31af5a3ab17732182516138",
          "url": "https://github.com/vaislang/vais/commit/c4710095099acd20b8cc5b9cc1688592b5a3f20f"
        },
        "date": 1770320440196,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2461,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5495,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6175,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11480,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 18013,
            "range": "± 437",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34348,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30323,
            "range": "± 566",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65965,
            "range": "± 505",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 268697,
            "range": "± 1706",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 410268,
            "range": "± 53172",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 99671,
            "range": "± 921",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 712093,
            "range": "± 4566",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 150710,
            "range": "± 2997",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 177057,
            "range": "± 693",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 184664,
            "range": "± 1556",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 227445,
            "range": "± 3868",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 484888,
            "range": "± 15614",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 681141,
            "range": "± 2508",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 373987,
            "range": "± 1977",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1089380,
            "range": "± 13511",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 39767,
            "range": "± 286",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 196495,
            "range": "± 1793",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 377124,
            "range": "± 2865",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1804758,
            "range": "± 19388",
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
          "id": "09d698dc497b0b3ac308887da6dcbf2fff6b241b",
          "message": "feat(selfhost): add bidirectional type inference to type checker\n\n- Add CheckMode constants (MODE_INFER, MODE_CHECK) for bidirectional checking\n- Implement substitution map for type variable binding\n- Add unify() function for recursive type unification\n- Add apply_substitutions() to resolve type variables\n- Add check_expr_bidirectional() as main entry point\n- Add check_lambda_with_expected() for lambda parameter inference\n- Add check_array_with_expected() for array element type inference\n- Extend test suite with unification and substitution tests\n- Update ROADMAP with Stage 3-1 completion and inkwell backend bug\n\nType checker now supports:\n- Type variable creation and binding\n- Unification of primitive, array, function, and named types\n- Lambda parameter type inference from expected function type\n- Array element type propagation from expected array type\n\nKnown issue: inkwell backend fails to compile selfhost files due to\nstruct field lookup bug (documented in ROADMAP)\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T09:09:33+09:00",
          "tree_id": "8e142706b6b7d85e497799bbfb43e9fa4b25c399",
          "url": "https://github.com/vaislang/vais/commit/09d698dc497b0b3ac308887da6dcbf2fff6b241b"
        },
        "date": 1770336982464,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 1991,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 4987,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5747,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10699,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 16507,
            "range": "± 291",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 31960,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29003,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 62379,
            "range": "± 423",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 250046,
            "range": "± 3195",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 373972,
            "range": "± 9521",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 105598,
            "range": "± 1242",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 635178,
            "range": "± 13103",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 153812,
            "range": "± 472",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 182046,
            "range": "± 740",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 191491,
            "range": "± 630",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 232023,
            "range": "± 2725",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 467690,
            "range": "± 2582",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 651558,
            "range": "± 9994",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 386373,
            "range": "± 1328",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1027557,
            "range": "± 6643",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37025,
            "range": "± 241",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 181470,
            "range": "± 2298",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 357041,
            "range": "± 2188",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1755132,
            "range": "± 17742",
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
          "id": "93e93a7c29bf450ac4e71e8cd948d3b1c20db33e",
          "message": "fix(inkwell): resolve struct type inference for selfhost compilation\n\nFix inkwell backend struct field access bugs that prevented selfhost/*.vais\ncompilation:\n- Add StaticMethodCall case to infer_value_struct_type for constructor patterns\n- Add SelfCall (@) case to infer_struct_name for method context\n- Resolves issue where identical LLVM struct types caused wrong type inference\n\nE2E tests: 241/241 passed\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T10:05:16+09:00",
          "tree_id": "4af3e9f0f6fca863545fefd894326857b59c159e",
          "url": "https://github.com/vaislang/vais/commit/93e93a7c29bf450ac4e71e8cd948d3b1c20db33e"
        },
        "date": 1770340323844,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2405,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5354,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6217,
            "range": "± 435",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11048,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17676,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33853,
            "range": "± 309",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29840,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65408,
            "range": "± 361",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 270666,
            "range": "± 1581",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 414106,
            "range": "± 1698",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 99250,
            "range": "± 1050",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 717802,
            "range": "± 3251",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 150465,
            "range": "± 614",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 177292,
            "range": "± 1352",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 185719,
            "range": "± 578",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 227363,
            "range": "± 1277",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 484356,
            "range": "± 3019",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 683865,
            "range": "± 2777",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 372606,
            "range": "± 1203",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1097004,
            "range": "± 19045",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38250,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 195981,
            "range": "± 4481",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 380101,
            "range": "± 3041",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1833843,
            "range": "± 39390",
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
          "id": "db859e5804a74b500d746609839cf9ac03424836",
          "message": "feat(selfhost): add generic type resolution to type checker\n\nImplement Stage 3-2 of selfhost compiler:\n- Add generic binding management (add/get/clear_generic_binding)\n- Add instantiate_type() for replacing TY_GENERIC with bound types\n- Add infer_generic_from_types() for inferring generics from arguments\n- Enhance EXPR_CALL to infer generics and instantiate return types\n- Enhance EXPR_FIELD for generic struct field access\n- Update apply_substitutions() to handle TY_GENERIC and TY_NAMED\n- Add resolve_named_type_with_generics() for type argument resolution\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T10:33:08+09:00",
          "tree_id": "d8cd656c07e0bb32e7a6a9db4bd7c5bdc0e6d787",
          "url": "https://github.com/vaislang/vais/commit/db859e5804a74b500d746609839cf9ac03424836"
        },
        "date": 1770342003776,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2420,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5503,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6128,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11548,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17728,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33542,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30026,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 64881,
            "range": "± 345",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 270788,
            "range": "± 13340",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 413746,
            "range": "± 1588",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 98560,
            "range": "± 450",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 716198,
            "range": "± 1994",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 151733,
            "range": "± 654",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 177730,
            "range": "± 765",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 185863,
            "range": "± 926",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 227608,
            "range": "± 1645",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 483078,
            "range": "± 4970",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 682843,
            "range": "± 4261",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 374376,
            "range": "± 1855",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1092243,
            "range": "± 11411",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38892,
            "range": "± 392",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 195282,
            "range": "± 1462",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 378144,
            "range": "± 2963",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1830010,
            "range": "± 18291",
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
          "id": "b21adc22840c62901f5cffccc5371e2a30df9f04",
          "message": "feat(selfhost): add trait resolution to type checker\n\nAdd comprehensive trait system support to the selfhost type checker:\n- TraitDefInfo and TraitImplInfo structures for trait tracking\n- add_trait/find_trait/register_trait for trait definition registry\n- add_trait_impl/find_trait_impl for trait implementation tracking\n- type_implements_trait for trait bound verification\n- check_trait with super trait validation and method type checking\n- check_impl enhanced with trait existence and method verification\n- ITEM_TRAIT handling in check_item and check_module first pass\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T10:48:15+09:00",
          "tree_id": "f20aa1d5ea638533f48a382254dd5861fbcfc08f",
          "url": "https://github.com/vaislang/vais/commit/b21adc22840c62901f5cffccc5371e2a30df9f04"
        },
        "date": 1770342898140,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2413,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5257,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5897,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10945,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17403,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33900,
            "range": "± 224",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29776,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65682,
            "range": "± 359",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 272949,
            "range": "± 999",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 413643,
            "range": "± 6003",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 99924,
            "range": "± 642",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 721597,
            "range": "± 2005",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 150153,
            "range": "± 722",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 177463,
            "range": "± 1031",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 186344,
            "range": "± 2677",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 226983,
            "range": "± 1324",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 483282,
            "range": "± 3272",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 685369,
            "range": "± 3520",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 377949,
            "range": "± 2091",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1093195,
            "range": "± 18244",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38720,
            "range": "± 735",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 195554,
            "range": "± 1907",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 381868,
            "range": "± 2355",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1845412,
            "range": "± 20033",
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
          "id": "4ea06b49a05991e6459be89d22516fc0e47a4b75",
          "message": "feat(selfhost): add bitwise operators and index expression support\n\n- Add bitwise operator parsing (& | ^ << >>) with proper precedence chain\n- Implement index expression [i] parsing in parse_postfix()\n- Add generate_index() codegen with GEP + load pattern\n- Update ROADMAP to v0.5.1\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T13:09:20+09:00",
          "tree_id": "e82f971e9899368c01cbd627b1890ef650da8822",
          "url": "https://github.com/vaislang/vais/commit/4ea06b49a05991e6459be89d22516fc0e47a4b75"
        },
        "date": 1770351403116,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2422,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5352,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6222,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11200,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17912,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34028,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29839,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65620,
            "range": "± 759",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 272043,
            "range": "± 949",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 418648,
            "range": "± 1539",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 98967,
            "range": "± 571",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 717034,
            "range": "± 2501",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 150961,
            "range": "± 1978",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 179542,
            "range": "± 2635",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 187260,
            "range": "± 3738",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 228844,
            "range": "± 975",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 484386,
            "range": "± 10022",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 686028,
            "range": "± 3118",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 377072,
            "range": "± 3533",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1095054,
            "range": "± 19928",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38322,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 197197,
            "range": "± 1651",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 381107,
            "range": "± 2980",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1841563,
            "range": "± 19227",
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
          "id": "9701029fe62aeec50e4518ccd5342d0c018a92f3",
          "message": "feat(selfhost): add array literal parsing and codegen\n\n- Add array literal [e1, e2, ...] parsing in parser_s1.vais and parser.vais\n- Add generate_array() function in codegen.vais for LLVM IR generation\n- Add EXPR_INDEX and EXPR_ARRAY handling in codegen_s1.vais\n- Add EXPR_ARRAY constant (38) to constants.vais\n- Update ROADMAP.md to version 0.5.2\n\nCo-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T16:05:24+09:00",
          "tree_id": "9e44b5424fdd1ae7f40b4e0bd3dd05e0f00c52d4",
          "url": "https://github.com/vaislang/vais/commit/9701029fe62aeec50e4518ccd5342d0c018a92f3"
        },
        "date": 1770361924166,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2386,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5090,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6006,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10977,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17671,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33867,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30112,
            "range": "± 370",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65418,
            "range": "± 752",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 270876,
            "range": "± 900",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 413784,
            "range": "± 1725",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100594,
            "range": "± 478",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 714561,
            "range": "± 2650",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 151373,
            "range": "± 1305",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 178098,
            "range": "± 963",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 186402,
            "range": "± 1325",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 229094,
            "range": "± 3797",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 486139,
            "range": "± 2957",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 686354,
            "range": "± 3079",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 376876,
            "range": "± 2127",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1094120,
            "range": "± 13763",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37950,
            "range": "± 303",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 196474,
            "range": "± 1570",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 375989,
            "range": "± 15370",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1820905,
            "range": "± 19102",
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
          "id": "37d439c41e654c6f435a39f4b7ab4886ebfd3337",
          "message": "feat(selfhost): add method resolution and error suggestions to type checker\n\nImplement full method resolution for EXPR_METHOD_CALL, EXPR_SELF_CALL,\nand EXPR_STATIC_CALL with inherent/trait method lookup, generic binding,\nand built-in array/string methods. Add Levenshtein-based \"did you mean?\"\nsuggestions for undefined functions, variables, and structs.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T19:49:39+09:00",
          "tree_id": "3bd1768db4edd91765f951ce2536ee910c3fa4d4",
          "url": "https://github.com/vaislang/vais/commit/37d439c41e654c6f435a39f4b7ab4886ebfd3337"
        },
        "date": 1770375382136,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2411,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5389,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5982,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11408,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 18019,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34073,
            "range": "± 275",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29604,
            "range": "± 175",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65556,
            "range": "± 571",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 273690,
            "range": "± 2673",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 423722,
            "range": "± 6400",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 99474,
            "range": "± 726",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 722220,
            "range": "± 3376",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 150804,
            "range": "± 921",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 181038,
            "range": "± 1457",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 186546,
            "range": "± 1234",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 226905,
            "range": "± 1219",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 485621,
            "range": "± 2152",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 688010,
            "range": "± 5360",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 376958,
            "range": "± 3089",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1113237,
            "range": "± 18310",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38565,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 193570,
            "range": "± 1832",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 375356,
            "range": "± 3320",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1843728,
            "range": "± 15317",
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
          "id": "ff9a09eac1af80ec68b908a41827010e40d4338b",
          "message": "feat(selfhost): add trait bounds verification and exhaustiveness checking\n\nAdd trait bounds management (add/verify/register per generic param),\nmatch expression exhaustiveness checking for enums and bools, and\ntype check handlers for EXPR_MATCH, EXPR_TERNARY, EXPR_INDEX,\nEXPR_LAMBDA, and EXPR_TUPLE expressions.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T19:54:49+09:00",
          "tree_id": "636810ec8066112b4dd93bd7afb1b061ad12a317",
          "url": "https://github.com/vaislang/vais/commit/ff9a09eac1af80ec68b908a41827010e40d4338b"
        },
        "date": 1770375700467,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2393,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5341,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6140,
            "range": "± 221",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10999,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 18193,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34232,
            "range": "± 346",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30107,
            "range": "± 1032",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65907,
            "range": "± 650",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 271344,
            "range": "± 733",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 412308,
            "range": "± 1783",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 99852,
            "range": "± 1167",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 715629,
            "range": "± 25061",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 151152,
            "range": "± 1415",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 178451,
            "range": "± 1318",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 187076,
            "range": "± 718",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 230887,
            "range": "± 1354",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 486175,
            "range": "± 1940",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 688582,
            "range": "± 2301",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 376940,
            "range": "± 3077",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1103803,
            "range": "± 22090",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38320,
            "range": "± 306",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 185910,
            "range": "± 1455",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 373791,
            "range": "± 3164",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1824534,
            "range": "± 26965",
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
          "id": "a271a24e7575eef47cee887d8a0200fc9cc19cfb",
          "message": "feat(selfhost): add associated types, trait objects, and monomorphization tracking\n\nAdd TY_DYN_TRAIT/TY_ASSOC type kinds, associated type registration and\nresolution, object safety checking, dyn Trait unification support, and\ngeneric instantiation tracking for codegen monomorphization. Type checker\nnow at 85% completion.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T20:06:34+09:00",
          "tree_id": "66a478868f948ccae860f0de422f1928b7cce3e3",
          "url": "https://github.com/vaislang/vais/commit/a271a24e7575eef47cee887d8a0200fc9cc19cfb"
        },
        "date": 1770376395507,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2428,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5344,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6075,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10764,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17703,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34216,
            "range": "± 196",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30044,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65277,
            "range": "± 351",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 271806,
            "range": "± 1002",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 416407,
            "range": "± 2114",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100832,
            "range": "± 797",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 723671,
            "range": "± 3221",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 150763,
            "range": "± 1238",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 177609,
            "range": "± 1697",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 186356,
            "range": "± 683",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 227614,
            "range": "± 1566",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 485810,
            "range": "± 2215",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 686852,
            "range": "± 6841",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 376359,
            "range": "± 3501",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1099832,
            "range": "± 11430",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38453,
            "range": "± 316",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 197121,
            "range": "± 2050",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 381722,
            "range": "± 9226",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1848844,
            "range": "± 19776",
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
          "id": "d0a6c0cc35677fffa4acd8cb2dfc136f4b301323",
          "message": "feat(selfhost): add match, ternary, method, field, and struct literal codegen\n\nAdd 6 new expression types to selfhost code generator:\n- Match expression: pattern matching with wildcard/ident/literal/variant patterns and phi merge\n- Ternary: cond ? then : else with branch and phi node\n- Method call: receiver as first argument dispatch\n- Static call: TypeName.method() form\n- Field access: pointer offset + load_i64\n- Struct literal: malloc + per-field store_i64\n\nCodegen expression coverage: 14/32 (44%) → 20/32 (63%)\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T20:22:54+09:00",
          "tree_id": "280b4ea7545880e4bb647cf25ca40dea54fc136f",
          "url": "https://github.com/vaislang/vais/commit/d0a6c0cc35677fffa4acd8cb2dfc136f4b301323"
        },
        "date": 1770377401049,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2409,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5397,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6421,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11455,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 18018,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33986,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30045,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65686,
            "range": "± 289",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 273646,
            "range": "± 993",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 417307,
            "range": "± 1976",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 97658,
            "range": "± 592",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 726609,
            "range": "± 2032",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 150245,
            "range": "± 766",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 177571,
            "range": "± 1524",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 185873,
            "range": "± 1353",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 228903,
            "range": "± 4323",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 485626,
            "range": "± 1839",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 687111,
            "range": "± 2293",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 371677,
            "range": "± 4034",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1101805,
            "range": "± 21711",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38145,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 197332,
            "range": "± 1011",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 379586,
            "range": "± 2492",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1818799,
            "range": "± 18467",
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
          "id": "08fcbcad5cf1ed01fed428fc2ebd3fdea5d31667",
          "message": "feat(selfhost): add unit, self call, tuple, range, and assign op codegen\n\nAdd 5 more expression types to selfhost code generator:\n- Unit expression: returns 0 (void equivalent)\n- Self call (@): recursive call to current function\n- Tuple: malloc + per-element store_i64\n- Range: 3-element struct (start, end, inclusive)\n- Assign op (+=, -=, *=, /=): load-op-store pattern\n\nCodegen expression coverage: 20/32 (63%) → 25/32 (78%)\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T20:29:19+09:00",
          "tree_id": "498f8504cd7d565e40a4b76cd4da2d4569272bef",
          "url": "https://github.com/vaislang/vais/commit/08fcbcad5cf1ed01fed428fc2ebd3fdea5d31667"
        },
        "date": 1770377754993,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2460,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5404,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6076,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11201,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17644,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33722,
            "range": "± 175",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29677,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65802,
            "range": "± 429",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 271044,
            "range": "± 2951",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 414178,
            "range": "± 1944",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 99338,
            "range": "± 907",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 714348,
            "range": "± 3552",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 150355,
            "range": "± 684",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 178638,
            "range": "± 923",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 186184,
            "range": "± 893",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 228030,
            "range": "± 6473",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 483003,
            "range": "± 1425",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 681480,
            "range": "± 6518",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 372697,
            "range": "± 1512",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1092772,
            "range": "± 17641",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38882,
            "range": "± 691",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 195047,
            "range": "± 3543",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 378115,
            "range": "± 7031",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1832137,
            "range": "± 34525",
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
          "id": "05ebe01b3fc4d5ab334a38f3c8a000850fd2e405",
          "message": "feat(selfhost): add ref, deref, try, and unwrap codegen\n\nAdd 4 more expression types to selfhost code generator:\n- Ref (&): return pointer to alloca or stack-allocated temp\n- Deref (*): load value from pointer via load_i64\n- Try (?): check Result tag, early return on Err, extract Ok value\n- Unwrap (!): extract value at offset 8 (skip tag)\n\nCodegen expression coverage: 25/32 (78%) → 29/32 (91%)\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T20:36:24+09:00",
          "tree_id": "4e3d99589d7a0031af1016de5521c534624607c9",
          "url": "https://github.com/vaislang/vais/commit/05ebe01b3fc4d5ab334a38f3c8a000850fd2e405"
        },
        "date": 1770378194460,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 1994,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5125,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5543,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10388,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 16712,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 32364,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29160,
            "range": "± 898",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 63633,
            "range": "± 729",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 249828,
            "range": "± 1099",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 375083,
            "range": "± 1229",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 105601,
            "range": "± 836",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 639277,
            "range": "± 2493",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 154229,
            "range": "± 2184",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 181521,
            "range": "± 1916",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 191478,
            "range": "± 1088",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 233508,
            "range": "± 6587",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 463042,
            "range": "± 2892",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 646811,
            "range": "± 2127",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 382355,
            "range": "± 2179",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1025014,
            "range": "± 21197",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37206,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 184751,
            "range": "± 3042",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 354086,
            "range": "± 6268",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1759460,
            "range": "± 15972",
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
          "id": "1fa2d5d0d7c645ada4627efb846ae50200b497af",
          "message": "feat(selfhost): add lambda codegen with separate function generation\n\nLambda expressions are compiled to separate LLVM functions:\n- Each lambda gets a unique name (@__lambda_N)\n- Parameters are emitted as i64 arguments\n- Body is generated into a separate lambda buffer\n- Function pointer is returned as i64 via ptrtoint\n- Lambda buffer is emitted after all regular functions\n\nInfrastructure additions:\n- lambda_buf: StringBuffer for collecting lambda definitions\n- lambda_counter: unique ID generator\n- lambda_emit_* helpers for writing to lambda buffer\n- emit_lambda_functions() called in generate_module\n\nCodegen expression coverage: 29/32 (91%) → 30/32 (94%)\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T20:43:15+09:00",
          "tree_id": "ebbba66967eada8fa0f13724360766cfb7c0d011",
          "url": "https://github.com/vaislang/vais/commit/1fa2d5d0d7c645ada4627efb846ae50200b497af"
        },
        "date": 1770378605389,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2436,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5158,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6347,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11373,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 18139,
            "range": "± 177",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33946,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30200,
            "range": "± 386",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65615,
            "range": "± 294",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 271332,
            "range": "± 1192",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 415075,
            "range": "± 2468",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 102681,
            "range": "± 3104",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 719162,
            "range": "± 2997",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 150090,
            "range": "± 650",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 177595,
            "range": "± 760",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 186842,
            "range": "± 879",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 226155,
            "range": "± 794",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 486096,
            "range": "± 6730",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 685640,
            "range": "± 2474",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 376902,
            "range": "± 3952",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1095432,
            "range": "± 15465",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38355,
            "range": "± 266",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 198733,
            "range": "± 1807",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 380201,
            "range": "± 3285",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1839561,
            "range": "± 19467",
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
          "id": "8004dae27828f5f9724ea95ec32e093fa5df9e87",
          "message": "feat(selfhost): add generic monomorphization codegen infrastructure\n\nAdd complete monomorphization support to the selfhost codegen:\n- Save generic function ASTs for later specialization instead of skipping\n- Generate specialized function copies with type-substituted parameters\n- Name mangling for mono functions (fn_mono_typekinds pattern)\n- Generic binding system for type parameter resolution during codegen\n- Import mono entries from type checker's analysis results\n- Integrate mono generation into module codegen pipeline\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T20:59:43+09:00",
          "tree_id": "6a161e84c05f869ed2172a934d4a029ed089d1ad",
          "url": "https://github.com/vaislang/vais/commit/8004dae27828f5f9724ea95ec32e093fa5df9e87"
        },
        "date": 1770379593312,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2351,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 4355,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 4763,
            "range": "± 175",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10751,
            "range": "± 306",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17570,
            "range": "± 301",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33335,
            "range": "± 679",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30533,
            "range": "± 560",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65604,
            "range": "± 1087",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 266779,
            "range": "± 6001",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 406872,
            "range": "± 7144",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 97987,
            "range": "± 2194",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 712037,
            "range": "± 13206",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 149586,
            "range": "± 3523",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 175313,
            "range": "± 4031",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 185427,
            "range": "± 4324",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 228214,
            "range": "± 5016",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 478609,
            "range": "± 11347",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 676645,
            "range": "± 13028",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 370516,
            "range": "± 10267",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1093203,
            "range": "± 25933",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37676,
            "range": "± 524",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 189624,
            "range": "± 3465",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 366429,
            "range": "± 9089",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1815455,
            "range": "± 34661",
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
          "id": "f2e98a6cfe74fff559392309835c472adcdf7d70",
          "message": "feat(selfhost): add struct mono, vtable dispatch, lambda captures, and constant folding\n\nStage 4 codegen improvements:\n- Generic struct monomorphization (save_generic_struct, generate_mono_struct_type)\n- Trait vtable infrastructure (VtableEntry, emit_vtable_globals, generate_dyn_call)\n- Lambda free variable capture analysis (collect_captures with recursive AST walk)\n- Constant folding optimization for binary (+/-/*/÷/%) and unary (-/~) expressions\n- Type checker struct mono tracking (add_struct_mono_entry, record_struct_mono)\n- ROADMAP progress updates (Loop labels, Try-catch already complete)\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T21:52:30+09:00",
          "tree_id": "3c732d32f852684f2fd1af64260ba77ba451f4c6",
          "url": "https://github.com/vaislang/vais/commit/f2e98a6cfe74fff559392309835c472adcdf7d70"
        },
        "date": 1770382773440,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2408,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5572,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6237,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11040,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17781,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33981,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29985,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65601,
            "range": "± 651",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 271736,
            "range": "± 987",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 414750,
            "range": "± 3017",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 99388,
            "range": "± 773",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 717506,
            "range": "± 2064",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 150386,
            "range": "± 672",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 178203,
            "range": "± 775",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 187782,
            "range": "± 873",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 229523,
            "range": "± 2778",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 492776,
            "range": "± 4387",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 704083,
            "range": "± 8054",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 374000,
            "range": "± 1921",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1126802,
            "range": "± 24927",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37940,
            "range": "± 346",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 195422,
            "range": "± 3682",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 379568,
            "range": "± 3438",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1843387,
            "range": "± 20196",
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
          "id": "865e21efff9085b80981b98662856096d11c462f",
          "message": "feat(selfhost): add DCE, basic inlining, and move/ref capture distinction\n\n- Dead Code Elimination: 2-pass text-based IR optimization that collects\n  %N definitions/uses and removes unused side-effect-free instructions\n- Basic Inlining: inline small pure functions (≤10 instructions, no calls)\n  with register renaming and parameter substitution at call sites\n- Move/Ref Capture: lambda captures now distinguish mutable locals (ref\n  capture via ptrtoint/inttoptr) from immutable params (move capture via\n  value copy), enabling proper shared mutable state in closures\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T22:17:47+09:00",
          "tree_id": "7d27368598b9ac98733943f5b97592add15df6f0",
          "url": "https://github.com/vaislang/vais/commit/865e21efff9085b80981b98662856096d11c462f"
        },
        "date": 1770384274139,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2413,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5298,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5758,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11105,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17853,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33728,
            "range": "± 623",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29733,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65937,
            "range": "± 221",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 272878,
            "range": "± 1332",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 413377,
            "range": "± 15534",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100217,
            "range": "± 877",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 714638,
            "range": "± 3239",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 150990,
            "range": "± 2350",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 177639,
            "range": "± 784",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 187161,
            "range": "± 958",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 227750,
            "range": "± 1656",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 484444,
            "range": "± 8195",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 689107,
            "range": "± 3012",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 375886,
            "range": "± 1783",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1094012,
            "range": "± 14342",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38950,
            "range": "± 293",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 195052,
            "range": "± 1429",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 378733,
            "range": "± 2962",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1811206,
            "range": "± 16216",
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
          "id": "6e5c315a51d83f585d0b4e5852be4ee1925f8bc8",
          "message": "feat(selfhost): add type mismatch detailed descriptions and error printing\n\nAdd format_type/format_type_to_sb for ResolvedType → human-readable string\nconversion (all 31 type kinds), print_errors for formatted error output,\nand standalone tc_sb_* StringBuffer helpers. Update 3 mismatch call sites\nto pass actual expected/found type info instead of hardcoded zeros.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-06T22:50:30+09:00",
          "tree_id": "d42e9ffddec338200537edefcc169ca100f2a0e3",
          "url": "https://github.com/vaislang/vais/commit/6e5c315a51d83f585d0b4e5852be4ee1925f8bc8"
        },
        "date": 1770386224086,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 1979,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5111,
            "range": "± 233",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5857,
            "range": "± 241",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10229,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 16640,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 32216,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 28748,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 62312,
            "range": "± 799",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 248846,
            "range": "± 3497",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 373101,
            "range": "± 1947",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 105907,
            "range": "± 337",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 636383,
            "range": "± 3045",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 153696,
            "range": "± 564",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 182560,
            "range": "± 579",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 191879,
            "range": "± 2794",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 232610,
            "range": "± 2938",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 469059,
            "range": "± 3186",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 654151,
            "range": "± 2587",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 386759,
            "range": "± 4796",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1029510,
            "range": "± 8226",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37479,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 182686,
            "range": "± 968",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 355740,
            "range": "± 5106",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1732893,
            "range": "± 19094",
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
          "id": "1e19f5023c7aeeb1452d2cc8edd3801fb38737ed",
          "message": "feat(selfhost): achieve bootstrap — Stage1→Stage2→Stage3 fixed point\n\n- Type Checker 100%: add check_enum, check_type_alias (7/7 item types)\n- Fix E001: add Ref(T)↔T auto-deref to unify() in inference.rs\n- Inkwell builtins: add fopen_ptr/memcpy_str wrappers, realloc declaration\n- Bootstrap achieved: SHA256 e14776a6... matches across stages (17,807 lines)\n- Update ROADMAP: Phase 38 complete, MIR moved to optional Stage 6\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T07:22:45+09:00",
          "tree_id": "16258386b22a08984e83a7445fa9eb7793fce75f",
          "url": "https://github.com/vaislang/vais/commit/1e19f5023c7aeeb1452d2cc8edd3801fb38737ed"
        },
        "date": 1770416979170,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2444,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5141,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6058,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11358,
            "range": "± 121",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17634,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33967,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30330,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66282,
            "range": "± 523",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 266381,
            "range": "± 1014",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 405932,
            "range": "± 5486",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 98998,
            "range": "± 422",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 704273,
            "range": "± 3598",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 151536,
            "range": "± 1402",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 177895,
            "range": "± 651",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 186839,
            "range": "± 2989",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 227872,
            "range": "± 1189",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 482704,
            "range": "± 2489",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 679627,
            "range": "± 2118",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 376360,
            "range": "± 1973",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1092760,
            "range": "± 13447",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37708,
            "range": "± 516",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 195164,
            "range": "± 2173",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 375713,
            "range": "± 3457",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1821823,
            "range": "± 20235",
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
          "id": "feac6fe625c20018b030d48ebcf8b0405d8cd17a",
          "message": "feat(selfhost): add cross-verification tests — Rust vs selfhost compiler\n\nAdd cross-verification infrastructure comparing Rust compiler (vaisc) and\nselfhost compiler (vaisc-stage1) execution results for example programs.\n9/33 examples produce identical stdout and exit codes.\n\nKey changes:\n- Add ternary operator (?) lexing, parsing, and codegen to selfhost\n- Add self-recursion (@) codegen to selfhost\n- Extend cg struct to store current function name for @ calls\n- Create scripts/cross-verify.sh for shell-based verification\n- Create crates/vaisc/tests/cross_verify_tests.rs (10 Rust integration tests)\n- Update ROADMAP.md with cross-verification results\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T08:47:32+09:00",
          "tree_id": "5f492b5b6690ff0e7efed48bc39f00d6959ded16",
          "url": "https://github.com/vaislang/vais/commit/feac6fe625c20018b030d48ebcf8b0405d8cd17a"
        },
        "date": 1770422884638,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2394,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5290,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6110,
            "range": "± 290",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11504,
            "range": "± 168",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17714,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33795,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30173,
            "range": "± 861",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65876,
            "range": "± 706",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 267756,
            "range": "± 1559",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 407126,
            "range": "± 2881",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 99535,
            "range": "± 521",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 704754,
            "range": "± 1996",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 152859,
            "range": "± 5469",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 180626,
            "range": "± 1641",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 187055,
            "range": "± 1038",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 230094,
            "range": "± 1459",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 483023,
            "range": "± 23789",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 683513,
            "range": "± 2695",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 376513,
            "range": "± 3061",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1092716,
            "range": "± 14848",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37994,
            "range": "± 456",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 194000,
            "range": "± 1849",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 373961,
            "range": "± 3158",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1818201,
            "range": "± 25531",
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
          "id": "235b2deb94f093232e5788b0502103b90d6f3915",
          "message": "feat(build): add native dependencies, multifile build, and pkg install integration\n\n- Add [native-dependencies] section to vais.toml for declaring C/system library\n  dependencies with -l/-I/-L flags and C source file compilation support\n- Support directory input in `vaisc build src/` — auto-detect main.vais/lib.vais\n  entry point, set VAIS_DEP_PATHS for module resolution, resolve package deps\n- Wire native deps through compile_to_native for automatic clang flag injection\n- Add 4 integration tests (native deps parsing, directory build, multifile import)\n- Update ROADMAP.md Phase 36/37 remaining items as completed\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T09:20:51+09:00",
          "tree_id": "78a0f359211cdbca839efbd2d3a113b103f5a17c",
          "url": "https://github.com/vaislang/vais/commit/235b2deb94f093232e5788b0502103b90d6f3915"
        },
        "date": 1770424078740,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2390,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5566,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5995,
            "range": "± 313",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11429,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 18044,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34413,
            "range": "± 696",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30235,
            "range": "± 190",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 68114,
            "range": "± 667",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 270103,
            "range": "± 1298",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 408141,
            "range": "± 2120",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 101666,
            "range": "± 1085",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 707827,
            "range": "± 23832",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 153824,
            "range": "± 1547",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 179498,
            "range": "± 957",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 188114,
            "range": "± 3307",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 232462,
            "range": "± 1682",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 485031,
            "range": "± 1594",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 684762,
            "range": "± 2616",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 379401,
            "range": "± 1739",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1091752,
            "range": "± 26464",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38261,
            "range": "± 3500",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 193499,
            "range": "± 1472",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 375192,
            "range": "± 3192",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1824882,
            "range": "± 43657",
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
          "id": "6d6fa4613ac94ce9e38ef4159993975f107dc155",
          "message": "feat: add large-scale benchmarks, MIR infrastructure, CI miri, and production guide\n\n- Add 50K-line scaling benchmarks (criterion + shell script): 50K lines in 79ms\n- Add selfhost MIR core data structures (mir.vais, 659 LOC) and builder API (mir_builder.vais, 297 LOC)\n- Add miri unsafe code verification to CI fuzz workflow\n- Add TCP/async compilation benchmarks and memory stability tests\n- Add production deployment guide with Docker, CI/CD, and REST API tutorial\n- Complete Phase 36 Stage 6-8 and Phase 37 Stage 6-7\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T09:45:48+09:00",
          "tree_id": "8011add0a8967fa3cb3dbaa0c9a248797ab620fa",
          "url": "https://github.com/vaislang/vais/commit/6d6fa4613ac94ce9e38ef4159993975f107dc155"
        },
        "date": 1770425560097,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2408,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5404,
            "range": "± 218",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6062,
            "range": "± 311",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11345,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17779,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33996,
            "range": "± 233",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30777,
            "range": "± 311",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66284,
            "range": "± 445",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 271339,
            "range": "± 2902",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 414253,
            "range": "± 6015",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100585,
            "range": "± 860",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 717804,
            "range": "± 15817",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 151271,
            "range": "± 493",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 177569,
            "range": "± 770",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 186324,
            "range": "± 962",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 227358,
            "range": "± 735",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 482168,
            "range": "± 1407",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 680439,
            "range": "± 2292",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 377072,
            "range": "± 2033",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1086263,
            "range": "± 10558",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37695,
            "range": "± 540",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 194665,
            "range": "± 1396",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 377297,
            "range": "± 2620",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1832481,
            "range": "± 16537",
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
          "id": "17f554c005f194a6be827146b35aa256f2a66100",
          "message": "feat(selfhost): add MIR lowering — AST→MIR conversion (1,420 LOC)\n\nImplement mir_lower.vais for the self-hosting compiler's MIR pipeline.\nConverts parsed AST into MIR using the existing MirBuilder API, covering\nall 32 expression types, 5 statement types, control flow (if/loop/match/\nternary), function calls, aggregates, and break/continue via loop stack.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T10:02:29+09:00",
          "tree_id": "05eab6c026ec6285c40493f629ea92c83ac59ea8",
          "url": "https://github.com/vaislang/vais/commit/17f554c005f194a6be827146b35aa256f2a66100"
        },
        "date": 1770426552473,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2385,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5426,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6144,
            "range": "± 374",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11238,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17492,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34260,
            "range": "± 265",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30460,
            "range": "± 434",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66819,
            "range": "± 1057",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 268926,
            "range": "± 1097",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 407740,
            "range": "± 1647",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 101424,
            "range": "± 588",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 704452,
            "range": "± 3526",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 151630,
            "range": "± 5807",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 178849,
            "range": "± 4595",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 188182,
            "range": "± 5688",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 230224,
            "range": "± 1012",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 484373,
            "range": "± 4280",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 682156,
            "range": "± 2122",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 376422,
            "range": "± 1535",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1085491,
            "range": "± 15246",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38240,
            "range": "± 253",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 195579,
            "range": "± 2252",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 377228,
            "range": "± 46485",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1799839,
            "range": "± 18514",
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
          "id": "8e1b2f071f959f23c863af1e13fa2eb84fdbe5aa",
          "message": "feat(selfhost): add MIR borrow checker — ownership/borrow checking on MIR (1,357 LOC)\n\nImplements Rust-style ownership and borrow checking at the MIR level,\nusing liveness, dominance, and CFG analysis from mir_analysis.vais.\n\n- mir_borrow.vais: 7 checks (use-after-move, move-while-borrowed,\n  double-mut-borrow, mut/immut conflict, assign-while-borrowed,\n  dangling-ref, mut-borrow-of-immut) with NLL-style liveness-based\n  borrow invalidation\n- mir_test_borrow.vais: 7 test cases covering all check types\n- mir.vais: add mir_type_mut_ref() for &mut T (inner_count=1)\n- ROADMAP updates for Phase 39\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T10:59:39+09:00",
          "tree_id": "d5fb36e5b72059e120c66d69924dc4962c764023",
          "url": "https://github.com/vaislang/vais/commit/8e1b2f071f959f23c863af1e13fa2eb84fdbe5aa"
        },
        "date": 1770429998404,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2380,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5357,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5986,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11388,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17830,
            "range": "± 362",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33844,
            "range": "± 615",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30084,
            "range": "± 618",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 67199,
            "range": "± 1360",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 268871,
            "range": "± 5774",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 409439,
            "range": "± 7039",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 101233,
            "range": "± 516",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 706461,
            "range": "± 7489",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 152471,
            "range": "± 1160",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 178053,
            "range": "± 1010",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 186245,
            "range": "± 3067",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 226556,
            "range": "± 2622",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 480706,
            "range": "± 3003",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 678989,
            "range": "± 2314",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 373278,
            "range": "± 2637",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1084966,
            "range": "± 17257",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38030,
            "range": "± 516",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 191958,
            "range": "± 3368",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 372817,
            "range": "± 3812",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1807174,
            "range": "± 22208",
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
          "id": "4c7edd905341164a74daf8fddb79b98296004621",
          "message": "feat(selfhost): integrate MIR pipeline — Source→Lex→Parse→MIR→Opt→LLVM IR (~350 LOC)\n\n- Add mir_main.vais: full MIR compiler entry point with CLI flags (--no-opt, --stats)\n- Fix parser.vais: add built-in type token support (TOK_TY_I8~TOK_TY_STR), store params_len\n- Fix 136 loop-break bugs across 7 MIR modules: R 0 (function return) → B (loop break)\n- Fix Inkwell struct type inference ambiguity when multiple structs share LLVM layout\n- Fix lower_stmts to use inline 56-byte StmtNode access instead of pointer array\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T11:38:18+09:00",
          "tree_id": "0b4bd81e01bba152f9459eb7895dec0e28802033",
          "url": "https://github.com/vaislang/vais/commit/4c7edd905341164a74daf8fddb79b98296004621"
        },
        "date": 1770432293061,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2426,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5253,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6088,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11993,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17917,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34523,
            "range": "± 185",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30301,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 67435,
            "range": "± 1349",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 268380,
            "range": "± 1031",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 409117,
            "range": "± 3555",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100687,
            "range": "± 716",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 704548,
            "range": "± 2195",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 152028,
            "range": "± 726",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 177952,
            "range": "± 728",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 186888,
            "range": "± 706",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 228763,
            "range": "± 977",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 482922,
            "range": "± 1430",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 684759,
            "range": "± 7184",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 378612,
            "range": "± 2124",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1091639,
            "range": "± 7158",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 48198,
            "range": "± 285",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 198214,
            "range": "± 1645",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 380168,
            "range": "± 2981",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1841099,
            "range": "± 21380",
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
          "id": "6c10ee73306a4fa66d74e23427d9781f043b8d4c",
          "message": "feat(selfhost): add code formatter and doc generator, update ROADMAPs\n\n- Code formatter (1,475 LOC): AST-based pretty-printing with --check/--write modes\n  - fmt.vais (1,289 LOC) + fmt_main.vais (186 LOC)\n  - Handles all 32 EXPR + 5 STMT + 7 ITEM + 8 PAT + 12 TYPE nodes\n- Doc generator (1,236 LOC): Markdown output with signatures, field tables, doc comments\n  - doc_gen.vais (1,046 LOC) + doc_gen_main.vais (190 LOC)\n  - Extracts # comments from source and correlates with parsed items\n- selfhost/ROADMAP.md: rewrite to v0.8.0 reflecting all completed milestones\n- ROADMAP.md: Phase 22/26a status changed to ⏳ (non-code items remaining)\n- .gitignore: add selfhost/fmt_main and selfhost/doc_gen_main binaries\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T12:53:24+09:00",
          "tree_id": "f123ca01ad03cad11f55941fb19ad0721e3fc6f1",
          "url": "https://github.com/vaislang/vais/commit/6c10ee73306a4fa66d74e23427d9781f043b8d4c"
        },
        "date": 1770436855692,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2428,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5624,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6187,
            "range": "± 239",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 12993,
            "range": "± 244",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17636,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33837,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30114,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65711,
            "range": "± 384",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 268809,
            "range": "± 1544",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 406816,
            "range": "± 2204",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100559,
            "range": "± 495",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 704817,
            "range": "± 3315",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 150556,
            "range": "± 806",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 177223,
            "range": "± 955",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 186278,
            "range": "± 2189",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 227651,
            "range": "± 1273",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 480123,
            "range": "± 1522",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 679256,
            "range": "± 6110",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 373985,
            "range": "± 1582",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1085844,
            "range": "± 6617",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 39054,
            "range": "± 695",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 196995,
            "range": "± 1614",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 379526,
            "range": "± 3636",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1815006,
            "range": "± 16868",
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
          "id": "77e17db39ed5995cb41ef375c8bd7d224f6142b8",
          "message": "feat(selfhost): add stdlib modules — Vec, String, HashMap, defer, file I/O, print\n\nAdd 6 new selfhost standard library modules totaling ~1,460 LOC:\n- vec.vais (295 LOC): dynamic array with push/pop/sort/reverse/clone\n- string.vais (351 LOC): owned string with eq/concat/slice/find/trim\n- hashmap.vais (333 LOC): open addressing hash map with put/get/remove\n- file_io.vais (249 LOC): file read/write/exists wrappers around C builtins\n- print.vais (229 LOC): enhanced output — i64/bool/hex/char/repeat\n\nImplement defer statement (D expr) in selfhost codegen:\n- Parser: STMT_DEFER parsing for D keyword\n- Codegen: defer stack with LIFO cleanup at return/function exit\n- Tested: single defer, multiple defers (LIFO order), early return\n\nAlso add memcmp as Inkwell/type-checker builtin for string comparison.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T14:51:13+09:00",
          "tree_id": "861b83ebeac42cc04cd861bee4cf635ddc6c9d3b",
          "url": "https://github.com/vaislang/vais/commit/77e17db39ed5995cb41ef375c8bd7d224f6142b8"
        },
        "date": 1770443887137,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2382,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5296,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6015,
            "range": "± 275",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10929,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17690,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34000,
            "range": "± 333",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29764,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66103,
            "range": "± 630",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 269129,
            "range": "± 4107",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 410834,
            "range": "± 2506",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100971,
            "range": "± 523",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 707231,
            "range": "± 2782",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 151233,
            "range": "± 1028",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 178039,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 186419,
            "range": "± 1044",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 229424,
            "range": "± 1038",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 484309,
            "range": "± 1575",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 682616,
            "range": "± 3306",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 377695,
            "range": "± 1606",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1087317,
            "range": "± 16111",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38302,
            "range": "± 424",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 186933,
            "range": "± 3901",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 384477,
            "range": "± 4277",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1818733,
            "range": "± 19959",
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
          "id": "fba6404083bba33a2c2e0b9303fdf02916c84685",
          "message": "test(selfhost): add stdlib test suites — 276 assertions across 6 modules\n\nAdd comprehensive test suites for all 6 selfhost stdlib modules:\n- Vec (103 assertions), String (58), HashMap (50), Option/Result (32),\n  File I/O (12), Print (21) — all passing\n\nFix 3 bugs found during testing:\n- file_io.vais: fopen_ptr return value discarded when used as last expr\n- file_io.vais: feof() i32→i64 garbage bits causing false EOF detection\n- test_print.vais: compiler builtin println() collision with user function\n\nAdd Rust E2E integration (selfhost_stdlib_tests.rs, 6 tests).\nUpdate ROADMAPs with Phase 40 completion.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T15:28:14+09:00",
          "tree_id": "c00221f7694438e3607bb5a79735ce55b5b1ac18",
          "url": "https://github.com/vaislang/vais/commit/fba6404083bba33a2c2e0b9303fdf02916c84685"
        },
        "date": 1770446098085,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 1990,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5156,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5737,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10417,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 16676,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 32326,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 28888,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 63828,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 254978,
            "range": "± 1675",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 384520,
            "range": "± 3472",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 106194,
            "range": "± 754",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 658062,
            "range": "± 5056",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 156167,
            "range": "± 1199",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 184584,
            "range": "± 716",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 193184,
            "range": "± 495",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 234122,
            "range": "± 899",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 471220,
            "range": "± 2359",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 657301,
            "range": "± 2884",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 388375,
            "range": "± 1812",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1033678,
            "range": "± 3430",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38988,
            "range": "± 203",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 186198,
            "range": "± 1432",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 360136,
            "range": "± 2308",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1744180,
            "range": "± 20713",
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
          "id": "12766d2f675e5258d6bef47c3e6a18e6f34d74d6",
          "message": "feat(selfhost): add pointer and reference types — *T, &T, &mut T, &expr, *expr\n\nParser: *T type parsing, &mut T support, &expr/&*expr expression parsing\nType Checker: TYPE_REF/TYPE_REF_MUT resolution in resolve_type_node\nCodegen: *ptr = val (deref assignment), inttoptr fix for Inkwell deref\nTests: 6/6 pointer assertions, 248 E2E + 7 selfhost stdlib all passing\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T17:08:01+09:00",
          "tree_id": "fc8a8a75064a9360a3721a18bf1df067222385b1",
          "url": "https://github.com/vaislang/vais/commit/12766d2f675e5258d6bef47c3e6a18e6f34d74d6"
        },
        "date": 1770452077063,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2391,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5460,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5988,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11811,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17690,
            "range": "± 283",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34660,
            "range": "± 445",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30332,
            "range": "± 387",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66963,
            "range": "± 692",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 267488,
            "range": "± 1048",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 408394,
            "range": "± 1092",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 99553,
            "range": "± 796",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 703686,
            "range": "± 4012",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 152488,
            "range": "± 844",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 180189,
            "range": "± 1018",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 188935,
            "range": "± 3783",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 230683,
            "range": "± 1157",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 488674,
            "range": "± 5576",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 684809,
            "range": "± 4546",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 379858,
            "range": "± 1441",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1090311,
            "range": "± 18655",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37746,
            "range": "± 276",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 194003,
            "range": "± 1267",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 375387,
            "range": "± 2778",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1822528,
            "range": "± 19703",
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
          "id": "ab94b5d47d02f7bd0152c8d39519eea0bde949e7",
          "message": "docs: consolidate ROADMAP.md — compress Phase 28-40, add Phase 41\n\n- Reduce ROADMAP.md from 1,809 to 312 lines (83% reduction)\n- Compress completed Phase 28-40 detailed checklists into summary table\n- Move long-term observation items (TCP benchmark, memory leak test) to dedicated section\n- Add Phase 41: v2.0 roadmap (error recovery, closures, iterators, package ecosystem)\n- Remove redundant \"project review\" section\n- Sync selfhost/ROADMAP.md with next phase reference\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T17:17:00+09:00",
          "tree_id": "e0bdab3766f11171255a52e31929dc6c5c906fff",
          "url": "https://github.com/vaislang/vais/commit/ab94b5d47d02f7bd0152c8d39519eea0bde949e7"
        },
        "date": 1770452708155,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2022,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5279,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5906,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10539,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 16709,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 32657,
            "range": "± 479",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 28966,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 63448,
            "range": "± 579",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 252589,
            "range": "± 665",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 378259,
            "range": "± 4990",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 107506,
            "range": "± 1663",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 643744,
            "range": "± 2579",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 155368,
            "range": "± 833",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 183377,
            "range": "± 1091",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 192211,
            "range": "± 9098",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 232739,
            "range": "± 2646",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 470304,
            "range": "± 1282",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 658664,
            "range": "± 2714",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 388263,
            "range": "± 1802",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1038041,
            "range": "± 8436",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 39027,
            "range": "± 234",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 184178,
            "range": "± 1545",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 360741,
            "range": "± 3600",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1776481,
            "range": "± 12678",
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
          "id": "90451434163059fc77fc80e642382ed2709d7366",
          "message": "feat(selfhost): add error recovery parser — multi-error collection, sync, line:col reporting\n\n- Add recovery_mode to selfhost Parser: collects up to 20 errors instead of stopping at first\n- Implement synchronize_to_item() and synchronize_to_stmt() for token-level recovery\n- Add offset_to_line/offset_to_col utilities for byte offset → line:col conversion\n- Add print_errors() method with formatted \"line:col: error: message\" output\n- Add p_sb_* string buffer functions for error message formatting\n- Add 10 E2E error recovery tests (258 total E2E tests, all passing)\n- ROADMAP: mark Phase 41 Stage 1 complete\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T17:33:44+09:00",
          "tree_id": "fa6983487a1d9e747a8ed685b3e36302582b3e7a",
          "url": "https://github.com/vaislang/vais/commit/90451434163059fc77fc80e642382ed2709d7366"
        },
        "date": 1770453638411,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2462,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5228,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5858,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11103,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17777,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34075,
            "range": "± 275",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29466,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66829,
            "range": "± 357",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 274403,
            "range": "± 1065",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 420106,
            "range": "± 1851",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 99371,
            "range": "± 326",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 726849,
            "range": "± 3796",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 151102,
            "range": "± 635",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 177561,
            "range": "± 735",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 185915,
            "range": "± 1046",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 227457,
            "range": "± 1174",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 490097,
            "range": "± 1895",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 692697,
            "range": "± 4397",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 376709,
            "range": "± 1447",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1107878,
            "range": "± 7006",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37649,
            "range": "± 254",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 188602,
            "range": "± 1315",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 369169,
            "range": "± 2467",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1787506,
            "range": "± 21965",
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
          "id": "c98aa6b6fb78141b3be8d67efc36ae2e7f5d68d5",
          "message": "test: reach 301 E2E tests — add 9 tests for recursion, bitwise, closures, enums, pipelines\n\nNew tests: recursive_fibonacci, self_recursion_operator, bitwise_operations,\nmultiple_return_paths, closure_compose_apply_twice, mutable_accumulator_pattern,\nstruct_method_chaining, enum_tag_matching, higher_order_pipeline\n\nTotal: 301 E2E tests, all passing (Stage 6 complete)\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T18:44:35+09:00",
          "tree_id": "88ad8cf575a1833816b7eb8b2b461b2feab56dfa",
          "url": "https://github.com/vaislang/vais/commit/c98aa6b6fb78141b3be8d67efc36ae2e7f5d68d5"
        },
        "date": 1770458015044,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2430,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5378,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6156,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11166,
            "range": "± 172",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17948,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34137,
            "range": "± 284",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30095,
            "range": "± 316",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66291,
            "range": "± 461",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 270705,
            "range": "± 910",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 412051,
            "range": "± 2407",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100545,
            "range": "± 463",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 713341,
            "range": "± 2344",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 152263,
            "range": "± 6458",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 180150,
            "range": "± 730",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 188088,
            "range": "± 1057",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 231016,
            "range": "± 1043",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 488962,
            "range": "± 3646",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 689352,
            "range": "± 3251",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 382654,
            "range": "± 1460",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1096555,
            "range": "± 9724",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38761,
            "range": "± 533",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 195414,
            "range": "± 2846",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 377044,
            "range": "± 3059",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1832814,
            "range": "± 19302",
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
          "id": "b967561a525a34f86856b65189a96c5975691bc7",
          "message": "feat: add package ecosystem — vaisc new/test, pkg tree/doc + 14 E2E tests\n\n- vaisc new <name>: project scaffolding with vais.toml, src/main.vais,\n  tests/, .gitignore; --lib flag for library projects\n- vaisc test [path]: auto-discover .vais test files, compile+run,\n  pass/fail summary; --filter name pattern, --verbose output\n- vaisc pkg tree: dependency tree visualization with ├──/└── formatting,\n  transitive deps, --depth limit\n- vaisc pkg doc: auto-generate markdown/html docs for all src/*.vais,\n  index file with module listing\n- 14 new E2E tests covering all Stage 5 features (78 registry tests total)\n- ROADMAP.md Stage 5 marked complete\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T19:00:04+09:00",
          "tree_id": "280d20f93c781f16f0c818c0856319f536f4a61c",
          "url": "https://github.com/vaislang/vais/commit/b967561a525a34f86856b65189a96c5975691bc7"
        },
        "date": 1770458817156,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2426,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5367,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6081,
            "range": "± 432",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11599,
            "range": "± 354",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17863,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33633,
            "range": "± 276",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30146,
            "range": "± 568",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 68254,
            "range": "± 618",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 269310,
            "range": "± 2333",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 409578,
            "range": "± 2269",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 99283,
            "range": "± 604",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 705656,
            "range": "± 2840",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 150708,
            "range": "± 444",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 177950,
            "range": "± 491",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 185599,
            "range": "± 685",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 228036,
            "range": "± 1003",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 483031,
            "range": "± 1767",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 684000,
            "range": "± 1517",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 376849,
            "range": "± 1469",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1093284,
            "range": "± 9047",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38554,
            "range": "± 235",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 195457,
            "range": "± 1398",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 375579,
            "range": "± 2509",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1823816,
            "range": "± 13775",
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
          "id": "fc94b9a4963085a2e91a855a4f1640f5c9222a8f",
          "message": "fix: resolve 5 compiler bugs + update playground examples and docs\n\nParser fixes:\n- Fix tilde-mut (~) prefix not recognized in is_let_stmt()\n- Fix cross-line postfix parsing: prevent `(` on new line from being\n  parsed as function call (fixes consecutive tuple destructuring)\n- Fix cross-line binary ops: prevent `-` on new line from being parsed\n  as subtraction (fixes `-1` after puts() in match arms)\n\nType system fix:\n- Fix E022 false positive on enum match scrutinee — remove duplicate\n  move check that incorrectly flagged single-use enum params\n\nOther:\n- Add Token::Yield to selfhost lexer test mapping\n- Fix clippy: map_or → is_none_or\n- Fix playground examples: type-infer-params uses explicit types\n- Update LANGUAGE_SPEC.md: document yield, iterators, error handling\n  (?/! operators), derive(Error), package ecosystem, enum impl blocks\n\nAll 301 E2E tests pass. 114 selfhost lexer tests pass. 0 clippy warnings.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T19:35:45+09:00",
          "tree_id": "b9027b656cdabb20df8e49efdef7baa14a6561e9",
          "url": "https://github.com/vaislang/vais/commit/fc94b9a4963085a2e91a855a4f1640f5c9222a8f"
        },
        "date": 1770460959171,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 1996,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 4987,
            "range": "± 222",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5870,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10178,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 16798,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 32585,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29237,
            "range": "± 589",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 63013,
            "range": "± 427",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 252980,
            "range": "± 2125",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 378560,
            "range": "± 2416",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 107369,
            "range": "± 578",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 644894,
            "range": "± 3249",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 155911,
            "range": "± 594",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 183880,
            "range": "± 5589",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 193510,
            "range": "± 588",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 234508,
            "range": "± 1407",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 470235,
            "range": "± 2773",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 654735,
            "range": "± 3083",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 388634,
            "range": "± 1855",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1038021,
            "range": "± 27289",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 36865,
            "range": "± 277",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 181993,
            "range": "± 1396",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 356327,
            "range": "± 2856",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1750373,
            "range": "± 17665",
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
          "id": "102ac996a6570353c0d56ca0f9b00a8a15ba1888",
          "message": "docs: mark Phase 41 Stage 7 as complete — all CI workflows already implemented\n\nbench.yml + bench-regression.yml already cover:\n- criterion benchmarks on PR (auto-run)\n- PR comment with performance comparison table\n- 10% regression threshold → CI failure\n- GitHub Pages benchmark dashboard (benchmark-action)\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T20:27:36+09:00",
          "tree_id": "f93cd46088eab1285477e3a207fdc85ad6985075",
          "url": "https://github.com/vaislang/vais/commit/102ac996a6570353c0d56ca0f9b00a8a15ba1888"
        },
        "date": 1770464100942,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2408,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5334,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6087,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11093,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17707,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33762,
            "range": "± 252",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30555,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65802,
            "range": "± 531",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 268189,
            "range": "± 2077",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 407893,
            "range": "± 5196",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100192,
            "range": "± 436",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 707200,
            "range": "± 2777",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 150548,
            "range": "± 706",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 178395,
            "range": "± 712",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 185832,
            "range": "± 913",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 229119,
            "range": "± 1676",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 482669,
            "range": "± 1930",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 679546,
            "range": "± 2934",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 376624,
            "range": "± 2637",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1085702,
            "range": "± 7956",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37946,
            "range": "± 283",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 193463,
            "range": "± 1718",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 375405,
            "range": "± 2996",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1819432,
            "range": "± 16601",
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
          "id": "816ea5ae8b8e72c34b3e92a4dfa1a8eed5c34044",
          "message": "refactor: remove unused dependencies and redundant std files\n\n- Remove unused workspace dependencies: ariadne, codespan-reporting\n- Upgrade workspace colored from v2.1 to v3, unify vaisc to use workspace ref\n- Remove redundant std/iter_minimal.vais and std/iter_simple.vais (iter.vais is canonical)\n- Remove redundant std/test_simple.vais (test.vais is canonical)\n- Also cleaned 181 untracked .ll files (13MB) and selfhost/runtime.o locally\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T20:38:26+09:00",
          "tree_id": "ec9437527e61130ea2040ace49c6eb6ea19258be",
          "url": "https://github.com/vaislang/vais/commit/816ea5ae8b8e72c34b3e92a4dfa1a8eed5c34044"
        },
        "date": 1770464707215,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2424,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5145,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6186,
            "range": "± 185",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11223,
            "range": "± 121",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17627,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33986,
            "range": "± 258",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30470,
            "range": "± 1223",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66798,
            "range": "± 286",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 265771,
            "range": "± 815",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 405092,
            "range": "± 10594",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100169,
            "range": "± 542",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 702932,
            "range": "± 3500",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 151187,
            "range": "± 581",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 178777,
            "range": "± 763",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 186980,
            "range": "± 764",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 227862,
            "range": "± 1142",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 483282,
            "range": "± 1323",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 679032,
            "range": "± 2179",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 378750,
            "range": "± 1024",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1080023,
            "range": "± 7691",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38111,
            "range": "± 263",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 196546,
            "range": "± 1583",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 379180,
            "range": "± 8635",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1844379,
            "range": "± 14073",
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
          "id": "a57772c230594cd941e8a6037bfe32665800313e",
          "message": "fix: resolve CI failures — cargo fmt, _GNU_SOURCE, playground mock mode\n\n- Run cargo fmt across workspace (12 files reformatted)\n- Add #define _GNU_SOURCE to std/thread_runtime.c for pthread_tryjoin_np on Linux\n- Improve playground mock mode: better syntax validation (parens, strings),\n  simulate output from puts/println calls, show construct summary,\n  display clear \"preview mode\" message when server is offline\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T20:45:49+09:00",
          "tree_id": "859058466023a419873353ed9ad94a0ac12681be",
          "url": "https://github.com/vaislang/vais/commit/a57772c230594cd941e8a6037bfe32665800313e"
        },
        "date": 1770465150970,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2414,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5377,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5619,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11584,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17736,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34346,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30514,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66705,
            "range": "± 6764",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 266870,
            "range": "± 1156",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 406456,
            "range": "± 1831",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100981,
            "range": "± 542",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 703134,
            "range": "± 4671",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 152797,
            "range": "± 702",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 179798,
            "range": "± 3047",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 186943,
            "range": "± 745",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 228510,
            "range": "± 1083",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 482196,
            "range": "± 1614",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 681721,
            "range": "± 6863",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 378294,
            "range": "± 1226",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1083881,
            "range": "± 13603",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38920,
            "range": "± 926",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 195130,
            "range": "± 1176",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 380643,
            "range": "± 2535",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1843971,
            "range": "± 11714",
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
          "id": "509149b87cf9ec982d9fbae49c808574f1814de2",
          "message": "fix: use platform-correct stdin/stdout symbols in Inkwell builtins\n\n- macOS uses __stdinp/__stdoutp, Linux uses stdin/stdout\n- get_stdin() and get_stdout() now select symbol via cfg!(target_os)\n- Fixes linker errors on Linux CI for selfhost stdlib tests\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T20:56:04+09:00",
          "tree_id": "efb4a32da8ce23a69f612dd8539711c34cf01429",
          "url": "https://github.com/vaislang/vais/commit/509149b87cf9ec982d9fbae49c808574f1814de2"
        },
        "date": 1770465778488,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2955,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 6357,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 7454,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 13197,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17815,
            "range": "± 233",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34346,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29854,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66018,
            "range": "± 370",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 267558,
            "range": "± 1404",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 409122,
            "range": "± 1425",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100683,
            "range": "± 517",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 706455,
            "range": "± 2656",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 151362,
            "range": "± 679",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 179241,
            "range": "± 970",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 187742,
            "range": "± 917",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 229836,
            "range": "± 2282",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 489478,
            "range": "± 1715",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 689657,
            "range": "± 3191",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 385384,
            "range": "± 1823",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1089505,
            "range": "± 18034",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 44421,
            "range": "± 260",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 228836,
            "range": "± 4865",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 442849,
            "range": "± 9430",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 2146702,
            "range": "± 57244",
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
          "id": "256b4ae494a75da3fbf839cec2100545909fe693",
          "message": "fix: macOS CI — reduce fuzz nesting depth to prevent stack overflow\n\n- fuzz_deeply_nested_expressions: reduce max depth 20→15 (macOS smaller stack)\n- Fixes stack overflow abort on macOS CI runners\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T21:05:24+09:00",
          "tree_id": "c6a9a9af96bfa9be77ba61510f5f4df5dc649d7b",
          "url": "https://github.com/vaislang/vais/commit/256b4ae494a75da3fbf839cec2100545909fe693"
        },
        "date": 1770466327866,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2405,
            "range": "± 323",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5195,
            "range": "± 579",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5926,
            "range": "± 723",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11574,
            "range": "± 931",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17876,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33910,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30175,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66173,
            "range": "± 335",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 266689,
            "range": "± 1335",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 404882,
            "range": "± 13768",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100524,
            "range": "± 858",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 700974,
            "range": "± 3070",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 152095,
            "range": "± 1172",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 179351,
            "range": "± 775",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 188374,
            "range": "± 5738",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 229169,
            "range": "± 2529",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 485902,
            "range": "± 4087",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 684137,
            "range": "± 9429",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 380871,
            "range": "± 3068",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1084465,
            "range": "± 8284",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38829,
            "range": "± 4389",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 196076,
            "range": "± 21618",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 379910,
            "range": "± 43307",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1845704,
            "range": "± 215658",
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
          "id": "4ed067149b4b0ad7bbf42edc452b6da499a723d0",
          "message": "fix: merge global profiler tests to prevent race condition on CI\n\nThe two global profiler tests shared a process-wide Mutex state and\nwould race when cargo test ran them in parallel, causing init to fail\nand sample_count to remain 0 on ubuntu CI.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T21:16:01+09:00",
          "tree_id": "a15b257876c22b12c0bf847af94e14a264eb8ec5",
          "url": "https://github.com/vaislang/vais/commit/4ed067149b4b0ad7bbf42edc452b6da499a723d0"
        },
        "date": 1770466988966,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2402,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5311,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6278,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11678,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 18225,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34402,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30495,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 68788,
            "range": "± 1115",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 266976,
            "range": "± 1067",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 405804,
            "range": "± 10086",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 101572,
            "range": "± 820",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 703418,
            "range": "± 4127",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 152648,
            "range": "± 2720",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 179247,
            "range": "± 911",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 188371,
            "range": "± 1386",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 230626,
            "range": "± 1553",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 485856,
            "range": "± 2332",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 684033,
            "range": "± 3466",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 379178,
            "range": "± 1517",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1093518,
            "range": "± 20238",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38054,
            "range": "± 634",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 195359,
            "range": "± 3504",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 378269,
            "range": "± 2503",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1842183,
            "range": "± 15765",
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
          "id": "409b409af5f159a76780d29e1d909019345cdc71",
          "message": "fix: remove broken codecov badge from README\n\nThe codecov badge used a placeholder token (YOUR_CODECOV_TOKEN) and\ndisplayed as broken/unknown on the repository homepage.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T21:38:42+09:00",
          "tree_id": "0a2312e570571bce19b08024158bb87b790ffc2f",
          "url": "https://github.com/vaislang/vais/commit/409b409af5f159a76780d29e1d909019345cdc71"
        },
        "date": 1770468332432,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2388,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5102,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6110,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10679,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17975,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33894,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30428,
            "range": "± 508",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 67762,
            "range": "± 1243",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 269271,
            "range": "± 1774",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 407581,
            "range": "± 2155",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100812,
            "range": "± 682",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 704754,
            "range": "± 16302",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 152338,
            "range": "± 536",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 181190,
            "range": "± 2032",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 188851,
            "range": "± 1088",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 229417,
            "range": "± 3227",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 483686,
            "range": "± 1448",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 681862,
            "range": "± 2645",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 378607,
            "range": "± 2833",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1085479,
            "range": "± 16098",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37693,
            "range": "± 489",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 187785,
            "range": "± 2964",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 384194,
            "range": "± 2939",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1789565,
            "range": "± 17565",
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
          "id": "23a083cf67c9788fde89f0a0801f94160edb3369",
          "message": "fix: redeploy playground server with latest vaisc + fix enum example\n\n- Update Dockerfile.playground: Rust 1.89, LLVM 17 in both build and\n  runtime stages (clang-17 needed for opaque pointer support)\n- Add fly.playground.toml for Fly.io deployment config\n- Add deploy-playground-server.yml CI workflow for auto-deployment\n- Fix enum playground example: avoid match+enum combo that causes\n  LLVM crash on server (clang-17/x86_64 opaque pointer issue)\n\nAll 18 playground examples now pass on api.vaislang.dev.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T22:10:48+09:00",
          "tree_id": "4fbde9833a996a74f9b4ee14e7de9f011540eb3b",
          "url": "https://github.com/vaislang/vais/commit/23a083cf67c9788fde89f0a0801f94160edb3369"
        },
        "date": 1770470256674,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2440,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5165,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6783,
            "range": "± 656",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10965,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17871,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34020,
            "range": "± 321",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30123,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66781,
            "range": "± 1493",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 267762,
            "range": "± 1250",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 407556,
            "range": "± 2032",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 101635,
            "range": "± 9748",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 709078,
            "range": "± 10578",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 152156,
            "range": "± 830",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 180610,
            "range": "± 1089",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 187941,
            "range": "± 685",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 231261,
            "range": "± 1347",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 485830,
            "range": "± 2661",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 683239,
            "range": "± 3875",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 381745,
            "range": "± 2795",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1088417,
            "range": "± 9995",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38740,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 195637,
            "range": "± 5747",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 377089,
            "range": "± 2407",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1831271,
            "range": "± 13806",
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
          "id": "1da5256ca56803022ba02983732a764c3f2363ae",
          "message": "fix: resolve enum+match LLVM crash — add fallthrough block for valid phi nodes\n\nThe Inkwell codegen's generate_match() had the last arm's false branch\ngoing directly to merge_block without a phi entry, producing invalid\nLLVM IR that crashed clang-17 on Linux. Fixed by creating a dedicated\nfallthrough block with a default value. Updated playground enum example\nto use proper match on enum variants.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T22:22:34+09:00",
          "tree_id": "453e79e2c901933f5748a67cfc04356c93d783d6",
          "url": "https://github.com/vaislang/vais/commit/1da5256ca56803022ba02983732a764c3f2363ae"
        },
        "date": 1770470954059,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2409,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5213,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5842,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 12116,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17765,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34385,
            "range": "± 730",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29754,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66577,
            "range": "± 1557",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 267316,
            "range": "± 1063",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 410120,
            "range": "± 1653",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 101155,
            "range": "± 678",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 705381,
            "range": "± 10083",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 151856,
            "range": "± 734",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 178370,
            "range": "± 857",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 187118,
            "range": "± 745",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 227958,
            "range": "± 1547",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 483324,
            "range": "± 2372",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 681309,
            "range": "± 2597",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 377716,
            "range": "± 6830",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1084189,
            "range": "± 16317",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38672,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 196217,
            "range": "± 2602",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 380503,
            "range": "± 2632",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 3157289,
            "range": "± 66088",
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
          "id": "424362df4927925f4f7cb9265d583a4e0ebe427e",
          "message": "docs: update benchmark numbers — 50K lines 63ms (800K lines/s), 21% faster\n\nCriterion benchmarks show compilation improved from 79ms to 62.6ms for\n50K lines (throughput: 641K → 800K lines/sec). Updated README, ROADMAP,\nand docs/benchmarks.md with latest figures. Also updated E2E test count\nfrom 258 to 307.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-07T22:41:17+09:00",
          "tree_id": "432d6cd14ee4d1577298c6231f480f476e633ee0",
          "url": "https://github.com/vaislang/vais/commit/424362df4927925f4f7cb9265d583a4e0ebe427e"
        },
        "date": 1770472105530,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2006,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 4889,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5762,
            "range": "± 319",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 10549,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 16574,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 32564,
            "range": "± 249",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29460,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 62871,
            "range": "± 354",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 247605,
            "range": "± 1225",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 370133,
            "range": "± 1377",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 106365,
            "range": "± 452",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 631389,
            "range": "± 8538",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 155308,
            "range": "± 980",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 183472,
            "range": "± 925",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 192631,
            "range": "± 788",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 233943,
            "range": "± 1431",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 466659,
            "range": "± 1925",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 650486,
            "range": "± 3702",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 387031,
            "range": "± 1939",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1029478,
            "range": "± 22395",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38144,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 184697,
            "range": "± 2272",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 357557,
            "range": "± 2625",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1770491,
            "range": "± 29024",
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
          "id": "c0fd420bd6bbe77c50d2cc91d1e65a9c73aa370b",
          "message": "feat: add per-module codegen with --per-module flag (Phase 42 Stage 1)\n\nSplit monolithic codegen pipeline into per-module compilation:\n- Module struct gets modules_map for source→item tracking\n- generate_module_subset() produces independent .ll per module\n- Cross-module extern declarations auto-generated\n- compile_per_module() orchestrates .ll→.o→binary linking\n- String prefix prevents constant naming collisions\n- All 307 E2E + 54 unit + 34 formatter + 128 integration tests pass\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-08T00:19:28+09:00",
          "tree_id": "7379c3b3bc633a0fe3ff45ec526e2fa888c06f06",
          "url": "https://github.com/vaislang/vais/commit/c0fd420bd6bbe77c50d2cc91d1e65a9c73aa370b"
        },
        "date": 1770477979191,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2397,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5296,
            "range": "± 575",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6159,
            "range": "± 583",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11557,
            "range": "± 983",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17844,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33981,
            "range": "± 190",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30200,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 68153,
            "range": "± 309",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 268939,
            "range": "± 1682",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 409494,
            "range": "± 1363",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 101192,
            "range": "± 2757",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 709232,
            "range": "± 2471",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 152037,
            "range": "± 658",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 179533,
            "range": "± 591",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 189835,
            "range": "± 904",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 230216,
            "range": "± 1029",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 484154,
            "range": "± 8433",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 684487,
            "range": "± 9993",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 377398,
            "range": "± 1440",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1091622,
            "range": "± 19983",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38651,
            "range": "± 4282",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 196278,
            "range": "± 21230",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 378871,
            "range": "± 43113",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1821507,
            "range": "± 211772",
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
          "id": "8a8bc8f6f28130b882eca99122d78bf1069a3561",
          "message": "bench: add Rust/Python benchmarks, update docs with actual measurements\n\nAdd benchmark.rs and benchmark.py for cross-language comparison.\nUpdate docs/benchmarks.md with real measured results:\n- Vais -O2 matches C/Rust exactly (1.00x on fib(40))\n- Binary size: Vais 58KB vs Rust 433KB (7.5x smaller)\n- Token analysis: honest GPT-4 tokenizer comparison\nUpdate run_bench.sh to support all 4 languages.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-08T00:33:35+09:00",
          "tree_id": "74c53fcfc45c55ebbe4af0f547d24725045f1e31",
          "url": "https://github.com/vaislang/vais/commit/8a8bc8f6f28130b882eca99122d78bf1069a3561"
        },
        "date": 1770478823440,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2385,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5286,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6206,
            "range": "± 291",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11144,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17957,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33821,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30152,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 66505,
            "range": "± 820",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 271716,
            "range": "± 1732",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 412515,
            "range": "± 1779",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100904,
            "range": "± 669",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 711077,
            "range": "± 3298",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 152440,
            "range": "± 1204",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 179688,
            "range": "± 700",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 187961,
            "range": "± 2682",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 233804,
            "range": "± 7260",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 482821,
            "range": "± 1403",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 683163,
            "range": "± 2557",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 378008,
            "range": "± 15238",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1090922,
            "range": "± 19736",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37985,
            "range": "± 243",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 193551,
            "range": "± 1799",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 374195,
            "range": "± 2751",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1824148,
            "range": "± 30909",
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
          "id": "b884c29978b9329a49db4b8608f196c9ca47970b",
          "message": "feat: complete Phase 42 incremental compilation (Stage 3~5)\n\nStage 3 - Stability & Automation:\n- Auto-enable per-module for multi-file projects (no --per-module needed)\n- Atomic cache writes (tempfile + rename pattern)\n- Circular import detection with clear error messages\n- --emit-ir --per-module support for per-module .ll output\n- Cache size limit with LRU cleanup (--cache-limit, default 512MB)\n\nStage 4 - Internal Optimization:\n- Remove empty_blocks double clone in optimize.rs\n- Replace clone() with std::mem::take() in parallel.rs\n- Optimize substitute_type() with early-return when unchanged\n- Add modules_map index bounds validation\n\nStage 5 - Benchmark Verification:\n- 5 new per-module E2E tests (312 total)\n- Incremental benchmark: 30K lines 1-file change 571ms → 96ms (5.9x)\n- BASELINE.md updated with Phase 42 results\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-08T07:00:41+09:00",
          "tree_id": "6055eb4326f001eca195e8a56d2af4304ea696f2",
          "url": "https://github.com/vaislang/vais/commit/b884c29978b9329a49db4b8608f196c9ca47970b"
        },
        "date": 1770502041061,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2407,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5525,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6157,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11484,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17829,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33860,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30288,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 67272,
            "range": "± 342",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 268198,
            "range": "± 871",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 408905,
            "range": "± 1860",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 99796,
            "range": "± 374",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 704862,
            "range": "± 6284",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 152378,
            "range": "± 559",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 180544,
            "range": "± 669",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 189099,
            "range": "± 984",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 229384,
            "range": "± 868",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 489899,
            "range": "± 1977",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 683738,
            "range": "± 2524",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 380018,
            "range": "± 1118",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1092416,
            "range": "± 13613",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38349,
            "range": "± 264",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 196500,
            "range": "± 1375",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 380589,
            "range": "± 2637",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1846052,
            "range": "± 18404",
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
          "id": "449fae3079baf0afae4fc16f14dea1e5db8f498a",
          "message": "fix: migrate vais-python/vais-node bindings to PyO3 0.28 and NAPI 3.x\n\n- vais-python: PyObject → Py<PyAny>, Python::with_gil → py param,\n  #[pyclass(skip_from_py_object)] for deprecated warning, Token::Yield added\n- vais-node: Object → ParseResult struct (#[napi(object)]),\n  create_object/create_array → struct return, Token::Yield added\n- Both crates now pass cargo check with 0 errors and 0 warnings\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-08T09:49:58+09:00",
          "tree_id": "3ed6660600aafe961ce6143b19521455e26ef28c",
          "url": "https://github.com/vaislang/vais/commit/449fae3079baf0afae4fc16f14dea1e5db8f498a"
        },
        "date": 1770512211645,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2407,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5486,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6099,
            "range": "± 321",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11125,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17821,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 33860,
            "range": "± 1614",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29943,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65717,
            "range": "± 343",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 269551,
            "range": "± 1395",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 411089,
            "range": "± 2000",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 99776,
            "range": "± 678",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 707724,
            "range": "± 2928",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 152645,
            "range": "± 1299",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 179599,
            "range": "± 814",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 188562,
            "range": "± 1031",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 228839,
            "range": "± 1395",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 485446,
            "range": "± 2069",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 683674,
            "range": "± 2719",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 378904,
            "range": "± 1729",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1088858,
            "range": "± 17255",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 39018,
            "range": "± 999",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 197933,
            "range": "± 1800",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 382432,
            "range": "± 3354",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1841359,
            "range": "± 23447",
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
          "id": "92d02ead12faf17d9dcb112a2d15042d848075e5",
          "message": "feat: support nested struct field access (Phase 44)\n\n- Add struct_field_type_names map to Inkwell codegen for resolving\n  intermediate struct types in nested field access (e.g., outer.inner.val)\n- Fix infer_struct_name(Expr::Field) to look up field type names\n  instead of returning Unsupported error\n- Parser and type checker already supported nested FieldAccess via\n  recursive AST structure and check_expr recursion\n- Add 3 E2E tests: 2-level, 3-level, and arithmetic with nested access\n- E2E: 315 → 318 tests, all passing\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-08T10:13:12+09:00",
          "tree_id": "b82f071dee55ef0d95f54d3956344efc01127f91",
          "url": "https://github.com/vaislang/vais/commit/92d02ead12faf17d9dcb112a2d15042d848075e5"
        },
        "date": 1770513623697,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2406,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5334,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 6071,
            "range": "± 264",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11566,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17787,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34272,
            "range": "± 182",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 29920,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65900,
            "range": "± 334",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 270813,
            "range": "± 1242",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 412666,
            "range": "± 2293",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 100010,
            "range": "± 511",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 718832,
            "range": "± 4390",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 151961,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 178413,
            "range": "± 5075",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 186501,
            "range": "± 761",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 227108,
            "range": "± 988",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 484883,
            "range": "± 1851",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 685197,
            "range": "± 1854",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 377025,
            "range": "± 1580",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1092427,
            "range": "± 13327",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 37804,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 192799,
            "range": "± 1571",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 374882,
            "range": "± 5103",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1814770,
            "range": "± 20836",
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
          "id": "5d5bcc3fcfc3c341fef192a29a162888a6c23089",
          "message": "feat: add stdlib modules for env/process/signal (Phase 45)\n\nAdd system-level standard library modules:\n- std/env.vais: getenv/setenv/unsetenv wrappers\n- std/process.vais: system/popen/pclose/exit wrappers\n- std/signal.vais: signal/raise + POSIX signal constants\nRegister all 9 system functions in type checker, text codegen, and Inkwell codegen.\n322 E2E tests passing (+4 new system function tests).\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-08T10:21:36+09:00",
          "tree_id": "358e7786d6cde24562e534cafc28794c8efe1514",
          "url": "https://github.com/vaislang/vais/commit/5d5bcc3fcfc3c341fef192a29a162888a6c23089"
        },
        "date": 1770514105467,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2386,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5375,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5923,
            "range": "± 262",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11146,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17812,
            "range": "± 162",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34007,
            "range": "± 175",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30086,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 65816,
            "range": "± 1513",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 280137,
            "range": "± 2168",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 427247,
            "range": "± 2413",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 104864,
            "range": "± 3239",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 734049,
            "range": "± 2244",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 161678,
            "range": "± 798",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 188224,
            "range": "± 767",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 197299,
            "range": "± 839",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 239416,
            "range": "± 1301",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 513525,
            "range": "± 19116",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 710275,
            "range": "± 2726",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 393780,
            "range": "± 1693",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1126508,
            "range": "± 8005",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 39117,
            "range": "± 469",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 195959,
            "range": "± 1411",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 379312,
            "range": "± 2657",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1838463,
            "range": "± 14895",
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
          "id": "0545195d8a692d72376c3595af9dc42cffc72916",
          "message": "refactor: modularize parser — lib.rs 4,208→792 lines (Phase 46)\n\nSplit parser/lib.rs into focused modules:\n- types.rs (798 lines): type/generic/parameter/const-expr parsing\n- item.rs (1,141 lines): item/macro/trait/impl parsing\nlib.rs now contains only Parser struct, helpers, and public API.\nAll 322 E2E tests and 11 parser tests passing.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-08T10:37:31+09:00",
          "tree_id": "58c26b302e083b570c90f90ebad4a07766ecac30",
          "url": "https://github.com/vaislang/vais/commit/0545195d8a692d72376c3595af9dc42cffc72916"
        },
        "date": 1770515057449,
        "tool": "cargo",
        "benches": [
          {
            "name": "lexer/tokenize/fibonacci",
            "value": 2402,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/sort",
            "value": 5173,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/struct_heavy",
            "value": 5618,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "lexer/tokenize/complex",
            "value": 11523,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/fibonacci",
            "value": 17684,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/sort",
            "value": 34145,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/struct_heavy",
            "value": 30161,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "parser/parse/complex",
            "value": 67786,
            "range": "± 279",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/fibonacci",
            "value": 277755,
            "range": "± 933",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/sort",
            "value": 423725,
            "range": "± 1726",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/struct_heavy",
            "value": 104550,
            "range": "± 620",
            "unit": "ns/iter"
          },
          {
            "name": "type_checker/check/complex",
            "value": 729321,
            "range": "± 42154",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/fibonacci",
            "value": 161353,
            "range": "± 813",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/sort",
            "value": 189551,
            "range": "± 570",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/struct_heavy",
            "value": 198026,
            "range": "± 1245",
            "unit": "ns/iter"
          },
          {
            "name": "codegen/generate/complex",
            "value": 239319,
            "range": "± 1080",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/fibonacci",
            "value": 501971,
            "range": "± 2112",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/sort",
            "value": 706006,
            "range": "± 4702",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/struct_heavy",
            "value": 391584,
            "range": "± 1123",
            "unit": "ns/iter"
          },
          {
            "name": "full_compile/compile/complex",
            "value": 1116902,
            "range": "± 4743",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/100_funcs",
            "value": 38079,
            "range": "± 337",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/500_funcs",
            "value": 194079,
            "range": "± 1578",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/1000_funcs",
            "value": 377016,
            "range": "± 3122",
            "unit": "ns/iter"
          },
          {
            "name": "lexer_scaling/tokenize/5000_funcs",
            "value": 1817098,
            "range": "± 16786",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}