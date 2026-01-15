<template>
    <div class="main" style="padding: 10px">
        <el-tabs v-model="activeName" size="small">
            <el-tab-pane label="详细数据" name="records" style="width: 100%">
                <el-form
                    size="small"
                    :model="searchOptions"
                    :inline="true"
                    class="demo-form-inline"
                >
                    <el-form-item>
                        <el-input
                            style="width: 120px"
                            v-model="searchOptions.dy_name"
                            placeholder="抖音名称"
                            clearable
                        />
                    </el-form-item>
                    <el-form-item>
                        <el-input
                            placeholder="抖音号"
                            v-model="searchOptions.dy_account"
                            clearable
                        />
                    </el-form-item>
                    <el-form-item>
                        <el-date-picker
                            v-model="searchOptions.create_at"
                            placeholder="选择日期"
                            type="date"
                        />
                    </el-form-item>
                    <el-form-item>
                        <el-button
                            type="primary"
                            :icon="Search"
                            @click="searchHandler"
                            >查询</el-button
                        >
                        <el-button type="info" @click="onSubmit"
                            >重置</el-button
                        >
                        <el-button type="danger" @click="onSubmit"
                            >清空所有数据</el-button
                        >
                    </el-form-item>
                </el-form>
                <el-table
                    :data="userHistory"
                    size="small"
                    style="width: 100%"
                    max-height="600"
                    border
                    show-summary
                    :summary-method="sumHistory"
                >
                    <el-table-column
                        type="selection"
                        width="50"
                        align="center"
                        label="#"
                    />
                    <el-table-column
                        prop="seq"
                        label="C#"
                        align="center"
                        width="50"
                        show-overflow-tooltip
                    />
                    <el-table-column
                        prop="dy_name"
                        label="抖音名称"
                        width="80"
                        align="center"
                        show-overflow-tooltip
                    />
                    <el-table-column
                        prop="dy_account"
                        label="抖音号"
                        width="100"
                        align="center"
                        show-overflow-tooltip
                    />

                    <el-table-column
                        prop="task_name"
                        label="任务名称"
                        align="center"
                        width="100"
                    />
                    <el-table-column
                        prop="task_msg"
                        min-width="200"
                        label="任务信息"
                        show-overflow-tooltip
                    />
                    <el-table-column
                        prop="canyu_count"
                        label="FD参与数"
                        width="100"
                        align="center"
                        sortable
                    />

                    <el-table-column
                        prop="win_count"
                        label="FD中次数"
                        width="100"
                        align="center"
                        sortable
                    />

                    <el-table-column
                        prop="huode_diamond_count"
                        label="FD获得钻石"
                        align="center"
                        width="120"
                        sortable
                    />

                    <el-table-column
                        prop="hongbao_canyu_count"
                        label="HB参与数"
                        align="center"
                        width="100"
                        sortable
                    />
                    <el-table-column
                        prop="hongbao_win_count"
                        label="HB中次数"
                        align="center"
                        width="100"
                        sortable
                    />

                    <el-table-column
                        prop="hongbao_huode_diamond_count"
                        label="HB得钻石"
                        width="100"
                        align="center"
                        sortable
                    />

                    <el-table-column
                        prop="xiaohao_diamond_count"
                        label="消耗数量"
                        width="100"
                        align="center"
                        sortable
                    />
                    <el-table-column
                        prop="count_num"
                        fixed="right"
                        align="center"
                        label="实时"
                        sortable
                        width="80"
                    />
                    <el-table-column
                        prop="create_at"
                        fixed="right"
                        label="查询时间"
                        width="100"
                    />
                </el-table>
                <div
                    style="
                        text-align: center;
                        margin-top: 10px;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                    "
                >
                    <el-pagination
                        layout="total,sizes,prev,pager,next,jumper"
                        v-model:current-page="currentPage"
                        v-model:page-size="pageSize"
                        :page-sizes="[30, 60, 100]"
                        size="small"
                        :total="historyTotal"
                        @size-change="historyPagerSizeChange"
                        @current-change="historyPagerCurrentChange"
                    ></el-pagination>
                </div>
            </el-tab-pane>
            <el-tab-pane label="福袋任务" name="task">
                <el-container size="small">
                    <el-main>
                        <div class="config-box mb-10">
                            <p>选择分组</p>
                            <div>
                                <el-form :disabled="groupSelectFormDisabled">
                                <el-checkbox-group v-model="groupSelection">
                                    <el-checkbox
                                        v-for="group in groups"
                                        :key="group"
                                        :value="group[0]"
                                        >{{ group[1] }}</el-checkbox
                                    >
                                </el-checkbox-group>
                                </el-form>
                            </div>

                            <p>浏览器功能配置</p>
                            <div>
                                <el-form
                                    size="small"
                                    v-model="browserOptions"
                                    :inline="true"
                                    :label-position="top"
                                    :disabled="browserOptionsDisabled"
                                    label-width="100px"
                                    class="browser-config-form"
                                >
                                    <el-form-item label="任务类型">
                                        <el-radio-group
                                            v-model="browserOptions.tp"
                                        >
                                            <el-radio
                                                value="hb"
                                                label="红包"
                                            ></el-radio>
                                            <el-radio
                                                value="fd"
                                                label="福袋"
                                            ></el-radio>
                                        </el-radio-group>
                                    </el-form-item>

                                    <!--一般是比特浏览器的数量大一点-->
                                    <el-form-item label="线程数量">
                                        <el-input-number
                                            style="width: 100px"
                                            v-model="
                                                browserOptions.thread_number
                                            "
                                            :min="1"
                                            :max="50"
                                            placeholder="线程数量"
                                            type="number"
                                        ></el-input-number>
                                    </el-form-item>

                                    <el-form-item label="二次奖概率">
                                        <el-input-number
                                            v-model="
                                                browserOptions.second_prize_rate
                                            "
                                            style="width: 100px"
                                            placeholder="二次中奖概率"
                                            type="number"
                                        ></el-input-number>
                                    </el-form-item>
                                    <el-form-item label="启用二次检查">
                                        <el-switch
                                            v-model="
                                                browserOptions.enable_second_check
                                            "
                                        ></el-switch>
                                    </el-form-item>
                                    <el-form-item label="循环运行">
                                        <el-switch
                                            v-model="browserOptions.loop_run"
                                        ></el-switch>
                                    </el-form-item>

                                    <el-form-item label="下发次数">
                                        <el-input-number v-model="browserOptions.can_used_times" type="number" placeholder="下发次数"/>
                                    </el-form-item>

                                </el-form>
                            </div>
                        </div>

                        <div class="config-box mb-10">
                            <div
                                style="
                                    display: flex;
                                    margin-bottom: 10px;
                                    justify-content: space-between;
                                    align-items: center;
                                    border-bottom: 1px solid #eee;
                                "
                            >
                                <span style="font-weight: bold"
                                    >是否开启采集功能?
                                </span>
                                <el-switch
                                    size="small"
                                    v-model="spiderOptions.enable_spider"
                                    inline-prompt
                                    active-text="采集中..."
                                    inactive-text="开启采集"
                                    :loading="cjLoading"
                                    :before-change="cjBFHandler"
                                ></el-switch>
                            </div>
                            <div>
                                <el-form
                                    v-model="spiderOptions"
                                    :disabled="spiderEditDisabled"
                                    :inline="true"
                                    :label-position="top"
                                    size="small"
                                    label-width="100px"
                                    class="collector-config-form"
                                >
                                    <el-form-item label="多线程数量">
                                        <el-input-number
                                            style="width: 100px"
                                            v-model="
                                                spiderOptions.thread_number
                                            "
                                            :min="1"
                                            :max="5"
                                            placeholder="采集线程数量"
                                            type="number"
                                        ></el-input-number>
                                    </el-form-item>
                                    <el-form-item label="中奖比例">
                                        <el-input-number
                                            :min="1"
                                            v-model="
                                                spiderOptions.winning_ratio
                                            "
                                            :max="100"
                                            style="width: 100px"
                                            placeholder="中奖比例"
                                            type="number"
                                        ></el-input-number>
                                    </el-form-item>
                                    <el-form-item label="人数限制">
                                        <el-input-number
                                            v-model="spiderOptions.people_limit"
                                            style="width: 100px"
                                            :min="1"
                                            :max="9999"
                                            placeholder="人数限制"
                                            type="number"
                                        ></el-input-number>
                                    </el-form-item>
                                    <el-form-item label="最小时间(秒)">
                                        <el-input-number
                                            style="width: 120px"
                                            v-model="spiderOptions.min_time"
                                            :min="1"
                                            :max="9999"
                                            placeholder="最小时间"
                                            type="number"
                                        ></el-input-number>
                                    </el-form-item>
                                    <el-form-item label="最大时间(秒)">
                                        <el-input-number
                                            style="width: 120px"
                                            :min="1"
                                            :max="9999"
                                            v-model="spiderOptions.max_time"
                                            placeholder="最大时间"
                                            type="number"
                                        ></el-input-number>
                                    </el-form-item>

                                    <el-form-item label="最小人均钻石">
                                        <el-input-number
                                            style="width: 100px"
                                            :min="1"
                                            :max="100"
                                            type="number"
                                            placeholder="最小人均钻石"
                                            v-model="
                                                spiderOptions.min_avg_diamond
                                            "
                                        ></el-input-number>
                                    </el-form-item>
                                    <el-form-item label="只采集大包">
                                        <el-switch v-model="spiderOptions.only_big_packet"></el-switch>
                                    </el-form-item>
                                    <el-form-item label="参与粉丝团">
                                        <el-switch v-model="spiderOptions.join_fans_group"></el-switch>
                                    </el-form-item>
                                    <el-form-item label="直播广场CK">
                                        <el-button
                                            style="width: 200px"
                                            type="primary"
                                            @click="openCKManager"
                                            >管理CK</el-button
                                        >
                                    </el-form-item>
                                </el-form>
                            </div>
                        </div>

                        <div class="config-box">
                            <p>窗口整理配置</p>
                            <div>
                                <el-form
                                    size="small"
                                    v-model="windowOptions"
                                    :inline="true"
                                    :label-position="top"
                                    label-width="100px"
                                    class="window-config-form"
                                >
                                    <el-form-item label="自动排列窗口">
                                        <el-switch v-model="windowOptions.auto_arrange"></el-switch>
                                    </el-form-item>

                                    <el-form-item label="x坐标偏移">
                                        <el-input-number v-model="windowOptions.x_offset"></el-input-number>
                                    </el-form-item>

                                    <el-form-item label="Y坐标偏移">
                                        <el-input-number value="15" v-model="windowOptions.y_offset"></el-input-number> 
                                    </el-form-item>

                                    <el-form-item label="手动操作">
                                        <el-button type="primary" @click="window_bounds">立即排列窗口</el-button>
                                    </el-form-item>
                                </el-form>
                            </div>
                        </div>
                    </el-main>
                    <el-aside width="140px">
                        <div class="control-box" style="margin-top: 20px">
                            <el-button class="cb" type="primary" size="small" :icon="VideoPlay" :loading="startAutomationLoading" @click="startAutomation" :disabled="start_btn_disabled" >启动自动化</el-button>
                            <el-button class="cb" type="danger" size="small" :icon="VideoPause" @click="stopAutomation" >停止自动化</el-button> 
                            <el-button class="cb" type="primary" size="small" :icon="VideoPlay" @click="re_start_automation">启动未成功窗口</el-button>
                            <el-button class="cb" type="info" size="small" @click="fetchGroups" >获取分组</el-button>
                            <el-button class="cb" type="info" size="small" @click="flexable_windows">排列窗口</el-button>
                            <el-button class="cb" size="small" type="success" @click="saveSettings" >保存参数</el-button>
                            <el-button class="cb" size="small" type="primary" @click="logoutHandler" :icon="User" :loading="logoutLoading" >用户退出</el-button>
                        </div>
                        <div>
                            <div class="bt-c">
                                <p>采集状态: {{ spider_status }}</p>
                                <p>总房间数: {{ total }}</p>
                                <p>有用直播间: {{ allowed_count }}</p>
                                <p>有福袋播间: {{ fd_count }}</p>
                                <el-button type="text" @click="showLog">查看日志</el-button>
                            </div>
                        </div>
                    </el-aside>
                </el-container>
            </el-tab-pane>
        </el-tabs>

        <el-dialog v-model="showCookieManagerDialog">
            <el-form
                :model="cookiesForm"
                label-width="100"
                size="small"
                inline
                ref="cookieFormRef"
                style="width: 500px"
            >
                <el-form-item prop="cookie">
                    <el-input
                        v-model="cookiesForm.cookie"
                        placeholder="输入广场cookie"
                        clearable
                    ></el-input>
                </el-form-item>

                <el-form-item>
                    <el-button
                        type="primary"
                        @click="saveCookieHandle"
                        :loading="saveCookieLoading"
                        >保存</el-button
                    >
                </el-form-item>
            </el-form>
            <el-table size="small" border :data="cookiesList">
                <el-table-column prop="ck" show-overflow-tooltip label="内容" />
                <el-table-column label="操作" width="80" align="center">
                    <template #default="scope">
                        <el-button
                            link
                            type="danger"
                            size="small"
                            @click.prevent="deleteCookieHandle(scope.$index)"
                            >删除</el-button
                        >
                    </template>
                </el-table-column>
            </el-table>
        </el-dialog>
        <div class="logger" ref="loggerRef" v-if="showLogDiv">
            <p v-for="i in logs" :key="i">{{ i }}</p>
        </div>
    </div>
