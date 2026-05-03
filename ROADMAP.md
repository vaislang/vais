# Vais Compiler ROADMAP

> Status: Certified Core frozen for downstream re-entry.
> Canonical workspace roadmap: `/Users/sswoo/study/projects/vais/ROADMAP.md`
> Last verified: 2026-05-03

This file is intentionally short. Historical drive plans and old candidate
lists live in `docs/history/`. Do not resume work from those archived plans
unless the root roadmap promotes that item again.

## Current Verified Baseline

The current active baseline is newer than the archived 2026-04-21 drive notes.
Use these gates as the floor for any compiler change:

| Gate | Current status |
|---|---|
| Core certification | `CORE_CERTIFICATION pass=16 fail=0 total=16` |
| MIR strict gate | `MIR OK` |
| Codegen invariant gate | `CODEGEN OK` |
| Unsafe documentation audit | `UNSAFE AUDIT OK: vais-codegen undocumented_unsafe_blocks=0` |
| Syntax integrity | `218 passed; 0 failed; 0 ignored` |
| Std package codegen | `82/82` |
| VaisDB package codegen | `261/261` |
| Phase 158 backend smoke | `18/18` |
| std/http_client runtime smoke | `1/1` |
| VaisDB runtime smoke | `28/28` |
| VaisDB runtime lock stability | WAL/LSN/buffer/page/checkpoint mutex release paths covered by current `28/28` smoke |
| vais-server runtime smoke | `13/13` |
| vais-server compiled SSR forwarding | `forward_ssr_render()` loopback upstream POST/status/content-type/body bridge plus upstream non-2xx/transport-failure/timeout/retry mapping and retry-budget observability covered by current `13/13` smoke |
| vais-web runtime smoke | `20/20` |
| Rust toolchain pin | `rust-toolchain.toml` pins Rust `1.92.0` with `rustfmt` and `clippy` components |
| Full Rust-hosted compiler test run | Last completed RC baseline: `cargo test --release` exit code `0`; latest current-batch attempt was interrupted after e2e/integrity passed because `registry_e2e_tests` hung at dyld start |
| Formatting sanity | `git diff --check` clean |

The language implementation is still not a product-level "100% complete"
system. The statement above means the current certified Core compiler gate is
green. Broader product surfaces remain outside Core until they are promoted by
fixture-backed gates.

Latest downstream promotion: `vais-web` now has a local server action file
upload production runtime smoke. The certified path scans a temporary
`app/upload/page.vaisx` route with `action()`, verifies that prerender skips the
SSR action route, serves multipart action POSTs through `handleServerAction()`,
preserves uploaded `File` name/type/size/text, and loads a minified code-split
production browser bundle in Playwright Chromium. This certifies local server
action file upload production runtime behavior only; live deployed platforms,
full dynamic production apps, and broader device coverage remain outside the
certified surface.

## Active Policy

- Core compiler work comes before `vaisdb`, `vais-server`, and `vais-web`
  product expansion.
- A new compiler task must promote exactly one invariant or one certification
  audit class at a time.
- Do not make fixes from archived failure counts or partial runtime smoke
  counts; those are historical.
- Do not treat successful downstream package codegen as proof of language
  semantics. Core proof still depends on the Core fixture manifest, strict MIR
  lowering, MIR validation, and interpreter/native agreement for promoted
  fixtures.
- When a stale test expectation conflicts with current certified behavior,
  update the test only after identifying the current source of truth and adding
  or preserving an invariant gate.

## Core Freeze Bundle

These gates are complete and must stay green while downstream product work
resumes:

1. **RC status report lock**
   - Add or update the certification status report in `docs/certification/`.
   - Record the exact active gates, known excluded surfaces, and stop rules.
   - Verification: `cargo test --release`, `bash scripts/check-integrity.sh`,
     `bash scripts/core-certify.sh`, `git diff --check`.

2. **Core spec drift audit**
   - Compare `docs/certification/VAIS_CORE_V0.md`,
     `docs/certification/MIR_CONTRACT.md`, `docs/LANGUAGE_SPEC.md`, and the
     actual fixtures.
   - Promote only documented, fixture-backed behavior.
   - Move unsupported or non-Core claims to deferred/experimental language.
   - Current result: recorded in
     `docs/certification/SPEC_DRIFT_AUDIT.md`.

