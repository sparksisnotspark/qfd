import { createRouter, createWebHashHistory } from "vue-router";
import LoginView from "../views/LoginView.vue";
import AdminView from "../views/AdminView.vue";
import CaijiDashboardView from "../views/admin/CaijiDashboardView.vue";
import KeysManageView from "../views/admin/KeysManageView.vue";
import UsersManageView from "../views/admin/UsersManageView.vue";
import RoomLinksView from "../views/admin/RoomLinksView.vue";
import KeyStoreView from "../views/admin/KeyStoreView.vue";

export default createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: "/caiji/login" },
    { path: "/caiji/login", component: LoginView },
    {
      path: "/caiji/caijiSettings",
      component: AdminView,
      children: [
        { path: "", redirect: "/caiji/caijiSettings/caiji" },
        { path: "caiji", component: CaijiDashboardView },
        { path: "keys", component: KeysManageView },
        { path: "users", component: UsersManageView },
        { path: "rooms", component: RoomLinksView },
        { path: "store", component: KeyStoreView }
      ]
    }
  ]
});
