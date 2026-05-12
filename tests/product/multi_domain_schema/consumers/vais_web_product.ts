import type { User, VaisI64 } from "../gen/user";
import { defineSchema, integer, text, toSQL } from "../webdb/src/schema.js";

function defineUserTable() {
  const table = defineSchema("users", (t) => {
    t.addColumn("id", integer().primaryKey());
    t.addColumn("email", text().unique());
    t.addColumn("name", text());
    t.addIndex("idx_users_email", ["email"], true);
  });
  return toSQL(table);
}

const user: User = {
  id: 7 as VaisI64,
  email: "ada@vais.dev",
  name: "Ada",
};

const email: string = user.email;
const ddl: string = defineUserTable();

if (!ddl.includes("email") || email.length === 0) {
  throw new Error("shared schema product smoke failed");
}
