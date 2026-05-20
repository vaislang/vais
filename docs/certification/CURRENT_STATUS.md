# Current Compiler Certification Status

Date: 2026-05-20

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
| std/http_client runtime smoke | `15/15` |
| std/tls runtime smoke | `2/2` |
| std/sqlite runtime smoke | `3/3` |
| std/sqlite TEXT column read/free + prepared metadata + transaction/DML helper boundary | `std/sqlite` builds and links against SQLite, `Database.memory()` opens a real in-memory DB, `column_text_owned()` returns a `SqliteText` malloc-owned copy stable across the next `step()`, empty TEXT reads as `""`, `SqliteText.free()` and the manual `SqliteText.drop()` alias are idempotent, legacy `column_text()` can be paired with `free_column_text()`, prepared int/text/null binds can be reused through `reset()` plus `clear_bindings()`, column count/name/type/int reads work inside the statement lifetime, `column_name_owned()` copies remain readable after `finalize()`, a failed prepare exposes non-empty borrowed `Database.error_message()` and `error_message_owned()` remains readable after `close()`, and trusted-name transaction/DML helpers cover create/drop table, table_exists, table_count, rollback/commit visibility, changes, last_insert_id, and exec_many |
| std/postgres runtime smoke | `1/1` |
| std/postgres TEXT/field-name/errmsg owned-copy/free boundary | `std/postgres` builds and links against libpq, `PgResult.get_text()` / `PgResult.field_name()` / `PgConnection.error_message()` remain libpq-owned borrowed text, `PgResult.get_text_owned()` / `PgResult.field_name_owned()` / `PgConnection.error_message_owned()` return explicit `PgText` owned wrappers with idempotent `free()` / manual `drop()` release, and the serverless libpq probe proves `__pg_getvalue_copy()` / `__pg_fname_copy()` / `__pg_error_message_copy()` return malloc-owned copies distinct from borrowed storage, stable after `PQclear()` or `PQfinish()`, and released through `__pg_free_text()` |
| VaisDB runtime smoke | `37/37` |
| VaisDB runtime lock stability | WAL/LSN/buffer/page/checkpoint mutex release paths covered by current `37/37` smoke |
| VaisDB parser-backed fixed-form EmbeddedDatabase SQL API | `EmbeddedDatabase.execute()` / `query()` stabilize public SQL input with malloc-copy, parse one statement, dispatch promoted people-table CREATE/INSERT/SELECT/UPDATE/DELETE AST shapes, and cleanup parser/token text explicitly. Promoted CREATE shapes now include nullable id/name, fixed-column NOT NULL variants, nullable fixed-column literal DEFAULT variants, fixed-column NOT NULL DEFAULT variants, fixed `id INT PRIMARY KEY`, fixed `name TEXT [NOT NULL] UNIQUE`, and fixed `id INT NOT NULL CHECK (id > 0)` using persisted `ColumnInfo.check_kind`. Smoke proves unsupported constraints/default/DML forms return Err, including id CHECK without NOT NULL, name CHECK, id UNIQUE, UNIQUE+CHECK, defaults combined with UNIQUE, table-level/composite PRIMARY KEY/UNIQUE/CHECK forms, FOREIGN KEY/REFERENCES forms, broad/constrained UPDATE/DELETE forms, INSERT ... SELECT, joins, non-people tables, and unsupported projections. The same smoke promotes only the fixed plain-table public UPDATE shape (`SET name = <string literal> WHERE id = <INT literal>`) and proves `42/ada` becomes `42/lovelace` after reopen while stale tuple versions stay hidden from same-handle and later-reopened queries; it also promotes the fixed plain-table public DELETE shape (`WHERE id = <INT literal>`) and proves `7/grace` is hidden after delete, total visible rows drop to 11, `42/lovelace` stays visible, and the deleted row remains hidden after another close/reopen. Fixed id-NOT-NULL UPDATE/DELETE proves `SET name = 'ada-updated' WHERE id = 1` exposes `1/ada-updated` plus nullable `2/NULL`, keeps id and expression updates rejected, rejects missing-WHERE and non-id DELETEs, deletes id 2, proves `2/NULL` is hidden while `1/ada-updated` survives, persists the updated row after reopen, and rejects NULL id again after reopen. Fixed name-NOT-NULL UPDATE/DELETE proves `SET name = 'ada-updated' WHERE id = 10` exposes `10/ada-updated` plus nullable-id `NULL/grace`, keeps id and expression updates rejected, preserves both rows after reopen, rejects NULL name again after reopen, then rejects missing-WHERE and non-id DELETEs, deletes id 10, proves `10/ada-updated` is hidden while `NULL/grace` survives, rejects NULL name again after delete reopen, and proves the deleted row stays hidden. Fixed combined-NOT-NULL UPDATE/DELETE proves omitted id and omitted name writes are rejected before update, `SET name = 'hopper-updated' WHERE id = 20` exposes `20/hopper-updated`, keeps id and expression updates rejected, rejects omitted id and omitted name again after reopen, proves the updated row persists, then inserts `21/grace`, rejects missing-WHERE and non-id DELETEs, deletes id 20, proves `20/hopper-updated` is hidden while `21/grace` survives, rejects omitted id/name after delete reopen, and proves the deleted row stays hidden. The same smoke now also promotes fixed indexed DML for the id PRIMARY KEY table: primary-key UPDATE proves `SET name = 'ada-updated' WHERE id = 1` exposes `1/ada-updated` plus surviving `2/grace` while keeping id updates and expression updates rejected, and primary-key DELETE proves delete of id 1 leaves `2/grace` visible and reinserting id 1 as `reused` succeeds. Fixed name-UNIQUE UPDATE/DELETE proves duplicate-name UPDATE rejects, changed-name UPDATE succeeds, the old unique key can be inserted again, the new unique key still rejects duplicates, DELETE preserves surviving rows, and delete/reinsert preserves unique-key reuse for both non-null and nullable UNIQUE shapes. The promoted CHECK shape accepts positive ids, rejects zero ids before and after reopen, keeps failed CHECK writes from adding heap rows, accepts fixed `SET name = 'ada-updated' WHERE id = 1` while keeping id/expression updates rejected, preserves the updated row after reopen, accepts fixed `DELETE WHERE id = 2` after rejecting missing-WHERE and non-id DELETEs, proves `2/grace` is hidden while `1/ada-updated` and `3/hopper` survive, rejects a zero-id write again after reopen, and proves the deleted row stays hidden. Omitted-column and explicit DEFAULT marker inserts materialize stored defaults before and after reopen, fixed both-column DEFAULT UPDATE/DELETE proves `SET name = 'anon-updated' WHERE id = 100` exposes `100/anon-updated`, hides stale `100/anon`, keeps id/expression/missing-WHERE UPDATE rejected, rejects missing-WHERE and non-id DELETEs, deletes id 100, proves the target is hidden while `101/anon` and existing default rows survive, then materializes another `DEFAULT, DEFAULT` row after delete/reopen while the deleted row stays hidden. Fixed id-only DEFAULT UPDATE/DELETE proves `SET name = 'id-default-updated' WHERE id = 101` exposes `101/id-default-updated`, hides stale `101/NULL`, keeps id/expression/missing-WHERE UPDATE rejected, rejects missing-WHERE and non-id DELETEs, deletes id 101, proves the target is hidden while `102/NULL` and `7/NULL` default rows survive, then materializes another `7/NULL` after delete/reopen while the deleted row stays hidden. Fixed name-only DEFAULT UPDATE/DELETE proves `SET name = <string literal> WHERE id = 100` exposes `100/name-default-updated`, hides stale `100/anon`, keeps id/expression/missing-WHERE UPDATE rejected, rejects missing-WHERE and non-id DELETEs, deletes id 100, proves the target is hidden while `101/anon` and `NULL/anon` default rows survive, then materializes another `NULL/anon` after delete/reopen while the deleted row stays hidden. Fixed id-NOT-NULL DEFAULT UPDATE/DELETE proves `SET name = <string literal> WHERE id = 102` exposes `102/id-nn-default-updated`, hides stale `102/NULL`, keeps id/expression/missing-WHERE UPDATE rejected, rejects explicit NULL id writes after reopen, then materializes another `7/NULL` while the update stays visible, rejects missing-WHERE and non-id DELETEs, deletes id 102, proves `102/id-nn-default-updated` is hidden while `7/hopper` and `7/NULL` default rows survive, rejects explicit NULL id after delete reopen, then materializes another `7/NULL` while the deleted row stays hidden. Fixed name-NOT-NULL DEFAULT UPDATE/DELETE proves `SET name = <string literal> WHERE id = 100` exposes `100/name-nn-default-updated`, hides stale `100/anon`, keeps id/expression/missing-WHERE UPDATE rejected, rejects explicit NULL name writes after reopen, materializes another `NULL/anon` while the update stays visible, rejects missing-WHERE and non-id DELETEs, deletes id 100, proves `100/name-nn-default-updated` is hidden while `102/anon` and `NULL/anon` defaults survive, rejects explicit NULL name after delete reopen, then materializes another `NULL/anon` while the deleted row stays hidden. Fixed combined-NOT-NULL DEFAULT UPDATE/DELETE proves `SET name = <string literal> WHERE id = 101` exposes `101/both-nn-default-updated`, hides stale `101/anon`, keeps id/expression/missing-WHERE UPDATE rejected, rejects explicit NULL id/name writes after reopen, materializes another `7/anon` while the update stays visible, rejects missing-WHERE and non-id DELETEs, deletes id 101, proves `101/both-nn-default-updated` is hidden while `7/hopper` and `7/anon` defaults survive, rejects explicit NULL id/name after delete reopen, then materializes another `7/anon` while the deleted row stays hidden; default-less nullable columns remain NULL, explicit NULL still violates defaulted NOT NULL and fixed UNIQUE NOT NULL columns, duplicate/NULL primary-key ids are rejected before and after reopen through the primary-index B+Tree lookup path, and duplicate non-NULL unique names are rejected while multiple NULL unique names are accepted before and after reopen through a separate `people_name_unique` B+Tree index; distinct non-NULL names still insert after a failed duplicate (`7/hopper`, `7/hamilton`, `7/NULL`, `100/anon`, `101/anon`, `102/anon`, `NULL/anon`, `101/anon`, `102/id-nn-default-updated`, `100/name-nn-default-updated`, `101/both-nn-default-updated`, `7/hopper`, `7/anon`, `1/ada`, `2/grace`, `1/reused`, `3/hopper`, nullable unique `ada`/`NULL`/`grace`). Fixed public `SELECT id, name` projection now returns id/name column metadata and cell order, including qualified refs, output aliases, and filtered arbitrary-id queries while broader projection forms remain rejected. Public SELECT id filtering now carries arbitrary INT literal values instead of hard-coded 42/7/99 filter cases, proving `id = 103` before reopen, missing `id = 12345`, and reversed `103 = id` after reopen while non-id filters remain rejected. Result text/alias cleanup is idempotent, and broad SQL remains out of scope. Arbitrary default expressions, arbitrary CHECK expressions, CHECK on other columns, table-level/composite/name PRIMARY KEY, composite/table-level UNIQUE/CHECK, broad/constrained UPDATE/DELETE beyond the fixed id NOT NULL UPDATE/DELETE, name NOT NULL UPDATE/DELETE, combined NOT NULL UPDATE/DELETE, both-column DEFAULT UPDATE/DELETE, id-only DEFAULT UPDATE/DELETE, name-only DEFAULT UPDATE/DELETE, id-NOT-NULL DEFAULT UPDATE/DELETE, name-NOT-NULL DEFAULT UPDATE/DELETE, combined NOT NULL DEFAULT UPDATE/DELETE, id PRIMARY KEY UPDATE/DELETE, name UNIQUE UPDATE/DELETE, and id CHECK UPDATE/DELETE shapes, INSERT ... SELECT, joins, non-people tables, broad planning/execution, recovery replay, and automatic Token/AST/Result Drop remain follow-up |
| VaisDB SQL DML heap round-trip | `CatalogManager.create_table` + `execute_insert` inserts one explicit INT/TEXT row plus repeated omitted-column and explicit DEFAULT rows into a heap page, verifies same-process heap item-count visibility, and reads back `42/ada`, `100/grace`, `101/grace`, `7/turing`, and `102/grace` through `TableScanExecutor`; the same smoke now calls internal `execute_update()` for `UPDATE people SET name = 'lovelace' WHERE id = 42`, verifies one affected row, commits the update, and scans from a later snapshot to prove five visible rows with `42/lovelace` visible and stale `42/ada` hidden. It then calls internal `execute_delete()` for `DELETE FROM people WHERE id = 100`, verifies one affected row, commits the delete, and scans from a later snapshot to prove four visible rows with deleted `100/grace` hidden, surviving `101/grace` visible, and updated `42/lovelace` still visible. `parse_default_value()` copies TEXT/VARCHAR defaults away from reusable `ColumnInfo.default_value` storage and marks materialized `SqlValue.StringVal` payloads as owned so Row cleanup frees only the materialized copy; broad/constrained public UPDATE/DELETE, public DEFAULT DDL beyond the promoted fixed-column literal shapes, arbitrary default expressions, reopen persistence for internal DML, planner execution, WAL recovery ordering, and broad SQL semantics remain follow-up |
| VaisDB SQL TEXT/VARCHAR/BLOB/VECTOR row decode + encode | TEXT/VARCHAR/BLOB/VECTOR schema-shaped row bytes decoded through `Row.decode`, plus `Row.encode` -> `Row.decode` round-trip for `StringVal`/`BlobVal`/`VectorVal` cells, with decoded content plus `get_owned` checked over a ~200-iteration loop by current `37/37` smoke; decoded StringVal/BlobVal/VectorVal payloads carry owned masks, explicit `Row.free_owned_payloads()` and standalone `SqlValue.free_owned_payload()` are idempotent, `Row.get_owned()` and `Row.clone_owned_payloads()` return independently owned payload copies that remain readable after source row cleanup, Blob/Vector enum construction disarms source local Vec temporaries after boxed enum payload ownership transfer, automatic `Row.drop` calls the same idempotent cleanup helper at scope exit, and automatic `SqlValue.drop` now cleans only standalone `SqlValue.deserialize()` payloads marked with `AUTODROP`; Row-borrowed/shallow copies without that bit are not freed by `SqlValue.drop` |
| vais-server runtime smoke | `25/25` |
| vais-server request body parser JSON grammar | `parse_body("application/json", body)` now accepts only complete JSON object bodies before flat KV extraction; current `25/25` smoke covers nested objects/arrays, strings, strict numbers with fractions/exponents, true/false/null, whitespace, and malformed cases including missing values, trailing commas, leading-zero numbers, invalid escapes, malformed arrays, and trailing non-whitespace tokens |
| vais-server util/json + WebSocket/OAuth/SSR/REST/OpenAPI/log JSON boundaries | `json_get` validates a complete top-level JSON object before returning string fields, `json_get_i64` does the same for integer fields, `json_get_value` validates the same object before returning string values unquoted/unescaped or object/array/number/bool/null as copied raw JSON, and `json_object_end_index` exposes the same complete-object boundary for response helpers; `WsEnvelope.from_json`, OAuth token response parsing, and SSR API request route/props parsing use those extractors for inbound parse, fixed-shape `WsEnvelope.to_json` plus `util/json.json_encode` / `json_encode_array` emit escaped outbound string JSON, `api/rest` covers `json_response`, `json_error`, `paginated_response`, `Link` / `links_to_json_array`, `add_links` including complete existing JSON object body validation before `_links` injection, content negotiation helpers, and `not_acceptable()` for raw JSON data payloads plus escaped error/link dynamic string fields, `api/openapi` covers bounded OpenAPI JSON generation for parameter, request-body, response, operation, spec, grouped path, path-parameter extraction, and escaped dynamic string metadata shapes, and `util/log_json.JsonLogEntry.to_json` covers fixed-shape structured JSON log entries with numeric status/latency plus escaped dynamic string fields; `util/log_json.generate_request_id()` and `core/context.Context.new()` cover CSPRNG-backed `req-` plus 16 lowercase-hex-character IDs through the shared `util/request_id.make_request_id()` mint point and promoted auth runtime boundary; current `25/25` smoke covers spaced valid objects, nested object/array values skipped during extraction, SSR object/array props, escaped string props, Vec-backed object pair encoding, escaped string values, array encoding, empty object/array outputs, exact outbound envelope JSON, REST JSON response/envelope/link helper shapes, REST error/link quote/backslash escaping, REST `add_links` nested/trailing object insertion plus array/malformed/trailing-byte rejection, OpenAPI parameter/request-body/response/operation/spec/generate_spec shapes, grouped methods on one path, `{id}` and `:post_id` extraction, structured log JSON exact output and quote/backslash/newline escaping, direct helper CSPRNG request-id prefix/length/hex alphabet/non-equality, automatic `Context.new()` request_id prefix/length/hex alphabet/non-equality, round-trip payload preservation, missing required fields, trailing non-whitespace, invalid escapes, trailing commas, leading-zero numbers, and literal-boundary rejection |
| vais-server middleware pipeline/recovery/CORS/logger/compress/rate-limit transforms | Symbolic middleware dispatch table covers pass-through before hooks, `deny` short-circuit response, reverse-order after hooks, response body string-concat transforms over a heap-owned body accumulator while preserving response status/content type, and `BeforeResult.next_with(ctx)` context carry-forward; repeated `Context.set_header()` response-header append and `Context.set_state()` dynamic state storage are owned and reach later response/middleware paths; direct `LoggerMiddleware.default()` / `LoggerMiddleware.json()` covers auth-runtime-backed clock reads, `before()` carrying `start_ms=` state, strict start-state parsing, text/JSON `after()` execution, and response preservation; direct `RecoveryMiddleware.default().after()` covers `PANIC:` and status-0 sentinel normalization to JSON 500 plus successful-response passthrough; direct `CompressMiddleware.new(...).before()` / `after()` covers gzip Accept-Encoding detection into carried `compress=gzip` state, response status/body preservation, owned `Content-Encoding: gzip` header append with and without existing headers, small-body no-op, and no-gzip continuation; direct `RateLimitStore` / `RateLimitMiddleware.new(...).before()` covers heap-backed per-IP store persistence across copied middleware values, copied IP-byte matching, promoted auth-runtime millisecond clock reads, first/second allowed-request headers, third-request 429 with `Retry-After` plus `X-RateLimit-Limit`, window reset after pinned time advance, other-IP isolation, and empty-state fallback to `unknown`; direct `CorsMiddleware.default().before()` covers default allow-all `OPTIONS` preflight, clean-context non-`OPTIONS` header propagation, and existing-header append for the default allow-all non-`OPTIONS` path; direct `CorsMiddleware.new(CorsConfig.for_origin(...)).before()` covers matching and non-matching single-origin `OPTIONS` preflight with status 204 plus matching and non-matching single-origin non-`OPTIONS` header propagation with existing-header append; direct `CorsMiddleware.new(CorsConfig { ... }).before()` covers a bounded non-credentialed custom config with configured origin/method/header values, custom `max_age`, and existing-header preservation plus matching/non-matching non-`OPTIONS` propagation into `Context.text()`, and a bounded credentialed custom config with custom origin/method/header/max-age values across matching/non-matching `OPTIONS` preflight and matching/non-matching non-`OPTIONS` propagation into `Context.text()` |
| vais-server auth password policy | strong-password acceptance, short/no-uppercase/no-digit rejection, minimal policy acceptance, and malformed-hash rejection covered by current `25/25` smoke |
| vais-server auth session lifecycle | create/get/get_session/destroy, data-bag insert/update/missing lookup, expired-session rejection, and cleanup retaining live sessions covered by current `25/25` smoke |
| vais-server auth OAuth state/exchange/token parse | Google/GitHub authorization URL construction with explicit/generated state, state validation success/error cases, bounded mock authorization-code exchange, and complete-object JSON validation before token response field extraction covered by current `25/25` smoke |
| vais-server auth JWT token/verify | deterministic access/refresh token encode/generation, role-free and role-bearing verification/decode, loop-based arbitrary role-count join/generation covered by the promoted 3-role smoke, and empty/malformed/bad-signature/expired/issuer-mismatch errors covered by current `25/25` smoke |
| vais-server compiled SSR forwarding | `forward_ssr_render()` loopback upstream POST/status/content-type/body bridge plus upstream non-2xx/transport-failure/timeout/retry mapping, retry-budget observability, nested JSON props preservation, JSON string escaping, and SSR raw-props JSON value grammar validation covered by current `25/25` smoke |
| vais-web runtime smoke | `61/77` in skip-mode CI default |
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
| std/http_client runtime | `smoke=15/15` |
| std/tls runtime | `smoke=2/2` |
| std/sqlite runtime | `smoke=3/3` |
| std/postgres runtime | `smoke=1/1` |
| VaisDB runtime | `smoke=37/37` |
| vais-server runtime | `smoke=25/25` |
| vais-web runtime | `smoke=61/77` |
| vais-web unit | `tests=390/390` |
| vais-web packages | `tests=3272/3272` |
| vais-web full-build | `packages=24/24` |
| Cross-package schema gate | `gate=15/15` |
| Multi-domain product gate | `gate=9/9` |
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
  HTTP request line, and preserves the original `POST` method,
  `Content-Type: application/json`, and JSON body across loopback
  `307 Location: /submit/final`, follows a loopback HTTPS
  `302 Location: /secure/final` redirect through a local TLS server with
  explicit `HttpClient.insecure_tls()` for the self-signed test endpoint,
  decodes `Transfer-Encoding: chunked` response bodies, and retries a stale
  pooled plain HTTP `GET` after the server closes an advertised keep-alive
  socket, reports malformed HTTP status lines, malformed response header lines,
  and malformed `Content-Length` values as parse errors, and reports truncated
  `Content-Length` response bodies as parse errors.
