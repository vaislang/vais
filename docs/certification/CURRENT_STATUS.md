# Current Compiler Certification Status

Date: 2026-05-03

## Verdict

The certified Core compiler freeze bundle is green. As of 2026-05-03, the
certified Vais Core compiler is frozen for downstream re-entry under
`CORE_FREEZE_DECISION.md`. This is not a claim that the whole Vais language,
every advanced feature, or every ecosystem package is finished. It means the
current Core contract and promoted smoke gates pass with the evidence below.

## Verified Gates

| Gate | Evidence |
|---|---|
| Core certification | `CORE_CERTIFICATION pass=16 fail=0 total=16` |
| Core negative diagnostics | Negative fixtures must declare a stable code and emit `error[CODE]`; see `DIAGNOSTIC_CONTRACT.md` |
| Core freeze criteria | `CORE_FREEZE_CRITERIA.md` defines the downstream re-entry gate bundle |
| Core freeze decision | `CORE_FREEZE_DECISION.md` records `Frozen for downstream re-entry` |
| MIR strict lowering and validation | `MIR OK`; `lower_strict_tests` has `17 passed; 0 failed; 0 ignored` |
| MIR interpreter/native comparison for promoted Core fixtures | Included in `core-certify.sh` and `check-integrity.sh` |
| Codegen invariant tests | `CODEGEN OK` |
| Unsafe documentation audit | `UNSAFE AUDIT OK: vais-codegen undocumented_unsafe_blocks=0` |
| String runtime ownership invariant | `phase_string_runtime` passes stored substring field ownership, return move, string-returning if-expression PHI, and per-module helper link coverage |
| Aggregate gate failure propagation | Grouped MIR/CODEGEN subtests short-circuit on first failing command |
| Syntax integrity | `218 passed; 0 failed; 0 ignored` |
| Std package codegen | `82/82` |
| VaisDB package codegen | `261/261` |
| Phase 158 backend smoke | `18/18` |
| std/http_client runtime smoke | `7/7` |
| VaisDB runtime smoke | `28/28` |
| VaisDB runtime lock stability | WAL/LSN/buffer/page/checkpoint mutex release paths covered by current `28/28` smoke |
| vais-server runtime smoke | `13/13` |
| vais-server compiled SSR forwarding | `forward_ssr_render()` loopback upstream POST/status/content-type/body bridge plus upstream non-2xx/transport-failure/timeout/retry mapping and retry-budget observability covered by current `13/13` smoke |
| vais-web runtime smoke | `23/23` |
| vais-web Cloudflare workerd in-process smoke | `Miniflare dispatchFetch` against generated `_worker.js` with real KV-backed `__STATIC_CONTENT` site binding (static index, dynamic route, 404) |
| Rust toolchain pin | `rust-toolchain.toml` pins Rust `1.92.0` with `rustfmt` and `clippy` components |
| Full Rust-hosted compiler test run | Last completed RC baseline: `cargo test --release` exit code `0`; latest current-batch attempt was interrupted after e2e/integrity passed because `registry_e2e_tests` hung at dyld start |
| Diff whitespace check | `git diff --check` clean |

## Canonical Gate Baseline

The table below is **auto-generated** from `GATE_MANIFEST.toml` by
`scripts/render-gate-tables.py` (Master Plan v17 §I-2 Step 12). The
free-form §Verified Gates table above intentionally retains narrative
context per row; the canonical baseline below is the machine-readable
single source for current pass counts. CI fails on drift.

<!-- gate-table:auto-start -->
| Gate | Current status |
|---|---|
| Core certification | `CORE_CERTIFICATION pass=16 fail=0 total=16` |
| MIR strict gate | `MIR OK` |
| Codegen invariant gate | `CODEGEN OK` |
| Unsafe documentation audit | `UNSAFE AUDIT OK: vais-codegen undocumented_unsafe_blocks=0` |
| Ecosystem package codegen | `std=82/82`, `vaisdb=261/261` |
| Backend smoke | `phase158=18/18` |
| std/http_client runtime | `smoke=7/7` |
| std/tls runtime | `smoke=2/2` |
| VaisDB runtime | `smoke=34/34` |
| vais-server runtime | `smoke=13/13` |
| vais-web runtime | `smoke=61/77` |
| vais-web unit | `tests=390/390` |
| vais-web packages | `tests=3272/3272` |
| vais-web full-build | `packages=24/24` |
| Cross-package schema gate | `gate=2/2` |
| Package full-build smoke | `smoke=2/2` |
<!-- gate-table:auto-end -->

