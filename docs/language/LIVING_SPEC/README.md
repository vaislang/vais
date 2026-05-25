# Living Spec Boundary

`LIVING_SPEC` files are executable syntax references for `vaisc check`.
The integrity gate requires every `.vais` file in this tree to type-check, but
that does not automatically make every example a product-complete runtime,
codegen, DB, server, or web claim.

Use these rules when reading or adding examples:

- A comment that says TC-only, reproducer, gap, nonclaim, or boundary is part
  of the contract for that file.
- A2 behavior is limited to `A2_SUBSETS.md`; broad `?`, dyn, closure, and
  function-value behavior outside those predicates is not implied by an example
  that type-checks.
- Rejected and Controlled surfaces remain governed by
  `EXCLUDED_FEATURES.md`. In particular, affine/linear annotations are
  type-carrier syntax only, automatic Drop is not promoted, and broad implicit
  coercions stay out of public examples.
- DB-facing code must still follow `docs/design/db-required-language-profile.md`
  for explicit cleanup, ownership, and diagnostic boundaries.