3. **Ignored/deferred surface audit**
   - Re-run the certification exclusion manifest audit.
   - Classify every ignored test that remains outside the canonical Core gate.
   - Any new ignore must be added to a manifest or removed.
   - Current result: recorded in
     `docs/certification/IGNORED_SURFACE_AUDIT.md`.

4. **Strict MIR promotion pass**
   - Promote the next smallest Core fixture class into strict MIR only when the
     interpreter can execute it without placeholder lowering.
   - Do not use LLVM success alone as semantic proof.
   - Current result: `Result<i64,i64>` construction and match are promoted, and
     other `Result<T, E>` payload shapes are covered by a strict negative gate.

5. **Aggregate gate failure propagation**
   - `check-integrity.sh` must fail when any grouped MIR or CODEGEN subtest
     fails.
   - Current result: grouped MIR/CODEGEN cargo tests use explicit `&&`
     short-circuiting so an early failure cannot be logged and then masked by a
     later passing command.

6. **Full release-candidate verification**
   - Re-run the full gate sequence after every RC slice.
   - No downstream product development should start while these gates are red.
   - Current result: `bash scripts/check-integrity.sh` is green for the current
     slice. The last completed RC full-suite baseline was green, but the latest
     current-batch `cargo test --release` attempt was interrupted after the
     e2e and integrity suites had passed because the `registry_e2e_tests`
     binary hung at dyld start. Treat that as a separate test-runner/tooling
     issue before using full-suite status as fresh evidence.

7. **AI-native language principle alignment**
   - Keep project-level AI-native goals tied to compiler gates, not broad
     feature claims.
   - Current result: the principle document lives at
     `/Users/sswoo/study/projects/vais/docs/design/ai-native-language-principles.md`
     and is linked from the live certification status.

8. **Core diagnostic surface gate**
   - Negative Core fixtures must not pass with vague or incidental error text.
   - Current result: `core_certification_manifest` requires every negative
     fixture to declare a stable diagnostic code and emit `error[CODE]`; see
     `docs/certification/DIAGNOSTIC_CONTRACT.md`.

9. **Core freeze criteria**
   - Downstream work may resume only after the freeze bundle is green in the
     same batch.
   - Current result: `docs/certification/CORE_FREEZE_CRITERIA.md` defines the
     bundle, and `core_freeze_criteria_doc_is_current` fails if the document
     drifts from current gate labels or manifest counts.

10. **Core freeze decision**
    - The active freeze decision must be a stale-guarded certification artifact,
      not a loose status note.
    - Current result: `docs/certification/CORE_FREEZE_DECISION.md` records
      `Frozen for downstream re-entry`, and
      `core_freeze_decision_doc_is_current` fails if the decision drifts from
      current evidence, scope, or the compiler roadmap status.

## Downstream Re-Entry Order

Only after the Core RC tasks stay green:

1. VaisDB embedded durability scenario. Current result:
   `embedded_durability_smoke.vais` verifies actual EmbeddedDatabase
   create/close/reopen durability.
2. VaisDB vector/HNSW correctness scenario. Current result:
   `hnsw_search_recall_larger_smoke.vais` remains part of the runtime gate.
3. `vais-server` minimal runtime gate.
   Current result: `minimal_runtime_smoke.vais` verifies Config/App,
   Context response, HTTP method/status/header, and error classification
   through `vaisc build` + executable runtime.
4. `vais-server` + VaisDB integration gate.
   Current result: `vaisdb_embedded_integration_smoke.vais` verifies
   `App`/`Context` plus EmbeddedDatabase create/flush/close/reopen durability
   in one `vaisc build` runtime.
5. `vais-server` request/router/runtime gate.
   Current result: `request_router_runtime_smoke.vais` verifies Request
   header/content-type handling plus static Router exact match, 404, and 405
   behavior in one `vaisc build` runtime.
6. Compiler string range/substring ownership gate.
   Current result: `phase_string_runtime` verifies per-module string helper
   link stability and a stored `substring` field surviving a struct return
   across module boundaries. Field assignment now transfers tracked string
   ownership into the struct `__ownership_mask`, and block/return move-out of a
   `Named` local skips source shallow-drop before the returned value is loaded.