</template>

<script setup>
import { reactive, ref, onMounted, nextTick, watch, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElLoading, ElMessage, ElMessageBox } from "element-plus";
import {
    Search,
    Delete,
    VideoPlay,
    VideoPause,
    User,
} from "@element-plus/icons-vue";
import { useAppStore } from "./../store";
import { useRouter } from "vue-router";
import { db, getDb } from "../db";
import { history } from "../db/schema";
import dayjs from "dayjs";
import { listen } from "@tauri-apps/api/event";
import { db_action_dispatcher } from "../api";

const max_legth = 50;
const logs = ref([]);

const loggerRef = ref(null);
const groupSelectFormDisabled = ref(false);

const total = ref(0);
const allowed_count = ref(0);
const showLogDiv = ref(false);
const fd_count = ref(0);

const showLog = async () => {
    showLogDiv.value = !showLogDiv.value;
};

const test_login_required = async () => {
    try {
        let res = await invoke("test_login_required", {params: JSON.stringify({
            group_id: groupSelection.value,
        })});
        console.log("---------", res);
    } catch (e) {
        console.log("error of login required", e);
    }
};

const getMacAddress = async () => {
    try {
        let res = await invoke("mac_address");
        logs.value.push(`MAC地址: ${res.replaceAll(":", "")}`);
        scrollToBottom();
    } catch (e) {
        console.log("error of mac address", e);
    }
};

