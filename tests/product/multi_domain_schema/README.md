# Multi-Domain Shared Schema Product Gate

This gate is the Step 14 product-level companion to the lower-level
`tests/empirical/cross_package_schema` propagation gate.

Source of truth:

- `../../../../examples/schema/user.vais`

Certified path:

- `vaisc emit-ts` generates TypeScript declarations from the shared schema.
- A VaisDB-style consumer builds and runs through real catalog `TableInfo` /
  `ColumnInfo` APIs.
- A vais-server-style consumer builds and runs through real `Context.json`
  response construction.
- A vais-web TypeScript consumer type-checks against the generated `.d.ts` and
  the real `@vaisx/db` schema builder source.
- A field rename in the shared schema fails all three consumers, proving the
  shared product contract does not silently drift.

Run:

```bash
cd compiler
bash tests/product/multi_domain_schema/tests/gate.sh
```