- Current std/sqlite runtime smoke compiles, links, and runs real in-memory
  SQLite database through `Database.memory()`, `exec`, `prepare`, `step`,
  and `column_text_owned()`. The certified TEXT read/free boundary is a
  malloc-owned `SqliteText` copy that remains readable after the next
  `step()`, handles empty TEXT, and is released through idempotent
  `SqliteText.free()` or the manual `SqliteText.drop()` alias. The legacy
  `column_text()` path is certified only when paired with `free_column_text()`.
  Prepared statement int/text/null binding,
  `reset()` plus `clear_bindings()` reuse, `changes()`, column count/name/type/
  int reads, and non-empty `Database.error_message()` after failed prepare are
  also certified inside the owning statement/database lifetime.
  `Statement.column_name_owned()` / `Row.get_name_owned()` copy metadata
  through the same explicit wrapper boundary, and `Database.error_message_owned()`
  remains readable after `close()`. The bounded transaction/DML helper smoke
  certifies trusted-name `create_table` /
  `drop_table`, `table_exists`, `table_count`, rollback/commit visibility,
  explicit-rowid `last_insert_id()`, `changes()`, and `exec_many()` on a single
  in-memory connection.
- Current std/postgres runtime smoke compiles, links, and runs the std wrapper
  against libpq without requiring a live PostgreSQL server. `PgResult.get_text`
  and `PgResult.field_name()` remain PGresult-owned borrowed text, while
  `PgConnection.error_message()` remains PGconn-owned borrowed text.
  `PgResult.get_text_owned()`, `PgResult.field_name_owned()`, and
  `PgConnection.error_message_owned()` return `PgText` wrappers with explicit
  `PgText.free()` and manual `PgText.drop()` release aliases. The C probe
  constructs a synthetic libpq `PGresult` and a serverless `PGconn` to verify
  `__pg_getvalue_copy()` / `__pg_fname_copy()` / `__pg_error_message_copy()`
  are malloc-owned copies distinct from `PQgetvalue()` / `PQfname()` /
  `PQerrorMessage()` storage, stable after `PQclear()` or `PQfinish()`, and
  released through `__pg_free_text()`.