const test_fetch_live = async () => {
    try {
        let res = await invoke("test_fetch_live", {
            params: JSON.stringify({
                group_id: groupSelection.value,
                thread_number: browserOptions.thread_number,
                up_limit: browserOptions.up_limit,
                second_prize_rate: browserOptions.second_prize_rate,
                enable_second_check: browserOptions.enable_second_check,
                loop_run: browserOptions.loop_run,
                tp: browserOptions.tp,
            }),
        });
        console.log("---------", res);
    } catch (e) {
        console.log("error of live", e);
    }
};

watch(
    logs,
    (newVal) => {
        if (newVal.length > max_legth) {
            logs.value = newVal.slice(newVal.length - max_length, max_length);
        }
    },
    { deep: true },
);

const spider_status = ref("待采集");

const scrollToBottom = () => {
    nextTick(() => {
        if (loggerRef.value) {
            loggerRef.value.scrollTop = loggerRef.value.scrollHeight;
        }
    });
};

const store = useAppStore();

const cookiesForm = reactive({
    cookie: "",
});
const saveCookieLoading = ref(false);
let cookiesList = ref([]);
let spiderEditDisabled = ref(false);

const currentPage = ref(1);
const pageSize = ref(100);
const historyTotal = ref(0);

const historyPagerSizeChange = async (val) => {
    pageSize.value = val;
    await searchHandler();
};

