window.BENCHMARK_DATA = {
  "lastUpdate": 1770379593698,
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
      }
    ]
  }
}