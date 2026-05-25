# Database Integration

Status: T-634 W3-E config/deployment/docs synced.

`vais-server` currently documents a bounded local VaisDB API surface. It is not
a raw SQL service, not a PostgreSQL/SQLite wire-protocol bridge, and not a
production connection-pool or remote database platform.

## Current Local DB API

The current server DB API is the unprefixed `v0` local route surface proven by
`compiler/tests/vais-server/smoke/vaisdb_embedded_integration_smoke.vais`.

Supported route families:

```http
GET /db/books
GET /db/books/count
POST /db/books
GET /db/tables/{table}/rows
GET /db/tables/{table}/rows?id=<INT>
GET /db/tables/{table}/rows?title=<SAFE_TITLE>
GET /db/tables/{table}/rows?body=<SAFE_BODY>
GET /db/tables/tasks/rows?status=<SAFE_STATUS>
GET /db/tables/tickets/rows?state=<SAFE_STATE>
GET /db/tables/contacts/rows
GET /db/tables/contacts/rows?status=<SAFE_STATUS>
GET /db/tables/stations/rows
GET /db/tables/stations/rows?dock=<SAFE_DOCK>
GET /db/tables/stations/rows?id=<INT>
```

`POST /db/books` accepts only this exact compact body:

```json
{"id":3,"title":"children"}
```

It returns either:

```json
{"inserted":1,"id":3,"title":"children"}
```

or:

```json
{"inserted":0,"id":3,"title":"children"}
```

Table-row routes return a fixed envelope:

```json
{"table":"albums","columns":[{"name":"id","type":"INT"},{"name":"title","type":"TEXT"}],"rows":[{"id":1,"title":"blue"},{"id":2,"title":"green"}]}
```

The positive table/schema matrix is limited to the W3-A evidenced shapes:

| Table | Columns | Filters |
|---|---|---|
| `albums` | `id INT`, `title TEXT` | `id`, `title` |
| `notes` | `id INT`, `body TEXT` | `body` |
| `tasks` | `id INT`, `title TEXT`, `status TEXT` | `status` |
| `tickets` | `id INT`, `summary TEXT`, `state TEXT` | `state` |
| `contacts` | `id INT`, `name TEXT`, `email TEXT`, `status TEXT` | `status` |
| `stations` | `id INT` plus fourteen selected `TEXT` columns ending in `dock TEXT` | `id`, `dock` |

## Error Envelopes

Errors use this shape:

```json
{"error":"db_invalid_filter"}
```

Current selected error names include:

- `db_invalid_table`
- `db_invalid_filter`
- `db_invalid_request`
- `db_unsupported_table`
- `db_unavailable`
- `db_query_failed`
- `db_close_failed`
- `db_write_failed`
- `db_conflict`
- `admin_forbidden` for the local guarded seed wrapper

## Local Lifecycle And Concurrency Boundary

Promoted DB route evidence uses a route-local `EmbeddedDatabase` handle for one
request/helper invocation.

- Routes open from the configured local path, run the selected query or write,
  materialize response-owned values, free owned result text where applicable,
  and close before returning.
- There is no promoted process-global DB handle, connection pool, TCP remote DB
  lifecycle, or persistent server-wide DB connection.
- Failed request-time opens return `500` with `{"error":"db_unavailable"}` for
  selected executable local routes.
- Executable timeout/cancellation/backpressure inputs are rejected as invalid
  route shapes: `timeout_ms=1`, `cancel=1`, and `backpressure=drop` on the
  unfiltered table-row route return `db_invalid_filter`; extra `timeout_ms` or
  `cancel` fields on fixed `POST /db/books` return `db_invalid_request`.
- SSR timeout/retry, rate-limit middleware, shutdown coordination, and
  WebSocket lifecycle smokes are non-DB primitives. They are not DB statement
  timeout, DB cancellation, DB backpressure, or production DB concurrency
  evidence.

## Migration, Fault, Health, And Observability Boundary

`vais-server` does not currently promote server-managed migrations,
schema-version routes, upgrade orchestration, rollback/downgrade behavior, or
migration recovery.

Selected executable fault evidence is limited to:

- `db_unavailable` for selected request-time DB open failures;
- `db_query_failed` for selected query failures after successful open, such as
  missing expected catalog state;
- invalid W3-D controls rejected before feature behavior can be inferred:
  `/db/migrations`, `/db/schema-version`, `/db/recovery`, `/db/ready`,
  `/db/metrics`, `/db/traces`, and `/db/audit` return `db_invalid_table`
  through the current table-row helper surface;