7. `vais-server` path params/query parser gate.
   Current result: `path_query_runtime_smoke.vais` verifies Request query
   parsing, `Params.parse_query`, dynamic `:param` route matching/extraction,
   and dynamic route 405/404 behavior in one `vaisc build` runtime. The parser
   uses certified string APIs such as `str.char_at` and `substring`; raw `str`
   pointer arithmetic remains outside the promoted path.
8. `vais-server` wildcard routing gate.
   Current result: `wildcard_runtime_smoke.vais` verifies terminal `*param`
   catch-all extraction, unnamed `*` default `wildcard` key extraction, static
   route priority over wildcard routes, wildcard 405 handling, empty remainder
   rejection, and embedded `a*b` wildcard rejection in one `vaisc build`
   runtime.
9. `vais-server` request body parser gate.
   Current result: `body_parser_runtime_smoke.vais` verifies form-urlencoded
   body parsing, compact flat JSON object parsing, content-type routed
   `parse_body`, Request body/content-type integration, unsupported
   content-type errors, and non-object JSON errors in one `vaisc build`
   runtime. Full JSON validation, nested objects/arrays, and broad escape
   semantics remain outside the promoted path.
10. `vais-server` middleware pipeline gate.
    Current result: `middleware_pipeline_runtime_smoke.vais` verifies pipeline
    registration/name lookup, unknown before pass-through, symbolic `deny`
    short-circuit response, and after hook reverse-order execution in one
    `vaisc build` runtime. Arbitrary middleware instance dispatch and response
    body string-concat transforms remain outside the promoted path.
11. `vais-server` SSR API runtime gate.
    Current result: `ssr_api_runtime_smoke.vais` verifies compiled
    `vais-server` SSR request parsing, render response shape, hydrate response
    shape, missing-route error response, and health response in one
    `vaisc build` runtime. The SSR module no longer imports the broader REST
    helper surface for this bounded contract, and `cookie.vais` no longer
    depends on stale external string/concat helpers pulled in through
    `Response`. Later gates cover compiled loopback SSR forwarding. Full JSON
    escaping and nested props remain outside the promoted path.
12. `std/http_client` plain HTTP loopback runtime gate.
    Current result: `phase_http_client_runtime` verifies a `vaisc build`
    executable sends a real loopback `POST /ssr/render` JSON request, links the
    required `http_runtime.c` + `http_client_runtime.c` files, parses status
    and body through the C runtime ABI, and exposes body text with a `{ptr,len}`
    string view. HTTPS/TLS, redirect semantics, keep-alive pooling, external
    network behavior, and compiled `vais-server` SSR forwarding remain outside
    the promoted path.
13. `vais-web` + `vais-server` end-to-end gate.
    Current result: `vais-server-bridge.test.ts` verifies the Node SSR bridge
    over real loopback HTTP: `POST /ssr/render`, props delivery into the
    renderer, HTML/head/style/script response shape, protocol-level 404 for
    unresolved routes, and HTTP 404 for non-render endpoints. This is wired
    into `scripts/check-integrity.sh`.
14. `vais-web` route/hydration runtime gate.
    Current result: `vais-web-route-hydration.test.ts` verifies SSR client
    hydration marker rendering, client router dynamic param resolution,
    loading boundary propagation, serialized state delivery into hydration, and
    queued event replay in a jsdom runtime smoke. This gate remains part of the
    current `WEB RUNTIME` `20/20` surface. Live deployed adapter behavior and
    broader device coverage remain outside the promoted path.
15. `vais-web` adapter runtime gate.
    Current result: `vais-web-adapter-runtime.test.ts` verifies static adapter
    generated output, server-only API route rejection for static builds, node
    adapter nested route flattening in generated server entry, and node request
    handler HTML response shape. Together with the SSR bridge and
    route/hydration smoke, this formed the generated-output/request-handler
    adapter gate.
