<script setup>
import { onMounted } from "vue";
import { useCaijiAdminStore } from "../../stores/caijiAdmin";

const store = useCaijiAdminStore();

onMounted(async () => {
  await Promise.all([store.loadKeyStore(), store.loadAdminSummary()]);
});
</script>

<template>
  <div class="admin-section">
    <div class="section-header">
      <div>
        <h3>升级管理</h3>
        <p>升级码数据单独成页，避免菜单区挤压内容宽度。</p>
      </div>
      <div class="section-actions">
        <button class="ghost-button ghost-button--primary" @click="store.openAddKeyStoreDialog">新增升级码</button>
        <button class="ghost-button" @click="store.loadKeyStore">刷新</button>
      </div>
    </div>

    <section class="content-card">
      <el-table :data="store.keyStoreList.value" border>
        <el-table-column prop="id" label="ID" width="70" />
        <el-table-column prop="key_code" label="KEY" min-width="160" show-overflow-tooltip />
        <el-table-column prop="keys_manage_key_code" label="关联 KEY" width="160" />
        <el-table-column prop="expiration_date" label="有效期" width="100" />
        <el-table-column prop="expiration_date_unit" label="单位" width="80" />
        <el-table-column prop="is_used" label="已使用" width="80" />
        <el-table-column prop="created_at" label="创建时间" width="180" />
        <el-table-column label="操作" width="160">
          <template #default="{ row }">
            <el-button link type="primary" @click="store.openEditKeyStoreDialog(row)">编辑</el-button>
            <el-button link type="danger" @click="store.deleteKeyStoreItem(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pager">
        <el-pagination
          background
          layout="total, prev, pager, next, sizes"
          :total="store.paging.store.total"
          :current-page="store.paging.store.page"
          :page-size="store.paging.store.page_size"
          :page-sizes="[10, 20, 50, 100]"
          @current-change="(page) => { store.paging.store.page = page; store.loadKeyStore(); }"
          @size-change="(size) => { store.paging.store.page_size = size; store.paging.store.page = 1; store.loadKeyStore(); }"
        />
      </div>
    </section>
  </div>
</template>