- `migration=apply`, `schema_version=1`, `recover=1`, `corrupt=simulate`,
  `ready=1`, `metrics=1`, `trace=1`, and `audit=1` return
  `db_invalid_filter` on selected table-row routes;
- extra fixed `POST /db/books` fields such as `migration`, `schema_version`,
  `recover`, `trace_id`, and `audit` return `db_invalid_request`.

Health and logging evidence is local and primitive: simple health JSON,
request IDs, JSON log formatting, and logger middleware response preservation.
This is not DB readiness, metrics, tracing, audit logging, retention,
redaction, tamper evidence, alerting, or a production observability SLO.

## Configuration, Deployment, And Docs Boundary

`ServerConfig.default()`, explicit in-process `ServerConfig` construction,
`addr()`, and exact `is_prod()` string behavior are local helper evidence only.
The current selected proof covers the default values, address formatting, and
`is_prod()` being true only for exact `prod`.

Unsupported W3-E controls are invalid request shapes, not hidden capabilities:

- `/db/config`, `/db/env`, `/db/deploy`, `/db/release`, `/db/publish`, and
  `/db/docs` return `db_invalid_table` through the current table-row helper
  surface.
- `config=prod`, `env=prod`, `database_url=file.db`, `deploy=1`, `release=1`,
  `publish=1`, `tls=1`, and `docs=1` return `db_invalid_filter` on selected
  table-row routes.
- Extra fixed `POST /db/books` fields such as `config`, `env`, `database_url`,
  `deploy`, `release`, `publish`, and `tls` return `db_invalid_request`.

This is not environment-variable loading, `.env` support, YAML/TOML/JSON server
config loading, CLI config flags, config precedence, server-wide
`DATABASE_URL`, config validation/fail-fast behavior, socket binding proof,
TLS/proxy/DNS behavior, selected deployment manifests, production config
profiles, release publishing, remote deployment, rollback, backup/restore, or
migration runbook evidence.

## Local Auth/Guard Boundary

The only documented DB-route guard is the local fixed seed wrapper around
exact-body `POST /db/books`.

- It uses a process-local `SessionStore` and an explicit same-store
  `session_id`.
- It requires fixed data-bag values `role=admin` and
  `capability=books.seed`.
- Missing/unknown session, missing role, wrong role, wrong capability, expired
  session, and bearer-like `Bearer <session_id>` aliases return `403` with
  `{"error":"admin_forbidden"}`.
- Denied attempts do not write the `children` row.

This is local evidence only. It is not browser login, cookie auth, bearer-token
DB route policy, CSRF protection, production RBAC, or deployed admin security.

## Not Supported

- Raw SQL endpoints such as `/db/sql`, `/db/query`, `/db/execute`, or `sql=`.
- `/api/v1` or `/v1` aliases, required version headers, or content negotiation.
- Arbitrary table/schema/admin APIs, table listing, DDL routes, migrations, or
  catalog browsing.
- Projection, sorting, ordering, limit, offset, `where=`, or arbitrary
  `filter=` controls.
- Generic writes such as `POST`, `PUT`, `PATCH`, or `DELETE` on table routes.
- Alternate `POST /db/books` bodies, extra fields, arrays, or arbitrary values.
- `DbConnection.execute(sql)` as a certified bridge to `EmbeddedDatabase`.
- `QueryBuilder` as a public server DB API surface.
- TCP remote DB mode, connection pooling, concurrent DB serving, DB
  queue/admission/backpressure policy, statement/open/close timeout,
  caller-driven cancellation, production capacity/SLO behavior, startup
  readiness, shutdown drain, production credentials, deployment, production
  auth/session/RBAC, JWT bearer-token DB route policy, cookies, CSRF, secret
  management, rate-limit or CORS attachment to DB routes, audit policy, health
  readiness, observability, backup, recovery, or production acceptance.
- Environment-variable loading, `.env`, YAML/TOML/JSON server config files,
  CLI config flags, config precedence, server-wide `DATABASE_URL`, config
  validation/fail-fast behavior, socket binding proof, TLS/proxy/DNS behavior,
  selected deployment manifests, production config profiles, release
  publishing, remote deployment, rollback, backup/restore, or migration
  runbooks.
- Server-managed migrations, schema-version routes, upgrade/rollback/
  downgrade orchestration, migration recovery, corrupt DB repair, metrics
  export, traces, audit retention, redaction, tamper evidence, backup/restore,
  recovery runbooks, or production observability SLOs.

## Evidence

- `compiler/tests/vais-server/smoke/vaisdb_embedded_integration_smoke.vais`
- `docs/design/server-vaisdb-bounded-api-contract.md`
- `docs/design/vais-completion-roadmap.md`
- `docs/public-alpha/supported-subset.md`
