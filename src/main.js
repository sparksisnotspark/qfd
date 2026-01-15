import { createApp } from "vue";
import { createPinia } from "pinia";
import { Store } from "@tauri-apps/plugin-store";
import ElementPlus from "element-plus";
import "element-plus/dist/index.css";

import App from "./MyApp.vue";

import router from "./router";

const pinia = createPinia();

createApp(App).use(ElementPlus).use(pinia).use(router).mount("#app");