## Meaning of the Result

The compiler is currently certified only for the explicit Core surface described
in `VAIS_CORE_V0.md`, the strict MIR subset described in `MIR_CONTRACT.md`, and
the invariant gates wired into `scripts/check-integrity.sh` and
`scripts/core-certify.sh`.

The following claims are valid:

- Core certification fixtures pass.
- Negative Core fixtures fail with stable structured diagnostic headers.
- Core freeze and downstream re-entry criteria are documented and covered by a
  stale-guard test.
- The certified Core compiler is frozen for downstream re-entry.
- Promoted strict MIR fixtures lower without semantic-loss placeholders.
- Unpromoted `Result<T, E>` payload layouts are rejected by strict MIR instead
  of reusing the `Result<i64, i64>` contract.
- The MIR interpreter and native LLVM path agree for promoted strict Core run
  fixtures.
- `scripts/check-integrity.sh` propagates grouped MIR/CODEGEN subtest failures
  into the final aggregate exit status.
- Production codegen unsafe blocks covered by the audit gate must carry
  `SAFETY:` comments, enforced through
  `clippy::undocumented_unsafe_blocks` for `vais-codegen`.
- Current std and VaisDB source files compile through the active compiler gate.
- Current VaisDB runtime smoke fixtures compile, link, and run.
- Promoted VaisDB WAL/LSN/buffer/page/checkpoint runtime paths no longer rely
  on disabled guard destructor lowering for mutex release; the current smoke
  suite covers the HNSW insert, vector bulk, vector WAL, and checkpoint replay
  paths that previously exposed mutex wait hangs.
- Current std/http_client runtime smoke compiles, links, and runs a real
  loopback plain HTTP `POST /ssr/render` request, verifies request
  serialization, parses HTTP status/body through the C runtime, exposes
  response body text with the current `{ptr,len}` string ABI, and follows
  loopback absolute `302 Location: http://127.0.0.1:<port>/final`,
  scheme-relative `302 Location: //127.0.0.1:<port>/final`, and root-relative
  `302 Location: /final` redirects, plus path-relative dot-segment
  `302 Location: ./next/../final` from `/docs/start/index` to
  `/docs/start/final`, query-only `302 Location: ?view=next` from
  `/docs/start/index?old=1` to `/docs/start/index?view=next`, and
  fragment-only `302 Location: #section` without sending `#section` in the
  HTTP request line.
- Current vais-server compiled SSR forwarding smoke compiles, links, and runs
  `forward_ssr_render()` through `std/http_client` against a real loopback
  upstream SSR service, preserving upstream status, content-type, and body in
  the server `Response` success path.
- Current vais-server SSR forwarding error/status smoke preserves upstream
  non-2xx status reason, content-type, and body, and maps transport failure to
  `502 Bad Gateway` with a JSON error response.
- Current vais-server SSR forwarding timeout smoke uses an explicit short
  request timeout and maps stalled upstream responses to `504 Gateway Timeout`
  with a JSON error response.
- Current vais-server SSR forwarding retry smoke retries bounded
  transport-failure/timeout attempts and succeeds when the next loopback
  upstream attempt returns a valid HTTP response.
- Current vais-server SSR forwarding retry-budget observability smoke returns
  bounded retry exhaustion metadata through `X-SSR-Retry-*` headers and body
  markers after the expected loopback attempts fail.
- EmbeddedDatabase open/close creates durable `.vdb` files and preserves meta
  checkpoint LSN across reopen.