const historyPagerCurrentChange = async (val) => {
    currentPage.value = val;
    await searchHandler();
};

const saveCookieHandle = async () => {
    if (cookiesForm.cookie != "") {
        saveCookieLoading.value = true;
        try {
            cookiesList.value.push({ ck: cookiesForm.cookie + "" });
            await store.setStoreKey("cookies", cookiesList.value);
            ElMessage.success("保存成功");
            setTimeout(() => {
                saveCookieLoading.value = false;
            }, 1000);
        } catch (e) {
            console.log("eee", e);
            saveCookieLoading.value = false;
        }
    }
};

const deleteCookieHandle = async (d) => {
    cookiesList.value.splice(d, 1);
    try {
        await store.setStoreKey("cookies", cookiesList.value);
        ElMessage.success("保存成功");
        setTimeout(() => {
            saveCookieLoading.value = false;
        }, 1000);
    } catch (e) {
        console.log("eee", e);
        saveCookieLoading.value = false;
    }
};

const loadCookies = async () => {
    try {
        let res = await store.loadStoreKey("cookies");
        console.log(res);
        cookiesList.value = res;
    } catch (e) {
        console.log(e, "error for this");
    }
};

const userHistory = ref([]);
const router = useRouter();
const top = ref("top");
const activeName = ref("task");
const groups = ref([]); //存储浏览器分组信息
const groupSelection = ref([]); //存储选中的分组
const startAutomationLoading = ref(false);
const start_btn_disabled = ref(false);
const cjLoading = ref(false);
const showCookieManagerDialog = ref(false); //显示管理 dialog

