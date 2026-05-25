# Database Integration

Status: T-593 W3-A server API docs synced.

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
- TCP remote DB mode, connection pooling, production credentials, deployment,
  production auth/session/RBAC, rate limits, CORS/CSRF, audit policy, health
  readiness, observability, backup, recovery, or production acceptance.

## Evidence

- `compiler/tests/vais-server/smoke/vaisdb_embedded_integration_smoke.vais`
- `docs/design/server-vaisdb-bounded-api-contract.md`
- `docs/design/vais-completion-roadmap.md`
- `docs/public-alpha/supported-subset.md`
