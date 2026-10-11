<script setup>
import { onMounted } from "vue";
import { useCaijiAdminStore } from "../../stores/caijiAdmin";

const store = useCaijiAdminStore();

onMounted(async () => {
  await Promise.all([store.loadKeys(), store.loadAdminSummary()]);
});
</script>

<template>
  <div class="admin-section">
    <div class="section-header">
      <div>
        <h3>KEYS 管理</h3>
        <p>把旧页里的表格和弹窗拆到独立路由，避免切换时整页重排。</p>
      </div>
      <div class="section-actions">
        <button class="ghost-button ghost-button--primary" @click="store.openAddKeysDialog">新增 KEY</button>
        <button class="ghost-button" @click="store.loadKeys">刷新</button>
      </div>
    </div>

    <section class="content-card">
      <el-table :data="store.keysManageList.value" border>
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="key_code" label="KEY CODE" min-width="180" />
        <el-table-column prop="user_id" label="用户ID" width="100" />
        <el-table-column prop="expired_at" label="过期时间" width="180" />
        <el-table-column prop="is_expired" label="已过期" width="100" />
        <el-table-column prop="created_at" label="创建时间" width="180" />
        <el-table-column label="操作" width="160">
          <template #default="{ row }">
            <el-button link type="primary" @click="store.openEditKeysDialog(row)">编辑</el-button>
            <el-button link type="danger" @click="store.deleteKeysItem(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pager">
        <el-pagination
          background
          layout="total, prev, pager, next, sizes"
          :total="store.paging.keys.total"
          :current-page="store.paging.keys.page"
          :page-size="store.paging.keys.page_size"
          :page-sizes="[10, 20, 50, 100]"
          @current-change="(page) => { store.paging.keys.page = page; store.loadKeys(); }"
          @size-change="(size) => { store.paging.keys.page_size = size; store.paging.keys.page = 1; store.loadKeys(); }"
        />
      </div>
    </section>
  </div>
</template>