- Current vais-server minimal runtime smoke compiles, links, and runs through
  Config/App, Context response, HTTP method/status/header, and error mapping.
- Current vais-server + VaisDB integration smoke compiles, links, and runs
  through server App/Context plus EmbeddedDatabase durable reopen.
- Current vais-server request/router smoke compiles, links, and runs through
  Request header/content-type handling plus static Router exact match, 404, and
  405 behavior.
- Current vais-server path/query smoke compiles, links, and runs through
  Request query parsing, `Params.parse_query`, dynamic route `:param`
  extraction, and dynamic route 404/405 behavior.
- Current vais-server wildcard routing smoke compiles, links, and runs through
  named/unnamed terminal `*param` catch-all extraction, static-over-wildcard
  priority, wildcard 405 handling, empty remainder rejection, and embedded
  `a*b` wildcard rejection.
- Current vais-server request body parser smoke compiles, links, and runs
  through form-urlencoded parsing, compact flat JSON object parsing,
  content-type routed `parse_body`, Request body/content-type integration,
  unsupported content-type errors, and non-object JSON errors.
- Current vais-server middleware pipeline smoke compiles, links, and runs
  through pipeline registration/name lookup, unknown before pass-through,
  symbolic `deny` short-circuit response, and after hook reverse-order
  execution.
- Current vais-server SSR API smoke compiles, links, and runs through flat
  render/hydrate request parsing, render response shape, hydrate response
  shape, missing-route error response, and health response.
- Current vais-web SSR bridge smoke starts the Node SSR service, sends real
  loopback HTTP `POST /ssr/render` requests, delivers request props into the
  renderer, and verifies HTML/head/style/script response shape plus 404
  behavior.
- Current vais-web route/hydration smoke renders SSR client hydration markers,
  resolves a dynamic route through the client router, delivers serialized state
  into hydration, replays a queued pre-hydration event, and removes hydration
  marker attributes in jsdom.
- Current vais-web adapter smoke verifies static adapter generated output,
  server-only API route rejection for static builds, node adapter nested route
  flattening in generated server entry, and node request handler HTML response
  shape.
- Current vais-web Node live adapter smoke writes the generated node server
  entry to a temporary filesystem, starts it in a child Node process on an
  ephemeral local port, and verifies static index serving, dynamic route
  fallback HTML, and `404.html` fallback with HTTP 404 status over real fetch
  requests.
- Current vais-web cloud adapter runtime smoke executes generated Vercel and
  Cloudflare adapter handlers through bounded platform-like request APIs,
  verifies nested dynamic route flattening into Vercel serverless output, and
  verifies Cloudflare Worker static asset, dynamic route, and 404 responses
  through Web Fetch `Request`/`Response`.
- Current vais-web browser bundle runtime smoke executes static adapter
  generated `client.js` in a jsdom browser environment, hydrates SSR
  `[data-vx]` markers through a browser global component registry, restores
  state from `data-vx-state`, removes hydration marker attributes, emits
  hydration events, and verifies click handling without `console.error`.
- Current vais-web real browser runtime smoke serves generated static adapter
  output over local HTTP and verifies hydration markers, browser events,
  marker cleanup, and click handling in Playwright Chromium without
  console/page errors.
- Current vais-web platform output runtime smoke writes generated Vercel Build
  Output API files and Cloudflare Worker output to a temporary filesystem,
  imports the generated serverless function/worker from disk, and verifies
  static, dynamic, and 404 responses through platform-like request/response
  contracts.
- Current vais-web production bundle runtime smoke builds a temporary browser
  fixture with `tsup --format esm --splitting --minify --platform browser`,
  injects the generated ESM entry and dynamic chunk through
  `AdapterConfig.clientBundle`, and verifies modulepreload, chunk loading,
  hydration marker cleanup, state delivery, and click handling in Playwright
  Chromium.
- Current vais-web file-routing production runtime smoke scans a temporary real
  `app/` tree with `/`, `(marketing)/about`, and `/docs/guide`, verifies group
  segment URL elision plus nested route manifest entries, generates static
  root/about/docs-guide pages and `404.html`, injects a minified code-split
  production bundle through `AdapterConfig.clientBundle`, and verifies chunk
  hydration, route metadata, marker/state cleanup, click handling, and 404
  fallback in Playwright Chromium.
