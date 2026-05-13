// Consumer: vais-web TS that imports the generated .d.ts.
// `tsc --noEmit` resolves the brand symbols + User interface from
// ../gen/user.d.ts via explicit type-only import.

import type { User, VaisI64 } from "../gen/user";

function showEmail(u: User): string {
  return u.email;
}

function makeUser(): User {
  return {
    id: 1 as VaisI64,
    email: "x@y",
    name: "n",
  };
}

// Touch both functions so tsc doesn't optimize them away.
const _e = showEmail(makeUser());
