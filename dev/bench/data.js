window.BENCHMARK_DATA = {
  "lastUpdate": 1770295384272,
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
      }
    ]
  }
}