<script setup>
import { onMounted } from "vue";
import { useCaijiAdminStore } from "../../stores/caijiAdmin";

const store = useCaijiAdminStore();

onMounted(async () => {
  await store.refreshBase();
});
</script>

<template>
  <div class="admin-section">
    <div class="option-slab">
      <div class="option-slab__header">
        <div>
          <span class="option-eyebrow">Collector Params</span>
          <h4>采集参数</h4>
        </div>
        <div class="section-actions">
          <button class="ghost-button ghost-button--primary" @click="store.dialogs.addCaiji = true">添加采集</button>
          <button class="ghost-button" @click="store.dialogs.addCookie = true">添加 Cookie</button>
          <button class="ghost-button" @click="store.allocateCaijiCookie">分配 Cookie</button>

          <button class="ghost-button ghost-button--primary" @click="store.startCaiji">启动采集</button>
          <button class="ghost-button ghost-button--danger" @click="store.stopCaiji">停止采集</button>
          <button class="ghost-button" @click="store.reloadOptions">重载参数</button>
          <button class="ghost-button" @click="store.syncCaijiStatus">同步状态</button>
          <button class="ghost-button" @click="store.refreshCaijiAndCookies">刷新数据</button>
          <button class="ghost-button" @click="store.storeSpiderOption">保存参数</button>
        </div>
      </div>

      <div class="option-form-grid">
        <label class="field-card">
          <span>线程数</span>
          <el-input-number v-model="store.spiderOptions.thread_number" :min="1" :max="20" />
        </label>
        <label class="field-card">
          <span>人数限制</span>
          <el-input-number v-model="store.spiderOptions.people_limit" :min="1" />
        </label>
        <label class="field-card">
          <span>最短间隔</span>
          <el-input-number v-model="store.spiderOptions.min_thread_time" :min="0.1" :step="0.1" />
        </label>
        <label class="field-card">
          <span>最长间隔</span>
          <el-input-number v-model="store.spiderOptions.max_thread_time" :min="0.5" :step="0.1" />
        </label>
        <label class="field-card">
          <span>最低均值</span>
          <el-input-number v-model="store.spiderOptions.min_avg_diamond" :min="1" />
        </label>
        <label class="field-card">
          <span>最低红包数</span>
          <el-input-number v-model="store.spiderOptions.min_packet_number" :min="1" />
        </label>
        <label class="field-card">
          <span>最低钻石</span>
          <el-input-number v-model="store.spiderOptions.min_diamond" :min="1" />
        </label>
        <label class="field-card field-card--wide">
          <span>上传 Token</span>
          <el-input v-model="store.spiderOptions.token" placeholder="Collector token" />
        </label>
      </div>
    </div>

    <div class="split-grid">
      <section class="content-card">
        <div class="card-title-row">
          <div>
            <span class="option-eyebrow">Collectors</span>
            <h4>采集器列表</h4>
          </div>
          <button class="ghost-button ghost-button--danger" @click="store.deleteSelectedCaiji">删除选中</button>
        </div>
        <el-table :data="store.caijiList.value" border @selection-change="store.selectedCaiji.value = $event">
          <el-table-column type="selection" width="48" />
          <el-table-column prop="id" label="ID" width="76" />
          <el-table-column prop="cj_name" label="采集名称" min-width="160" show-overflow-tooltip />
          <el-table-column label="使用 Cookie" min-width="240" show-overflow-tooltip>
            <template #default="{ row }">
              <div class="tag-wrap">
                <el-tag v-for="item in row.cookies || []" :key="item.id" type="info">{{ item.dy_name }}</el-tag>
              </div>
            </template>
          </el-table-column>
          <el-table-column label="状态" width="110">
            <template #default="{ row }">
              <el-tag v-if="Number(row.cj_status) === 2" type="success">working</el-tag>
              <el-tag v-else-if="Number(row.cj_status) === 1" type="primary">online</el-tag>
              <el-tag v-else type="danger">offline</el-tag>
            </template>
          </el-table-column>
        </el-table>
      </section>

      <section class="content-card">
        <div class="card-title-row">
          <div>
            <span class="option-eyebrow">Cookies</span>
            <h4>Cookie 列表</h4>
          </div>
          <button class="ghost-button ghost-button--danger" @click="store.deleteSelectedCookie">删除选中</button>
        </div>
        <el-table :data="store.cookieList.value" border @selection-change="store.selectedCookies.value = $event">
          <el-table-column type="selection" width="48" />
          <el-table-column prop="id" label="ID" width="76" />
          <el-table-column prop="dy_name" label="抖音名称" width="160" show-overflow-tooltip />
          <el-table-column label="Cookie 摘要" min-width="260">
            <template #default="{ row }">
              <div class="cookie-summary" :title="row.cookie">{{ row.cookie }}</div>
            </template>
          </el-table-column>
          <el-table-column label="采集器" min-width="200" show-overflow-tooltip>
            <template #default="{ row }">
              <div class="tag-wrap">
                <el-tag v-for="item in row.caiji || []" :key="item.id" type="info">{{ item.cj_name }}</el-tag>
              </div>
            </template>
          </el-table-column>
          <el-table-column label="可用" width="96">
            <template #default="{ row }">
              <el-tag :type="Number(row.can_use) === 1 ? 'success' : 'warning'">{{ Number(row.can_use) === 1 ? "可用" : "过期" }}</el-tag>
            </template>
          </el-table-column>
        </el-table>
        <div class="pager">
          <el-pagination
            background
            layout="total, prev, pager, next, sizes"
            :total="store.paging.cookies.total"
            :current-page="store.paging.cookies.page"
            :page-size="store.paging.cookies.page_size"
            :page-sizes="[10, 20, 50, 100]"
            @current-change="(page) => { store.paging.cookies.page = page; store.loadCookies(); }"
            @size-change="(size) => { store.paging.cookies.page_size = size; store.paging.cookies.page = 1; store.loadCookies(); }"
          />
        </div>
      </section>
    </div>
  </div>
</template>