16. `vais-web` Node live adapter gate.
    Current result: `vais-web-node-live.test.ts` writes the generated node
    server entry to a temporary filesystem, starts it in a child Node process
    on an ephemeral local port, and verifies static index serving, dynamic
    route fallback HTML, and `404.html` fallback with HTTP 404 status over real
    fetch requests. This contributes to the current generated-output runtime
    surface. Later gates cover generated cloud adapter runtime, static browser
    bootstrap runtime, real browser runtime, platform output runtime,
    production bundler output, file-routing production app output, and SSR
    data-loading production app output, and server action production runtime.
    Deployed live platform runtime and full dynamic production app behavior
    remain outside the promoted path.
17. `vais-server` compiled SSR forwarding retry-budget observability gate.
    Current result:
    `e2e_vais_server_12_ssr_forwarding_retry_budget_observability_runtime_smoke`
    verifies a `vaisc build` executable sends exactly three loopback
    `/ssr/render` requests for initial attempt + two retries, then returns
    `502 Bad Gateway` with `X-SSR-Retry-Budget: exhausted`,
    `X-SSR-Retry-Backoff: base+jitter`,
    `X-SSR-Retry-Last-Error: transport`, and body markers for retry budget,
    backoff, and jitter. The gate also keeps `SERVER RUNTIME` at `13/13`.
    Real delay sleep, probabilistic jitter, HTTPS/TLS, external network
    stability, and deployed Node SSR operation remain outside the promoted
    path.
18. `vais-web` real browser runtime gate.
    Current result: `vais-web-real-browser-runtime.test.ts` verifies static
    adapter generated `index.html`/`client.js` over a local HTTP server in
    Playwright Chromium. The smoke confirms SSR marker hydration,
    `vaisx:hydrated` event detail, mounted component metadata, marker
    attribute cleanup, browser click handling, and absence of browser
    console/page errors. This gate is part of the current `WEB RUNTIME`
    `20/20` surface. Live deployed platforms and broader device coverage
    remain outside the promoted path.
19. `vais-web` platform output runtime gate.
    Current result: `vais-web-platform-output-runtime.test.ts` writes generated
    Vercel Build Output API files and Cloudflare Worker output to a temporary
    filesystem, then imports the generated serverless function/worker from disk
    with native dynamic import. The smoke verifies Vercel static output,
    nested dynamic function routing, and 404 handling, plus Cloudflare static
    asset lookup, dynamic response, and missing-route 404 through
    platform-like request/response APIs. This gate is part of the current
    `WEB RUNTIME` `20/20` surface. Live deployed platforms and broader device
    coverage remain outside the promoted path.
20. `vais-web` production bundle/code-splitting runtime gate.
    Current result: `vais-web-production-bundle-runtime.test.ts` builds a
    temporary browser fixture through
    `tsup --format esm --splitting --minify --platform browser`, passes the
    generated `/assets/entry.js` and dynamic `/assets/counter-*.js` chunk to
    `AdapterConfig.clientBundle`, and serves the generated static shell over
    local HTTP in Playwright Chromium. The smoke verifies `modulepreload`,
    absence of default `/client.js`, dynamic chunk resource loading, hydration
    state/marker cleanup, click handling, and absence of console/page errors.
    The gate is part of the current `WEB RUNTIME` `20/20` surface. Live
    deployed platforms remain outside the promoted path.
21. `vais-web` file-routing production app gate.
    Current result: `vais-web-file-routing-production-runtime.test.ts` creates
    a temporary real `app/` directory, scans `/`, `(marketing)/about`, and
    `/docs/guide` with `buildRouteTree()`/`generateManifest()`, verifies group
    segment URL elision plus nested route manifest entries, builds a minified
    code-split browser bundle with `tsup`, injects the generated entry/chunk
    through `AdapterConfig.clientBundle`, and serves generated static output
    over local HTTP in Playwright Chromium. The smoke verifies generated
    `index.html`, `about/index.html`, `docs/guide/index.html`, `404.html`,
    dynamic chunk resource loading, hydration state/marker cleanup, route
    metadata, click handling, and missing-route 404 fallback. The gate is part
    of the current `WEB RUNTIME` `20/20` surface. Live deployed platforms and
    full dynamic production application behavior remain outside the promoted
    path.