- Current vais-server compiled SSR forwarding smoke compiles, links, and runs
  `forward_ssr_render()` through `std/http_client` against a real loopback
  upstream SSR service, preserving upstream status, content-type, and body in
  the server `Response` success path.
- Current vais-server SSR JSON escaping smoke compiles, links, and runs through
  local hydrate and compiled SSR forwarding paths, escaping quotes,
  backslashes, and control characters in route and string props payload fields.
- Current vais-server SSR raw props embedding treats complete object, array,
  string, number, boolean, and null values as raw JSON when the promoted JSON
  value parser consumes the full value plus trailing whitespace; malformed or
  incomplete values fall back to escaped strings.
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
- EmbeddedDatabase open/config-read/close no-crash behavior is covered.
  Close-state flipping, repeated-close Err cleanup, closed-handle `execute()`
  / `query()` Err cleanup, nested Database field access, and metadata restart
  remain follow-up for this public wrapper. Metadata restart itself remains
  covered by the dedicated checkpoint/recovery smokes.
- Current vais-server minimal runtime smoke compiles, links, and runs through
  Config/App, Context response, HTTP method/status/header, and error mapping.
- Current vais-server + VaisDB integration smoke compiles, links, and runs
  through server App/Context plus the public EmbeddedDatabase boundary.
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
  unsupported content-type errors, non-object JSON errors, full-object JSON
  grammar validation before acceptance, nested object/array value slicing,
  strict fraction/exponent number handling, and malformed JSON rejection for
  missing values, trailing commas, leading-zero numbers, invalid escapes,
  malformed arrays, and trailing non-whitespace tokens.
