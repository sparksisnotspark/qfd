import { drizzle } from "drizzle-orm/sqlite-proxy";
import Database from "@tauri-apps/plugin-sql";

import * as schema from "./schema";

export async function getDb() {
  return await Database.load("sqlite:sqlite.db");
}

function isSelectQuery(sql) {
  return sql.trim().toLowerCase().startsWith("select");
}

export const db = drizzle(
  async (sql, params, method) => {
    const sqlite = await getDb();
    let rows = [];
    if (isSelectQuery(sql)) {
      rows = await sqlite.select(sql, params).catch((e) => {
        console.error("SQL Error: ", e);
        return [];
      });
    } else {
      rows = await sqlite.execute(sql, params).catch((e) => {
        console.error("SQL Error: ", e);
        return [];
      });
      return { rows: [] };
    }
    rows = rows.map((row) => Object.values(row));
    const results = method == "all" ? rows : rows[0];
    await sqlite.close();
    return { rows: results };
  },
  { schema: schema, logger: true },
);
