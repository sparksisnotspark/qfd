<template>
    <div class="pannels">
        <el-image src="/icon.png" style="width:70px; height: 70px; margin-top: 10px;" />
        <div style="color: red; background-color: rgba(0, 0, 0, 0.2); font-size: 14px;  padding: 10px; margin: 10px; border-radius: 10px;" >
            <h3 style="border-bottom: 1px solid rgba(0, 0, 0, 0.3); margin: 0;">重要提示</h3>
            <p>
                1.本软件仅供学习交流使用，请勿用于非法用途，一切后果由使用者承担
            </p>
            <p>2.请尊守法律法规，创文明网络</p>
            <p>3.如果您点【登录】,则视为接受本声明所述的全部内容</p>
        </div>
        <el-tabs v-model="activeName">
            <el-tab-pane label="登录" name="login">
                <div>
                    <el-form
                        label-width="100px"
                        :model="loginInfo"
                        class="login-form"
                        :rules="loginRules"
                        ref="loginFormRef"
                    >
                        <el-form-item label="KEYCODE" prop="keycode">
                            <el-input
                                v-model="loginInfo.keycode"
                                placeholder="后台数据标识码"
                                clearable
                            ></el-input>
                        </el-form-item>
                        <el-form-item label="用户名" prop="username">
                            <el-input
                                v-model="loginInfo.username"
                                placeholder="请输入用户名"
                                clearable
                            ></el-input>
                        </el-form-item>
                        <el-form-item label="密码" prop="password">
                            <el-input
                                v-model="loginInfo.password"
                                placeholder="请输入密码"
                                type="password"
                                clearable
                            ></el-input>
                        </el-form-item>
                        <el-form-item>
                            <el-button
                                type="primary"
                                :loading="loginLoading"
                                @click="loginHandler"
                                >登录</el-button
                            >
                        </el-form-item>
                    </el-form>
                </div>
            </el-tab-pane>
            <el-tab-pane label="注册" name="register">
                <div>
                    <el-form
                        label-width="100px"
                        v-model="registerInfo"
                        class="register-form"
                        ref="registerFormRef"
                    >
                        <el-form-item label="用户名" prop="username">
                            <el-input
                                placeholder="请输入用户名"
                                v-model="registerInfo.username"
                                clearable
                            ></el-input>
                        </el-form-item>
                        <el-form-item label="密码" prop="password">
                            <el-input
                                v-model="registerInfo.password"
                                placeholder="请输入密码"
                                type="password"
                                clearable
                            ></el-input>
                        </el-form-item>
                        <el-form-item label="确认密码" prop="confirm_password">
                            <el-input
                                v-model="registerInfo.confirm_password"
                                placeholder="确认密码"
                                type="password"
                                clearable
                            ></el-input>
                        </el-form-item>
                        <el-form-item>
                            <el-button
                                type="primary"
                                @click="registerHandler"
                                :loading="registerLoading"
                                >注册</el-button
                            >
                        </el-form-item>
                    </el-form>
                </div>
            </el-tab-pane>
            <el-tab-pane label="充值" name="recharge">
                <div>
                    <el-form
                        v-model="rechage"
                        label-width="100px"
                        class="recharge-form"
                        ref="rechargeFormRef"
                    >
                        <el-form-item label="用户名" prop="username">
                            <el-input
                                v-model="rechage.username"
                                placeholder="输入用户名"
                                clearable
                            ></el-input>
                        </el-form-item>

                        <el-form-item label="充值码" prop="keycode">
                            <el-input
                                v-model="rechage.keycode"
                                placeholder="请输入充值码"
                                clearable
                            ></el-input>
                        </el-form-item>
                        <el-form-item>
                            <el-button
                                type="primary"
                                @click="rechargeHandler"
                                :loading="loginLoading"
                                >充值</el-button
                            >
                        </el-form-item>
                    </el-form>
                </div>
            </el-tab-pane>

            <el-tab-pane label="解绑" name="unbound">
                <div>
                    <el-form
                        v-model="unboundInfo"
                        label-width="100px"
                        class="unbound-form"
                        ref="unboundFormRef"
                    >
                        <el-form-item label="用户名" prop="username">
                            <el-input
                                v-model="unboundInfo.username"
                                placeholder="输入用户名"
                                @blur="handleBlur"
                                clearable
                            ></el-input>
                        </el-form-item>

                        <el-form-item>
                            <el-button
                                type="primary"
                                :loading="loginLoading"
                                @click="unboundHandler"
                                >解绑</el-button
                            >
                        </el-form-item>
                    </el-form>
                </div>
            </el-tab-pane>
        </el-tabs>
    </div>
</template>

<script setup>
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { resourceDir, join} from "@tauri-apps/api/path";
import { onMounted, reactive, ref } from "vue";

import { ElForm, ElMessage, ElMessageBox } from "element-plus";
import { api_login, api_rechage, api_register, api_unbound, init_sqlite_db } from "./../api.js";
import { useAppStore } from "./../store.js";
import { useRouter } from "vue-router";
import { iconPropType } from "element-plus/es/utils/index.mjs";
const activeName = ref("login");
const loginLoading = ref(false);
const loginFormRef = ref(null);
const registerFormRef = ref(null);
const unboundFormRef = ref(null);
const rechargeFormRef = ref(null);
const appName = ref("");
const appImageIcon = ref("");