- Current vais-server util/json, WebSocket envelope, and OAuth token-response
  smoke coverage compiles, links, and runs through complete top-level object
  validation before string/integer field extraction, nested value skipping,
  escaped string extraction, inbound `WsEnvelope.from_json` parsing, fixed-shape
  `WsEnvelope.to_json` outbound JSON serialization and round-trip parse,
  Vec-backed `json_encode` object-pair serialization, `json_encode_array`
  string-array serialization, OAuth token field extraction/defaults, missing
  required field errors, and malformed JSON rejection for trailing
  non-whitespace, invalid escapes, trailing commas, leading-zero numbers, and
  literal-boundary suffixes.
- Current vais-server REST API helper smoke compiles, links, and runs through
  `json_response`, `json_error`, `paginated_response`, `Link` /
  `links_to_json_array`, `add_links`, content negotiation helpers, and
  `not_acceptable()` for simple string values and raw JSON data payloads; the
  `add_links` path validates existing response bodies as complete top-level JSON
  objects before injecting `_links`, supports nested object/array values and
  trailing whitespace, handles `{}`, and rejects array, malformed-object, and
  trailing-byte bodies with a JSON 500 error.
- Current vais-server middleware pipeline smoke compiles, links, and runs
  through pipeline registration/name lookup, unknown before pass-through,
  symbolic `deny` short-circuit response, after hook reverse-order execution,
  and response body string-concat transforms that preserve status and content
  type, plus direct `RecoveryMiddleware.default().after()` panic/sentinel
  normalization to JSON 500 and successful-response passthrough, plus direct
  `CorsMiddleware.default().before()` default allow-all `OPTIONS` preflight
  short-circuit and direct
  `CorsMiddleware.new(CorsConfig.for_origin(...)).before()` matching
  single-origin `OPTIONS` preflight short-circuit with status 204, expected
  CORS headers, empty body, and `text/plain` content type, non-matching
  single-origin `OPTIONS` fallback with existing-header preservation, plus a
  bounded custom `CorsConfig { allowed_origin, allowed_methods,
  allowed_headers, allow_credentials: false, max_age: 123 }` `OPTIONS`
  preflight with existing-header preservation and configured max-age, plus the
  same bounded custom config on matching and non-matching non-`OPTIONS`
  requests carrying configured headers into `Context.text()`, plus a bounded
  credentialed custom `CorsConfig { allow_credentials: true, max_age: 321 }`
  shape covering matching/non-matching `OPTIONS` preflight and matching/non-
  matching non-`OPTIONS` propagation into `Context.text()`, plus repeated
  `Context.set_header()` response-header append, clean-context
  `CorsMiddleware.default().before()` non-`OPTIONS` CORS header propagation,
  default allow-all CORS non-`OPTIONS` existing-header preservation, and matching
  plus non-matching single-origin CORS non-`OPTIONS` existing-header preservation
  through `BeforeResult.next_with(ctx)` into a later `Context.text()` response.
