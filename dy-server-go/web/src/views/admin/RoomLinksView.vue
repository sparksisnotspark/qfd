<script setup>
import { onMounted } from "vue";
import { useCaijiAdminStore } from "../../stores/caijiAdmin";

const store = useCaijiAdminStore();

onMounted(async () => {
  await store.loadRooms();
});
</script>

<template>
  <div class="admin-section">
    <div class="section-header">
      <div>
        <h3>链接管理</h3>
        <p>房间链接队列单独成页，更适合后续加筛选和统计。</p>
      </div>
      <div class="section-actions">
        <button class="ghost-button" @click="store.loadRooms">刷新</button>
      </div>
    </div>

    <section class="content-card">
      <el-table :data="store.roomUrlList.value" border>
        <el-table-column prop="id" label="ID" width="90" />
        <el-table-column prop="room_url" label="链接" min-width="280" show-overflow-tooltip />
        <el-table-column prop="room_id" label="房间ID" width="150" />
        <el-table-column prop="web_rid" label="WebRid" width="150" />
        <el-table-column prop="title" label="标题" min-width="180" show-overflow-tooltip />
        <el-table-column prop="tp" label="类型" width="80" />
        <el-table-column prop="used_times" label="次数" width="80" />
        <el-table-column prop="delay" label="延迟" width="80" />
      </el-table>
      <div class="pager">
        <el-pagination
          background
          layout="total, prev, pager, next, sizes"
          :total="store.paging.rooms.total"
          :current-page="store.paging.rooms.page"
          :page-size="store.paging.rooms.page_size"
          :page-sizes="[10, 20, 50, 100]"
          @current-change="(page) => { store.paging.rooms.page = page; store.loadRooms(); }"
          @size-change="(size) => { store.paging.rooms.page_size = size; store.paging.rooms.page = 1; store.loadRooms(); }"
        />
      </div>
    </section>
  </div>
</template>