22. `vais-web` cross-browser hydration matrix gate.
    Current result: `vais-web-cross-browser-hydration-runtime.test.ts` serves
    static adapter generated `index.html`/`client.js` over local HTTP and runs
    the same hydration fixture in Playwright Chromium, Firefox, and WebKit.
    The smoke verifies SSR marker state restoration, `vaisx:hydrated` event
    detail, mount metadata, `data-vx`/`data-vx-state` cleanup, click handling,
    and absence of browser console/page errors in all three engines. The gate
    keeps `WEB RUNTIME` within the current `20/20` surface. Live deployed
    platforms, full dynamic production app behavior, and broader device
    matrices remain outside the promoted path.

23. `vais-web` SSR/data-loading production app gate.
    Current result: `vais-web-ssr-data-production-runtime.test.ts` creates a
    temporary `app/products/[sku]/page.vaisx` route with `load()`, verifies
    manifest generation and prerender skip for SSR data routes, serves a local
    SSR app through `createLoadContext()`, `executeLoad()`,
    `handleDataRequest()`, and `renderHtmlShell()`, and loads a minified
    code-split production bundle in Playwright Chromium. The smoke verifies
    route params, query source, cookie round-trip through `Set-Cookie`,
    `/__data.json` client data refresh, hydration marker cleanup, mount
    metadata, dynamic chunk resource loading, click handling, and no browser
    console/page errors. The gate keeps `WEB RUNTIME` within the current
    `20/20` surface. Live deployed platforms, full dynamic production app
    behavior, and broader device matrices remain outside the promoted path.
24. `vais-web` server action production runtime gate.
    Current result: `vais-web-server-action-production-runtime.test.ts`
    creates a temporary `app/contact/page.vaisx` route with `action()`,
    verifies manifest generation and prerender skip for server-action routes,
    serves local CSRF-protected forms through `injectCsrfField()` and
    `handleServerAction()`, and loads a minified code-split production bundle
    in Playwright Chromium. The smoke verifies hidden CSRF field injection,
    same-origin validation, form-urlencoded parsing, schema validation JSON
    errors, enhanced JSON submit success, plain form `303` redirect,
    hydration marker cleanup, mount metadata, dynamic chunk resource loading,
    and no unexpected browser console/page errors. The gate keeps
    `WEB RUNTIME` within the current `20/20` surface. Live deployed platforms,
    full dynamic production app behavior, and broader device matrices remain
    outside the promoted path.
25. `vais-web` server action auth/rate-limit production runtime gate.
    Current result:
    `vais-web-server-action-auth-rate-production-runtime.test.ts` creates a
    temporary `app/secure/page.vaisx` route with `action()`, verifies manifest
    generation and prerender skip for server-action routes, serves local action
    POSTs through `handleServerAction()` with `authRequired` and
    `rateLimit: "2/min"`, and loads a minified code-split production bundle in
    Playwright Chromium. The smoke verifies unauthenticated `401` responses
    with `WWW-Authenticate: Bearer`, Bearer-token action success, third
    request `429` rate exhaustion with `Retry-After` and `X-RateLimit-*`
    headers, `vx_session` cookie auth success, hydration marker cleanup, mount
    metadata, dynamic chunk resource loading, and no unexpected browser
    console/page errors. The gate keeps `WEB RUNTIME` at `20/20`. Live deployed
    platforms, full dynamic production app behavior, and broader device
    matrices remain outside the promoted path.
26. `vais-web` server action file upload production runtime gate.
    Current result:
    `vais-web-server-action-file-upload-production-runtime.test.ts` creates a
    temporary `app/upload/page.vaisx` route with `action()`, verifies manifest
    generation and prerender skip for server-action routes, serves multipart
    action POSTs through `handleServerAction()` with a required `file` schema
    field, and loads a minified code-split production bundle in Playwright
    Chromium. The smoke verifies enhanced multipart JSON submit success,
    uploaded `File` name/type/size/text preservation, plain multipart form
    `303` redirect, hydration marker cleanup, mount metadata, dynamic chunk
    resource loading, and no unexpected browser console/page errors. The gate
    keeps `WEB RUNTIME` at `20/20`. Live deployed platforms, full dynamic
    production app behavior, and broader device matrices remain outside the
    promoted path.

This order keeps language/compiler correctness separate from product feature
work and prevents old downstream failures from steering compiler fixes.