- Current vais-web cross-browser hydration runtime smoke serves static adapter
  generated `index.html`/`client.js` over local HTTP and verifies the same SSR
  marker hydration, state restoration, event detail, mount metadata, marker
  cleanup, click handling, and no browser console/page errors in Playwright
  Chromium, Firefox, and WebKit.
- Current vais-web SSR/data-loading production runtime smoke scans a temporary
  `app/products/[sku]/page.vaisx` route with `load()`, verifies prerender skip
  for SSR data routes, serves SSR HTML and `/__data.json` through the kit
  runtime helpers, round-trips route params/query/cookies, and verifies a
  minified code-split production bundle hydrates, loads its dynamic chunk,
  handles clicks, and produces no browser console/page errors in Playwright
  Chromium.
- Current vais-web server action production runtime smoke scans a temporary
  `app/contact/page.vaisx` route with `action()`, verifies prerender skip for
  server-action routes, serves CSRF-protected forms through `injectCsrfField()`
  and `handleServerAction()`, validates same-origin POSTs, form-urlencoded
  parsing, schema validation JSON errors, enhanced JSON submit success, plain
  form `303` redirect, production code-split hydration, dynamic chunk loading,
  and no unexpected browser console/page errors in Playwright Chromium.
- Current vais-web server action auth/rate-limit production runtime smoke scans
  a temporary `app/secure/page.vaisx` route with `action()`, verifies
  prerender skip for server-action routes, serves authenticated POSTs through
  `handleServerAction()` with `authRequired` and `rateLimit: "2/min"`,
  verifies unauthenticated `401` plus `WWW-Authenticate: Bearer`, Bearer-token
  success, third-request `429` plus `Retry-After` and `X-RateLimit-*` headers,
  `vx_session` cookie auth success, production code-split hydration, dynamic
  chunk loading, and no unexpected browser console/page errors in Playwright
  Chromium.
- Current vais-web server action file upload production runtime smoke scans a
  temporary `app/upload/page.vaisx` route with `action()`, verifies prerender
  skip for server-action routes, serves multipart action POSTs through
  `handleServerAction()` with a required `file` schema field, preserves
  uploaded `File` name/type/size/text, verifies plain multipart form `303`
  redirect, production code-split hydration, dynamic chunk loading, and no
  unexpected browser console/page errors in Playwright Chromium.
- Current vais-web Cloudflare workerd in-process runtime smoke builds the
  Cloudflare adapter `_worker.js`, instantiates Miniflare with a real workerd
  process, registers a `sitePath`-backed `__STATIC_CONTENT` KV binding seeded
  with a static `index.html`, and verifies through `dispatchFetch` that the
  generated worker serves the static asset for `/`, renders the dynamic
  `/blog/[slug]` route HTML through the fetch handler, and returns a `404`
  with `text/plain` body for unmatched routes — all against the real
  Cloudflare Workers runtime (not a JavaScript mock).
- Per-module string helper definitions are available even when only an imported
  module uses `substring`/`char_at`.
- A heap-owned `substring` assigned into a struct field transfers ownership into
  the struct mask and survives returning that struct across a module boundary.
- String-returning if-expressions preserve the expected `str` return context
  across block-local cleanup instead of collapsing to an `i64` placeholder.

The following claims are not valid yet:

- "Every Vais language feature is complete."
- "MIR is the semantic oracle for all Core constructs."
- "`vaisdb`, `vais-server`, and `vais-web` are product-complete."
- "vais-server request body parsing is a complete JSON validator."
- "vais-server request body parsing supports nested JSON objects/arrays or a
  broad JSON escape contract."
- "vais-server middleware supports arbitrary middleware instance dispatch."
- "vais-server middleware response body string-concat transforms are certified
  runtime behavior."
- "Automatic RAII/destructor-based unlock lowering is certified for every
  synchronization primitive and every package path."
