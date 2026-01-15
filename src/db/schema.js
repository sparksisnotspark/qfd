import { sqliteTable, text, integer } from "drizzle-orm/sqlite-core";

export const history = sqliteTable("history", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  dy_name: text("dy_name"),
  dy_account: text("dy_account"),
  seq: text("seq").unique(),
  xiaohao_diamond_count: integer("xiaohao_diamond_count").default(0),
  huode_diamond_count: integer("huode_diamond_count").default(0),
  count_num: integer("count_num").default(0),
  canyu_count: integer("canyu_count").default(0),
  hongbao_canyu_count: integer("hongbao_canyu_count").default(0),
  hongbao_huode_diamond_count: integer("hongbao_huode_diamond_count").default(
    0,
  ),
  hongbao_win_count: integer("hongbao_win_count").default(0),
  win_count: integer("win_count").default(0),
  create_at: text("create_at").notNull(),
});
