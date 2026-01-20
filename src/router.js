import { createWebHashHistory } from "vue-router";
import { createRouter } from "vue-router";
import useAppStore from "./store";

const routes = [
    {
        path: "/",
        redirect: "/home",
        component: () => import("@/pages/tasks.vue"),
    },
    {
        path: "/home",
        component: () => import("@/pages/tasks.vue"),
        meta: { requiredAuth: true },
    },
    {
        path: "/login",
        name: "login",
        component: () => import("@/pages/login.vue"),
        meta: { title: "用户登录" },
    },
    {
        path: "/register",
        component: () => import("@/pages/register.vue"),
        meta: { title: "用户注册" },
    },
    {
        path: "/tasks",
        component: () => import("@/pages/tasks.vue"),
        meta: { title: "抢福袋任务" },
    },
    {
        path: "/records",
        component: () => import("@/pages/records.vue"),
        meta: { title: "记录" },
    },
];

const router = createRouter({
    history: createWebHashHistory(),
    routes,
});

router.beforeEach(async (to, from, next) => {
    const appStore = useAppStore();
    await appStore.loadData();
    const requiredAuth = to.matched.some((record) => record.meta.requiredAuth);
    if (requiredAuth && !appStore.isAuth) {
        next({ name: "login" });
    } else {
        next();
    }
});

export default router;