const searchOptions = reactive({
    dy_name: "",
    dy_account: "",
    create_at: "",
});

const browserOptionsDisabled = ref(false); //浏览器表单是否enabled
const browserOptions = reactive({
    tp: "hb", //福袋或紅包
    up_limit: 10, //抢福袋上限
    thread_number: 5, //采集数据的线程数量
    username: "", //后台用户名
    second_prize_rate: 0, //二次中奖概率
    enable_second_check: false, //启用二次检查
    loop_run: false, //循环运行
    can_used_times: 2,
});

const spiderOptions = reactive({
    enable_spider: false, //启用采集
    thread_number: 2, //采集线程数量
    winning_ratio: 50, //中奖比例
    people_limit: 1000, //人数限制
    min_time: 5, //最小时间(秒)
    max_time: 10, //最大时间(秒)
    join_fans_group: false, //参与粉丝团
    live_square_ck: "", //直播广场CK
    only_big_packet: false, //只采集大包
    min_avg_diamond: 10, //最小人均钻石
    ip_fetch_url: "", //IP获取链接
});

const windowOptions = reactive({
    auto_arrange: false, //自动排列窗口
    x_offset: 50, //x坐标偏移
    y_offset: 15, //Y坐标偏移
});

const task_store_key = "task_store_key";

const onSubmit = () => {
    console.log("submit!");
};
const saveDefaultParams = async () => {
    try {
        await store.setStoreKey(task_store_key, {
            windowOptions: windowOptions,
            spiderOptions: spiderOptions,
            browserOptions: browserOptions,
        });
        ElMessage({ type: "success", message: "保存参数成功" });
    } catch (e) {
        console.log("some of error", e);
        ElMessage({ type: "error", message: "保存参数失败" });
    }
};

