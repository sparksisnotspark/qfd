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
<script>
import { listen } from "@tauri-apps/api/event";
import { ElMessage } from "element-plus";
import { onMounted } from "vue";
onMounted(async () => {
    const unlisten = await listen("error", (event) => {
        console.log(`Got error , payload is: ${event.payload}`);
        ElMessage.error(event.payload);
    });
    const success = await listen("notify", (event) => {
        ElMessage.success(event.payload);
    });
})
</script>