const loginInfo = reactive({
    username: "",
    password: "",
    keycode: "",
});

const router = useRouter();
const registerInfo = reactive({
    username: "",
    password: "",
    confirm_password: "",
    keycode: "",
});

const fetch_mac_address = async () => {
    try {
        let res = await invoke("mac_address");
        console.log("获取到的mac地址", res);
        return res.replaceAll(":", "");
    } catch (e) {
        console.log("error of mac address", e);
        return "";
    }
};

const handleBlur = () => {};

const registerRules = {
    username: [{ required: true, trigger: "blur", message: "用户名不能为空" }],
    keycode: [
        { required: true, trigger: "blur", message: "后台数据key不能为空" },
    ],
    password: [{ required: true, trigger: "blur", message: "密码不能为空" }],
};
const unboundInfo = reactive({
    username: "",
});
const unboundRules = {
    username: [{ required: true, trigger: "blur", message: "用户名不能为空" }],
};
const rechage = reactive({
    username: "",
    keycode: "",
});
const rechargeRules = {
    username: [{ required: true, trigger: "blur", message: "用户名不能为空" }],
    keycode: [
        { required: true, trigger: "blur", message: "后台数据key不能为空" },
    ],
};

const loginRules = {
    username: [{ required: true, trigger: "blur", message: "用户名不能为空" }],
    keycode: [
        { required: true, trigger: "blur", message: "后台数据key不能为空" },
    ],
    password: [{ required: true, trigger: "blur", message: "密码不能为空" }],
};

const store = useAppStore();
const registerLoading = ref(false);
const registerHandler = async () => {
    try {
        let mac_address = await fetch_mac_address();
        if (registerInfo.confirm_password != registerInfo.password) {
            ElMessage.warning("密码与确认密码不一样，请重新输入");
            return;
        }
        let res = await api_register({ ...registerInfo, mac_address });
        let data = res.data;
        if (data.code != 0) {
            ElMessage.warning(data.msg);
            loadingTimeout(registerLoading, false);
        } else {
            router.push("/login");
            ElMessage.success(data.msg);
        }
    } catch (e) {
        console.log(e);
        loadingTimeout(registerLoading, false);
    }
};

onMounted(async () => {
    try {
        const currentWindow = await getCurrentWindow();
        appName.value = await currentWindow.title();
        let res = await store.loadStoreKey("login_info_key");
        const resource_path = await resourceDir();
        const icon_path = await join(resource_path, "icons/icon.png");
        appImageIcon.value = icon_path; //convertFileSrc(icon_path);
        console.log("icon path is: ", icon_path);
        console.log("icon value ", appImageIcon.value);

        //const appIconB64 = await getAppImage();
        //appImageIcon.value = `data:image/png;base64,${appIconB64}`;
        
        loginInfo.keycode = res.keycode;
        loginInfo.username = res.username;
        loginInfo.password = res.password;
    } catch (e) {
        console.log(e, "未找到key");
    }
});

const unboundHandler = async () => {
    loginLoading.value = true;
    try {
        let machine_id = await fetch_mac_address();
        let params = {...unboundInfo, machine_id};
        let res = await api_unbound({ ...unboundInfo, machine_id});
        loginLoading.value = false;
        if(res.data.code == 0) {
            ElMessage.success(res.data.msg);
        } else {
            ElMessage.warning(res.data.msg);
        }
    } catch (e) {
        ElMessage.error(e);
        setTimeout(() => {
            loginLoading.value = false;
        }, 1000);
    }
};

const loadingTimeout = async (l, value) => {
    setTimeout(() => {
        l.value = value;
    }, 1000);
}

const rechargeHandler = async () => {
    loginLoading.value = true;
    try {
        let mac_address = await fetch_mac_address();
        let res = await api_rechage({ ...rechage, mac_address });
        if (res.data.code != 0) {
            ElMessage.warning(res.data.msg);
        } else {
            ElMessage.success(res.data.msg);
            router.push("/login");
        }
        loadingTimeout(loginLoading, false);
    } catch (e) {
        console.log(e);
        loadingTimeout(loginLoading, false);
    }
};

const loginHandler = async () => {
    loginLoading.value = true;
    loginFormRef.value.validate(async (valid) => {
        if (valid) {
            try {
                let mac_address = await fetch_mac_address();
                let r = await store.login({ ...loginInfo, mac_address });
                if (!r[0]) {
                    ElMessage.warning(r[1].msg);
                    loadingTimeout(loginLoading, false);
                } else {
                    let expired_at = r[1].data.expired_at;
                    ElMessageBox.alert(
                        `登录成功,到期时间为:${expired_at}`,
                        "提示",
                        {
                            confirmButtonText: "确定",
                            type: "success",
                        }
                    );
                    router.push("/");
                }
            } catch (e) {
                ElMessage.error(e);
                loadingTimeout(loginLoading, false);
            }
        } else {
            ElMessage.warning("验证失败");
            loadingTimeout(loginLoading, false);
        }
    });
};
</script>

<style lang="css" scoped>
.pannels {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
    height: 100%;
    background-color: #eee;
}
:deep(.el-tabs) {
    width: 400px;
    background-color: #fff;
    padding: 20px;
    border-radius: 20px;
}
</style>