const re_start_automation = async () => {
    ElMessageBox.confirm("重新启动未成功的窗口", "确认", {
        confirmButtonText: "确定",
        type: "warning",
    })
        .then(async () => {
            try {
                if (groupSelection.value.length <= 0) {
                    ElMessage({
                        type: "warning",
                        message: "分组不能为空，请选择分组",
                    });
                    return;
                }
                let res = await invoke("re_start_automation", {
                    params: JSON.stringify({
                        group_id: groupSelection.value,
                        thread_number: browserOptions.thread_number,
                        up_limit: browserOptions.up_limit,
                        second_prize_rate: browserOptions.second_prize_rate,
                        enable_second_check: browserOptions.enable_second_check,
                        loop_run: browserOptions.loop_run,
                        tp: browserOptions.tp,
                        join_fans_group:spiderOptions.join_fans_group,
                        can_used_times: browserOptions.can_used_times, 
                    }),
                });
                console.log("重新启动未成功窗口结果:", res);
                ElMessage.success("重新启动了未成功窗口");
            } catch (error) {
                console.error("Error invoking browser_group_windows:", error);
                ElMessage.error(error);
            }
        })
        .catch((e) => {
            ElMessage.error(e);
        });
}

const startAutomation = async () => {
    //启动自动化
    ElMessageBox.confirm("确定要启动自动化么？", "确认", {
        confirmButtonText: "确定",
        type: "warning",
    })
        .then(async () => {
            try {
                browserOptionsDisabled.value = true;
                groupSelectFormDisabled.value = true;
                start_btn_disabled.value = true;

                if (groupSelection.value.length <= 0) {
                    ElMessage.warning("分组不能为空，请选择分组");
                    return;
                }
                let res = await invoke("start_automation", {
                    params: JSON.stringify({
                        group_id: groupSelection.value,
                        thread_number: browserOptions.thread_number,
                        up_limit: browserOptions.up_limit,
                        second_prize_rate: browserOptions.second_prize_rate,
                        enable_second_check: browserOptions.enable_second_check,
                        loop_run: browserOptions.loop_run,
                        tp: browserOptions.tp,
                        join_fans_group:spiderOptions.join_fans_group,
                        can_used_times: browserOptions.can_used_times,
                    }),
                });
                console.log("res of this execute info: ", res);
                ElMessage.success("自动化运行已启动");
            } catch (error) {
                console.error("Error invoking browser_group_windows:", error);
                start_btn_disabled.value = false;
                browserOptionsDisabled.value = false;
                groupSelectFormDisabled.value = false;
                ElMessage.error(error);
            }
        })
        .catch((e) => {
            start_btn_disabled.value = false;
            browserOptionsDisabled.value = false;
            groupSelectFormDisabled.value = false;
            ElMessage.error(e);
        })
        .finally(async () => {
            setTimeout(async () => {
                await saveDefaultParams();
            }, 1000);
        });
};

const cjBFHandler = () => {
    cjLoading.value = true;

    return new Promise((resolve, reject) => {
        cjLoading.value = false;
        ElMessageBox.confirm("请确认您的操作", "确认", {
            type: "warning",
        }).then(
            async () => {
                let hasRun = spiderOptions.enable_spider;
                if (hasRun) {
                    spiderEditDisabled.value = false;
                    cjLoading.value = true;
                    await invoke("sync_stop_spider");
                    ElMessage.success("采集停止成功");
                } else {
                    //采集未启动，则启动采集
                    spiderEditDisabled.value = true;
                    console.log("--", browserOptions.tp);
                    await invoke("sync_start_spider", {
                        options: spiderOptions,
                        gatherType: browserOptions.tp,
                    });
                    ElMessage.success("采集运行成功");
                }
                cjLoading.value = false;
                return resolve(true);
            },
            (r) => {
                console.log("取消了任务");
                return reject(r);
            },
        );
    });
};

