# Cross-Package Shared Schema Gate

This fixture proves that a typed change to one shared Vais schema is seen by
Vais consumers and TypeScript consumers instead of drifting silently.

Certified path:

- `vaisc emit-ts` emits `gen/user.d.ts` from `schema/user.vais`.
- `consumers/vaisdb_table.vais` and `consumers/vais_server_api.vais` are
  checked and run natively after being combined with the shared schema.
- `consumers/vais_web_consumer.ts` type-checks against the generated `.d.ts`.
- A field rename and a field type change both force the expected downstream
  failures.

Run:

```bash
cd compiler
VAISC=target/debug/vaisc bash tests/empirical/cross_package_schema/tests/gate.sh positive
VAISC=target/debug/vaisc bash tests/empirical/cross_package_schema/tests/gate.sh negative
```

The TypeScript leg expects the sibling `lang/packages/vais-web` workspace to
have its `tsc` tool installed.