- Current vais-server SSR API smoke compiles, links, and runs through complete
  top-level object validation before render/hydrate route/props extraction,
  string/object/array props preservation, malformed-body rejection, render
  response shape, hydrate response shape, missing-route error response, and
  health response.
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
- "VaisDB public embedded SQL executes arbitrary parser/AST/planner SQL,
  or automatic Token/AST Drop is complete." The promoted boundary
  is parser-backed AST shape recognition for the fixed people-table forms
  listed above, qualified `people.column` references, explicit token/parser
  token cleanup, and explicit parser-created `Statement.free_parser_owned_text()`
  cleanup/disarm for smoke-proven AST paths including `SetOperation` /
  `JoinClause` nested `Box` cleanup.
- "vais-server request body parsing exposes a full JSON DOM."
- "vais-server middleware supports arbitrary middleware instance dispatch beyond
  the promoted symbolic dispatch table, or unpromoted concrete middleware
  paths beyond the direct Recovery after-hook, default CORS preflight,
  matching/non-matching single-origin CORS preflight, the bounded non-
  credentialed custom CORS `OPTIONS` preflight smoke, the bounded credentialed
  custom matching/non-matching `OPTIONS` preflight smoke, default allow-all
  CORS non-`OPTIONS` header propagation, matching/non-matching single-origin
  CORS non-`OPTIONS` header propagation, bounded custom matching/non-
  matching non-`OPTIONS` smokes for the two promoted custom shapes, direct
  CompressMiddleware marker/header behavior, and direct RateLimitMiddleware
  heap-backed store/time/header behavior, or distributed/persistent
  rate-limit stores, arbitrary policy matrices, or open-ended custom CORS config
  matrices beyond the promoted
  `CorsConfig.default()` / `CorsConfig.for_origin(...)` / two bounded custom
  config shapes."