const stopAutomation = async () => {
    ElMessageBox.confirm("确定停止任务？", "确认", { type: "warning" }).then(
        async () => {
            const loading = ElLoading.service({
                fullscreen: false,
                lock: true,
                text: "正在停止任务...",
            });
            try {
                console.log(groupSelection.value);
                let res = await invoke("stop_automation");
                console.log("Stopped automation:", res);
                ElMessage({
                    type: "success",
                    message: "自动化已停止",
                });
                loading.close();
            } catch (error) {
                loading.close();
                console.error("Error invoking stop_automation:", error);
                ElMessage({
                    type: "warning",
                    message: error.message,
                });
            }
            spiderEditDisabled.value = false;
            start_btn_disabled.value = false;
        },
    );
};

const searchHandler = async () => {
    let create_at = searchOptions.create_at;
    let params = [];
    let where = [];
    if (create_at) {
        create_at = dayjs(create_at).format("YYYY-MM-DD").toString();
    } else {
        create_at = dayjs().format("YYYY-MM-DD").toString();
    }
    where.push("create_at=?");
    params.push(create_at);

    let dy_name = searchOptions.dy_name;
    let dy_account = searchOptions.dy_account;
    if (dy_account != "") {
        where.push("dy_account=?");
        params.push(dy_account);
    }
    if (dy_name != "") {
        where.push("dy_name=?");
        params.push(dy_name);
    }

    let where_ = where.join(" and ");
    let db = await getDb();
    let page = currentPage.value;
    let ps = pageSize.value;
    let offset = (page - 1) * ps;
    let result_count = await db.select(
        `select count(*) as count from history where ${where_}`,
        params,
    );
    console.log(result_count, "result count is: ");
    historyTotal.value = result_count[0].count;
    let result = await db.select(
        `select * from history where ${where_} limit ${offset}, ${ps}`,
        params,
    );
    userHistory.value = result;
};

const sumHistory = (params) => {
    const { columns, data } = params;
    var sums = [];
    if (data.length > 0) {
        console.log("this is here for log", data);

        let count_num = 0;
        let canyu_count = 0;
        let win_count = 0;
        let hongbao_canyu_count = 0;
        let hongbao_win_count = 0;
        let hongbao_huode_diamond_count = 0;
        let xiaohao_diamond_count = 0;
        let huode_diamond_count = 0;

        data.forEach((v) => {
            count_num += v.count_num;
            win_count += v.win_count;
            canyu_count += v.canyu_count;
            hongbao_canyu_count += v.hongbao_canyu_count;
            hongbao_huode_diamond_count += v.hongbao_huode_diamond_count;
            hongbao_win_count += v.hongbao_win_count;
            xiaohao_diamond_count += v.xiaohao_diamond_count;
            huode_diamond_count += v.huode_diamond_count;
        });
        sums = [
            "合计",
            "",
            "",
            "",
            "",
            "",
            canyu_count,
            win_count,
            huode_diamond_count,
            hongbao_canyu_count,
            hongbao_win_count,
            hongbao_huode_diamond_count,
            xiaohao_diamond_count,
            count_num,
            "",
        ];
    } else {
        sums = [];
    }
    console.log("sums is: ", sums);
    return sums;
};

const fetchGroups = async () => {
    //获取分组
    const loading = ElLoading.service({
        fullscreen: false,
        lock: true,
        text: "正在获取分组...",
    });
    try {
        let res = await invoke("fetch_bitbrowser_group");
        if (res.length <= 0) {
            ElMessage.warning("没有分组，或者分组为空");
        }
        groups.value = res;
        console.log("BitBrowser groups:", res);
        loading.close();
    } catch (error) {
        console.error("Error invoking fetch_bitbrowser_group:", error);
        loading.close();
        ElMessage.error("获取分组失败，请检查您的比特浏览器是否有开启");
    }
    console.log("fetch groups!");
};

const openCKManager = () => {
    nextTick(() => {
        showCookieManagerDialog.value = true;
    });
};

let intervalId = null;