- "std/http_client HTTPS redirects, method-preservation nuances, keep-alive
  pooling, broad external network behavior, or production network reliability
  are certified."
- "Compiled vais-server SSR forwarding real delay sleep, probabilistic jitter,
  HTTPS/TLS, nested props/full JSON escaping, external network stability, or
  deployed Node SSR bridge operation are certified."
- "vais-server SSR API parsing supports nested JSON props or a broad JSON
  escape contract."
- "vais-web live deployed cloud platform runtime (i.e., a real upload through
  `wrangler deploy` to the Cloudflare edge, or equivalent for Vercel/Netlify/AWS),
  production browser/device hydration beyond the promoted local
  Chromium/Firefox/WebKit smoke, or full dynamic production app behavior are
  certified." The promoted Cloudflare workerd smoke runs the generated
  `_worker.js` in-process via Miniflare and does not exercise live edge
  deployment, account auth, or external network conditions.
- "vais-web live deployed action behavior is certified."
- "Raw `str` pointer arithmetic or `load_byte(str + i)` is a certified string
  parser implementation path."
- "Experimental crates are part of the Core proof."
- "An old roadmap success count is still current evidence."

## Active Source of Truth

Use these files in this order:

1. `/Users/sswoo/study/projects/vais/ROADMAP.md`
2. `/Users/sswoo/study/projects/vais/compiler/ROADMAP.md`
3. `/Users/sswoo/study/projects/vais/docs/design/ai-native-language-principles.md`
4. `docs/certification/CORE_FREEZE_DECISION.md`
5. `docs/certification/VAIS_CORE_V0.md`
6. `docs/certification/MIR_CONTRACT.md`
7. `docs/certification/DIAGNOSTIC_CONTRACT.md`
8. `docs/certification/CORE_FREEZE_CRITERIA.md`
9. `docs/certification/EXCLUDED_FEATURES.md`
10. `docs/certification/SPEC_DRIFT_AUDIT.md`
11. `docs/certification/IGNORED_SURFACE_AUDIT.md`
12. `tests/core/certification_exclusions.tsv`

Historical notes in `docs/history/` are archive material. They are useful for
diagnosis, but they are not active work orders.

## Stop Rules

Stop Core RC work and investigate root cause if any of these happen:

- `scripts/core-certify.sh` fails.
- `scripts/check-integrity.sh` reports a red `CORE`, `MIR`, or `CODEGEN` gate.
- `scripts/check-integrity.sh` logs a failing grouped cargo test but still exits
  successfully.
- A new `#[ignore]` appears in an audited certification file without a manifest
  entry.
- A negative Core fixture succeeds, lacks an expected diagnostic code, or no
  longer emits `error[CODE]`.
- `CORE_FREEZE_CRITERIA.md` no longer matches the current manifest summary,
  canonical gate labels, or source-of-truth list.
- `CORE_FREEZE_DECISION.md` no longer matches the current freeze evidence,
  frozen scope, downstream order, or source-of-truth list.
- Strict MIR lowering accepts a promoted Core fixture by emitting a placeholder
  value.
- Codegen receives unresolved `Unknown`, `Var`, or inference placeholder types
  for a Core fixture.
- A stale test expectation is updated without a current invariant or source of
  truth.

## Next Work

The next work is downstream product certification while preserving the frozen
Core:

1. Keep the VaisDB embedded durability smoke green.
2. Keep the vais-server runtime smoke green.
3. Keep the vais-web runtime smoke green.
4. Promote the next downstream surface only through a new runtime smoke. The
   next likely candidates are a live deployed-platform gate or another bounded
   product surface selected by the root roadmap.
5. Keep raw `str` pointer arithmetic out of promoted parser code until it has a
   separate compiler/runtime invariant.
6. Classify each downstream failure as product/API drift, compiler regression,
   or unsupported non-Core feature before changing compiler code.
7. Keep the freeze bundle green after any compiler-affecting change.

Only compiler regressions should modify the frozen Core compiler path.
