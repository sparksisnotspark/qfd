<template>
    <main class="container">
        <router-view></router-view>
    </main>
</template>

<style lang="css" scoped>
.container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100%;
    padding: 0;
    margin: 0;
}

html,
body,
html body {
    margin: 0;
    padding: 0;
}

#app {
    margin: 0;
    padding: 0;
}
</style>
<script setup>
import { listen, TauriEvent } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { ElMessage, ElMessageBox } from "element-plus";
import { onMounted, onUnmounted } from "vue";

const appWindow = getCurrentWindow();

let errorListener = null;
let closeListener = null;



onMounted(async () => {
    console.log("Listening for window close events...");

    errorListener = await listen("error", (event) => {
        console.log(`Got error , payload is: ${event.payload}`);
        ElMessage.error(event.payload);
    });

    appWindow.listen(TauriEvent.WINDOW_CLOSE_REQUESTED, (event) => {
        ElMessageBox.confirm("确认关闭窗口", "警告", {
            confirmButtonText: "仍要关闭",
            cancelButtonText: "再想一想",
            type: "warning",
        }).then(async () => {
            await invoke("exit_app_with_front");
        }).catch((e) => {
            console.log("关闭窗口出错", e);
        });
    });

    const success = await listen("notify", (event) => {
        ElMessage.success(event.payload);
    });
})

onUnmounted(() => {
})

</script>
