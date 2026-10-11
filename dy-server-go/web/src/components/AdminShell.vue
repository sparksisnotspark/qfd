<script setup>
import { computed, onBeforeUnmount, onMounted } from "vue";
import { useRoute } from "vue-router";
import { useCaijiAdminStore } from "../stores/caijiAdmin";
import AdminDialogs from "./AdminDialogs.vue";

const route = useRoute();
const store = useCaijiAdminStore();

const navItems = [
  { key: "caiji", label: "采集管理", path: "/caiji/caijiSettings/caiji" },
  { key: "keys", label: "KEYS 管理", path: "/caiji/caijiSettings/keys" },
  { key: "users", label: "用户管理", path: "/caiji/caijiSettings/users" },
  { key: "rooms", label: "链接管理", path: "/caiji/caijiSettings/rooms" },
  { key: "store", label: "升级管理", path: "/caiji/caijiSettings/store" }
];

const activeMenu = computed(() => route.path);

function logout() {
  window.location.href = "/caiji/logout";
}

onMounted(async () => {
  store.restoreSpiderOption();
  await store.refreshBase();
});

onBeforeUnmount(() => store.setAutoRefresh(false));
</script>

<template>
  <div class="admin-shell">
    <aside class="admin-sidebar">
      <div class="brand-mark">
        <p>Dy Server</p>
        <h1>采集后台</h1>
      </div>
      <nav class="nav-stack">
        <RouterLink
          v-for="item in navItems"
          :key="item.key"
          :to="item.path"
          class="nav-link"
          :class="{ 'is-active': activeMenu === item.path }"
        >
          <span class="nav-dot"></span>
          {{ item.label }}
        </RouterLink>
      </nav>
      <button class="sidebar-logout" @click="logout">退出登录</button>
    </aside>

    <div class="admin-main">
      <section class="route-panel">
        <RouterView />
      </section>
    </div>

    <AdminDialogs />
  </div>
</template>
