<script setup>
import { onMounted } from "vue";
import { useCaijiAdminStore } from "../../stores/caijiAdmin";

const store = useCaijiAdminStore();

onMounted(async () => {
  await store.loadUsers();
});
</script>

<template>
  <div class="admin-section">
    <div class="section-header">
      <div>
        <h3>用户管理</h3>
        <p>用户编辑独立出来，避免跟采集器面板混在一页。</p>
      </div>
      <div class="section-actions">
        <button class="ghost-button" @click="store.loadUsers">刷新</button>
      </div>
    </div>

    <section class="content-card">
      <el-table :data="store.usersList.value" border>
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="username" label="用户名" width="220" />
        <el-table-column prop="machine_id" label="机器ID" width="180" />
        <el-table-column prop="expired_at" label="过期时间" width="180" />
        <el-table-column prop="created_at" label="创建时间" width="180" />
        <el-table-column prop="updated_at" label="更新时间" width="180" />
        <el-table-column label="操作" width="120">
          <template #default="{ row }">
            <el-button link type="primary" @click="store.openEditUserDialog(row)">编辑</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pager">
        <el-pagination
          background
          layout="total, prev, pager, next, sizes"
          :total="store.paging.users.total"
          :current-page="store.paging.users.page"
          :page-size="store.paging.users.page_size"
          :page-sizes="[10, 20, 50, 100]"
          @current-change="(page) => { store.paging.users.page = page; store.loadUsers(); }"
          @size-change="(size) => { store.paging.users.page_size = size; store.paging.users.page = 1; store.loadUsers(); }"
        />
      </div>
    </section>
  </div>
</template>
