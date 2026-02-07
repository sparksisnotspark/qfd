import axios from "axios";
import dayjs from "dayjs";
import { db, getDb } from "./db";
import { history } from "./db/schema";
import { eq, sql, and, param, inArray } from "drizzle-orm";
import { getVersion } from '@tauri-apps/api/app';

const API_URL = "http://193.134.211.160:8787";

const service = axios.create({ baseURL: API_URL, timeout: 5000 });

export const init_sqlite_db = async (db) => {
    let sql_string = `CREATE TABLE IF NOT EXISTS history(
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				dy_name TEXT,
				dy_account TEXT,
				seq TEXT,
				xiaohao_diamond_count INTEGER DEFAULT 0,
				huode_diamond_count INTEGER DEFAULT 0,
				count_num INTEGER DEFAULT 0,
				canyu_count INTEGER DEFAULT 0,
				hongbao_canyu_count INTEGER DEFAULT 0,
				hongbao_huode_diamond_count INTEGER DEFAULT 0,
				hongbao_win_count INTEGER DEFAULT 0,
				win_count INTEGER DEFAULT 0,
				create_at TEXT NOT NULL,
				task_name TEXT,
				task_msg TEXT,
				UNIQUE(seq, create_at)
  )`;
    await db.execute(sql_string);
};
export const init_browser_seq = async (windows) => {
    for (const w of windows) {
        let exists = await check_record_exists(w.seq);
        console.log(exists, "是否存存", w);
        if (!exists) {
            let now = dayjs().format("YYYY-MM-DD");
            let r = await db
                .insert(history)
                .values({ seq: `${w.seq + ""}`, create_at: now })
                .returning();
            console.log(r, "returning");
        }
    }
};

export const getAppVersion = async () => {
    const version = await getVersion();
    return version;
}

export const clean_task_name = async (date) => {
    let create_at = date ? date : dayjs().format("YYYY-MM-DD");
    try {
        let db = await getDb();
        let sql_ = `UPDATE history SET task_name='', task_msg='' WHERE create_at=?`;
        let x = await db.execute(sql_, [create_at]);
        console.log(x, "clean task name info", create_at);
    } catch (e) {
        console.log(e, "clean exception");  
    }
}

export const delete_data = async(ids) => {
    try {
        let r = await db.delete(history).where(inArray(history.id, ids));
        console.log(r, "delete data result");
    } catch (e) {
        console.log("exception",e)
    }
}

export const db_action_dispatcher = async (event) => {
    const INCREMENT_COLUMNS_WHITELIST = [
        "xiaohao_diamond_count",
        "huode_diamond_count",
        "canyu_count",
        "win_count",
        "hongbao_canyu_count",
        "hongbao_huode_diamond_count",
        "hongbao_win_count",
    ];
    let payload = event.payload;
    let action = payload.action;
    let seq = payload.seq;
    let create_at = payload.create_at ? payload.create_at : dayjs().format("YYYY-MM-DD");

    if ("init_seq" == action) {
        await init_browser_seq(payload.windows);
    }

    if ("insert" == action) {
        let dy_name = payload.dy_name;
        let dy_account = payload.dy_account;
        let seq = payload.seq;
        let exists = await check_record_exists(seq);
        if (exists) {
            return;
        }
        try {
            let x = await db
                .insert(history)
                .values({ dy_name, dy_account, seq, create_at })
                .returning();
            console.log(x);
        } catch (e) {
            console.log(e, "--------------------------------");
        }
    }

    if ("update" == action) {
        let updateClauses = []; // 用于存放 SQL 更新子句 (例如 "name = ?", "count = count + ?")
        let params = []; // 用于存放所有绑定的值

        let exists = await check_record_exists(seq, create_at);
        if (!exists) { //记录不存在
            let dy_name = payload.dy_name;
            let dy_account = payload.dy_account;
            let seq = payload.seq;
            try {
                let x = await db.insert(history).values({ dy_name, dy_account, seq, create_at }).returning();
                console.log(x, "return insert info is x");
            } catch (e) {
                console.log("exception for add new records for this");
            }
        }
        // --- 遍历 payload 来构建动态的 UPDATE 语句 ---
        for (const key in payload) {
            if (Object.prototype.hasOwnProperty.call(payload, key)) {
                const value = payload[key];
                if (INCREMENT_COLUMNS_WHITELIST.includes(key)) {
                    // 如果在白名单中，我们构建一个 'col = col + ?' 形式的子句
                    // 将子句直接拼接到 SQL 字符串部分
                    updateClauses.push(`${key} = ${key} + ?`);
                    // 将要增加的数值（必须是数字）绑定为参数
                    params.push(Number(value)); // 确保值是数字类型
                } else if (key !== "seq" && key !== "create_at" && key !== "action") {
                    // 对于其他普通列（且非 where 条件列），我们构建 'col = ?' 形式的子句
                    updateClauses.push(`${key} = ?`);
                    // 将值绑定为参数
                    params.push(value);
                }
            }
        }

        // --- 执行数据库操作 ---
        if (updateClauses.length === 0) {
            console.log("No valid fields to update.");
            return;
        }

        try {
            let db = await getDb();
            // 拼接完整的 SQL 语句
            // 注意：WHERE 子句中的 seq 和 create_at 已经在外面定义，我们需要把它们的值加到 params 数组的最后
            let sql_ = `UPDATE history SET ${updateClauses.join(", ")} WHERE seq=? AND create_at=?`;
            params.push(seq + ""); // 假设 seq 在当前作用域可用
            params.push(create_at); // 假设 create_at 在当前作用域可用

            let x = await db.execute(sql_, params);
            console.log(x, "update infor", seq, create_at, payload);
        } catch (e) {
            console.log(e, "exceptions?");
        }
    }

    if ("delete" == action) {
        let id = payload.id;
        try {
            await db.delete(history).where(eq(history.id, id));
        } catch (e) {
            console.log(e, "delete exception");
        }
    }
};

export const check_record_exists = async (seq, now) => {
    if ( !now ) {
        now = dayjs().format("YYYY-MM-DD");
    }
    console.log("check exists now is: ", now);
    let res = await db
        .select()
        .from(history)
        .where(and(eq(history.seq, seq + ""), eq(history.create_at, now)));
    if (res.length > 0) return true;
    return false;
};

export const db_insert_history = async (params) => {
    sql = `insert into history(dy_name, dy_account, seq, create_at) values (?,?,?,?)`;
    await db.execute(sql, [
        params.get(
            "dy_name",
            params.get("dy_account"),
            params.get("seq"),
            params.get("create_at"),
        ),
    ]);
};

export const db_update_history = async (params) => { };

export const api_login = async (data) => service.post("/auth/login", data);
export const api_logout = async (data) => service.post("/auth/logout", data);
export const api_register = async (data) => service.post("/auth/register", data);
export const api_unbound = async (data) => service.post("/auth/unbound", data);
export const api_rechage = async (data) => service.post("/auth/recharge", data);
