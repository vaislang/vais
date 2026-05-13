# Multi-Domain Shared Schema Product Gate

This gate is the Step 14 product-level companion to the lower-level
`tests/empirical/cross_package_schema` propagation gate.

Source of truth:

- `schema/user.vais`
- `SCHEMA_SOURCE=/path/to/user.vais` can override the fixture when validating
  against a workspace-level shared schema.

Certified path:

- `vaisc emit-ts` generates TypeScript declarations from the shared schema.
- A VaisDB-style consumer type-checks through the real catalog `TableInfo` API.
- A vais-server-style consumer type-checks through real `Context.new` request
  context construction.
- A vais-web TypeScript consumer type-checks against the generated `.d.ts` and
  the real `@vaisx/db` schema builder source.
- A field rename in the shared schema fails all three consumers, proving the
  shared product contract does not silently drift.

Run:

```bash
cd compiler
VAISC=target/debug/vaisc bash tests/product/multi_domain_schema/tests/gate.sh
```

The gate expects the sibling `lang/packages/vais-web` workspace to have its
`tsc` tool installed.
