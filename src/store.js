import { defineStore } from "pinia";
import { api_login, api_logout } from "./api";
import { load } from "@tauri-apps/plugin-store";


const LOGIN_INFO_KEY = "login_info_key";
export const useAppStore = defineStore("appStore", {
    LOGIN_INFO_KEY: LOGIN_INFO_KEY,
    state: () => ({
        token: false,
        isAuth: false,
        user: null,
    }),
    actions: {
        async loadData() {
            const store = await this.get_store();
            try {
                this.token = await store.get("token");
                this.isAuth = await store.get("isAuth");
                console.log("this token", this.token, this.isAuth);
            } catch (e) {
                console.log(e, "loading data error");
            }
        },
        async get_store() {
            let store = await load("app.json", {autoSave: false});
            return store;
        },

        async loadStoreKey(key) {
            const store = await this.get_store();
            return await store.get(key);
        },

        async setStoreKey(key, value) {
            const store = await this.get_store();
            await store.set(key, value);
            await store.save();
            return true;
        },

        async login(credentials) {
            const store = await this.get_store();
            const res = await api_login(credentials);
            if (res.data.code !== 0) {
                return [false, res.data];
            }
            store.set("token", credentials.keycode);
            store.set("isAuth", true);
            store.set(LOGIN_INFO_KEY, credentials);
            this.token = credentials.keycode;
            this.isAuth = true;
            await store.save();
            return [true, res.data];
        },

        async logout(credentials) {
            try {
                const store = await this.get_store();
                const res = await api_logout(credentials);
                console.log(`remote res is ${res}`);
                this.token = null;
                this.isAuth = false;
                await store.delete("token");
                await store.delete("isAuth");
                await store.save();
            } catch(e) {
                this.token = null;
                this.isAuth = false;
            }
        },
    },
});

export default useAppStore;
