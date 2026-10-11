<script setup>
import { reactive } from "vue";
import { ElMessage } from "element-plus";
import http from "../api/http";

const form = reactive({ username: "", password: "" });

const submit = async () => {
  const { data } = await http.post("/caiji/login", new URLSearchParams(form));
  if (data.code !== 0) {
    ElMessage.error(data.msg);
    return;
  }
  window.location.href = "/caiji/caijiSettings#/caiji/caijiSettings/caiji";
};
</script>

<template>
  <div class="login-screen">
    <section class="login-stage">
      <div class="login-copy">
        <p class="login-kicker">Dy Server Migration</p>
        <h1>采集管理后台</h1>
        <p>
          旧 PHP 后台正在迁移到 Vue + Vite 新界面。登录后可进入拆分后的采集器、KEY、用户、
          链接和升级码管理页面。
        </p>
        <div class="login-badges">
          <span>Go + Gin API</span>
          <span>Vue 3 Admin</span>
          <span>旧接口兼容</span>
        </div>
      </div>

      <div class="login-panel">
        <div class="login-panel__header">
          <span class="panel-chip">Admin Access</span>
          <h2>登录控制台</h2>
        </div>

        <el-form class="login-form" @submit.prevent="submit">
          <el-form-item>
            <el-input v-model="form.username" placeholder="管理员账号" size="large" />
          </el-form-item>
          <el-form-item>
            <el-input v-model="form.password" type="password" placeholder="登录密码" show-password size="large" />
          </el-form-item>
          <el-button type="primary" native-type="submit" class="full" size="large">进入后台</el-button>
        </el-form>
      </div>
    </section>
  </div>
</template>