onMounted(async () => {
    await fetchGroups();
    intervalId = setInterval(async () => {
        await searchHandler();
    }, 2000);

    try {
        let res = await store.loadStoreKey(task_store_key);
        console.log(res, "默认参数获得中");
        await loadCookies();
        await searchHandler();
    } catch (e) {
        console.log("exception for loading default params!");
    }
    listen("spider", async (event) => {
        console.log("update for somethings??");
        allowed_count.value = event.payload.allowed_count;
        fd_count.value = event.payload.fd_count;
        total.value = event.payload.total;
    });
    let log = listen("log", async (event) => {
        console.log("log event is: ", event.payload);
        logs.value.push(event.payload);
        scrollToBottom();
    });

});

onUnmounted(() => {
    if (intervalId) {
        clearInterval(intervalId);
    }
});

const flexable_windows = async () => {
    //排列窗口
    try {
        let res = await invoke("flexable_windows", {});
        console.log("res is: ", res);
        ElMessage({ type: "success", message: res });
    } catch (error) {
        console.log("error");
    }
};
//对角线排列
const window_bounds = async () => {
    console.log("windows bounds called!");
    try {
        let res = await invoke("window_bounds", { p: windowOptions });
        console.log("res is: ", res);
        ElMessage({ type: "success", message: res });
    } catch (error) {
        console.log("error", error);
    }
};

const saveSettings = async () => {};

const logoutLoading = ref(false);
const logoutHandler = async () => {
    logoutLoading.value = true;
    try {
        let res = await store.logout();
        setTimeout(() => {
            router.push("/login");
        }, 500);
    } catch (e) {
        console.log("error of e", e);
    }
};

let unlisten = listen("db", async (event) => {
    await db_action_dispatcher(event);
});
</script>

<style lang="less" scoped>
.data-counter {
    background-color: #f5f7fa;
    border: 1px solid #dcdfe6;
    border-radius: 4px;
    text-align: center;
    font-size: 13px;
    font-weight: bold;
    padding: 10px 0;
    margin-bottom: 10px;
}

p {
    margin: 5px 0;
    font-weight: bold;
    border-bottom: 1px solid #eee;
    padding-bottom: 5px;
    margin-bottom: 8px;
}

.search-pannel {
    background-color: #fff;
    border-radius: 5px;
    padding: 20px;
}

.control-box {
    display: flex;
    justify-content: center;
    align-items: center;
    flex-direction: column;
}

:deep(.el-form) {
    display: flex;
    flex-direction: row;
    flex-wrap: wrap;
}
.spider-runing {
    color: green;
}
.config-box {
    border: 1px solid #eee;
    padding: 10px;
    font-size: 12px;
    border-radius: 3px;
    background: #fff;
}

.mt-20 {
    margin-top: 20px;
}

.mb-20 {
    margin-bottom: 20px;
}

.mb-10 {
    margin-bottom: 10px;
}

.mt-10 {
    margin-top: 10px;
}

.cb {
    margin-bottom: 10px;
    width: 100px;
    margin-left: 0;
}

:deep(.el-card__body) {
    padding: 10px;
}

:deep(.el-main) {
    padding: 0;
    margin-right: 10px;
    border-radius: 10px;
}

:deep(.el-aside) {
    background-color: #fff;
    border-radius: 5px;
}

:deep(.el-form-item) {
    display: flex;
    color: red;
    flex-wrap: wrap;
    flex-direction: column;
}

:deep(.el-form-item--label-right .el-form-item__label) {
    justify-content: unset;
    font-size: 12px;
    font-weight: bold;
    color: #666;
}

.browser-config-form .el-form-item .el-form-item__label {
    font-size: 12px;
    font-weight: bold;
    padding: 0;
    margin: 0;
}
.bt-c {
    font-size: 12px;
    p {
        padding: 0 10px 10px 10px;
    }
    button {
        margin-left: 10px;
    }
}
.logger {
    position: absolute;
    bottom: 0;
    left: 0;
    width: 100%;
    z-index: 1000;
    height: 150px;
    border: 1px solid #333;
    overflow-y: auto;
    background-color: #1e1e1e;
    color: #d4d4d4;
    font-family: monospace;
    font-size: 12px;
    padding: 10px;
    box-sizing: border-box;
}
</style>