- "Automatic RAII/destructor-based unlock lowering is certified for every
  synchronization primitive and every package path."
- "std/http_client broad external network behavior or production network
  reliability is certified."
- "std/sqlite broad SQL semantics, automatic Drop for SQLite TEXT wrappers,
  borrowed `column_name` / `errmsg` pointer use after finalize/close, or
  unpaired legacy `column_text()` leak freedom are certified." The promoted
  boundary is TEXT column malloc-copy reads paired with `SqliteText.free()`,
  the manual `SqliteText.drop()` alias, or `free_column_text()`, prepared
  bind/reset/clear-bindings reuse, borrowed metadata/error reads inside the
  owning statement/database lifetime, and owned metadata/error copies through
  `column_name_owned()` / `get_name_owned()` / `error_message_owned()` when the
  string must outlive `finalize()` or `close()`.
- "Postgres borrowed accessors use the same ownership/free boundary as
  std/sqlite owned accessors, or borrowed Postgres text/metadata/diagnostics
  remain valid after `PgResult.clear()` / `PgConnection.disconnect()`." The
  promoted borrowed accessors remain libpq-owned and must not be freed
  separately; use `PgResult.get_text_owned()` / `PgResult.field_name_owned()` /
  `PgConnection.error_message_owned()` plus `PgText.free()` or manual
  `PgText.drop()` for the explicit owned-copy boundary. Automatic Drop for
  copyable DB text wrappers is still a non-claim.
- "Compiled vais-server SSR forwarding real delay sleep, probabilistic jitter,
  HTTPS/TLS, complete JSON validation across every remaining ad-hoc parser path,
  external network stability, or deployed Node SSR bridge operation are
  certified."
- "Remaining vais-server ad-hoc JSON helpers outside the promoted request-body,
  SSR API route/props, SSR raw-props value parser, `json_get`/`json_get_i64`/
  `json_get_value`, util/json string encoders, WebSocket inbound/fixed outbound,
  OAuth token-response paths, `api/rest` response/envelope/link helper paths,
  bounded `api/openapi` document generation, and `util/log_json` entry
  serialization validate complete JSON grammar."
- "vais-server `json_get`/`json_get_i64`/`json_get_value` exposes a full JSON
  DOM or typed object model beyond promoted top-level field extraction."
- "vais-server `util/json.json_encode` and `json_encode_array` serialize a full
  JSON DOM, non-string values, or arbitrary typed/nested values beyond promoted
  escaped string object pairs and string arrays."
- "vais-server `api/rest` helpers serialize arbitrary typed JSON or validate
  full JSON DOM data for every helper input beyond the promoted `add_links`
  existing-object-body boundary."
- "vais-server monotonic request counter sequencing is certified."
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
