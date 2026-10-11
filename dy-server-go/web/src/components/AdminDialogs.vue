<script setup>
import { useCaijiAdminStore } from "../stores/caijiAdmin";

const store = useCaijiAdminStore();
</script>

<template>
  <el-dialog v-model="store.dialogs.addCaiji" title="添加采集器" width="480px">
    <el-form label-width="90px">
      <el-form-item label="采集名称">
        <el-input v-model="store.addCaijiForm.cj_name" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="store.dialogs.addCaiji = false">取消</el-button>
      <el-button type="primary" @click="store.submitAddCaiji">保存</el-button>
    </template>
  </el-dialog>

  <el-dialog v-model="store.dialogs.addCookie" title="添加 Cookie" width="720px">
    <el-form label-width="90px">
      <el-form-item label="抖音名称">
        <el-input v-model="store.addCookieForm.dy_name" />
      </el-form-item>
      <el-form-item label="Cookie">
        <el-input v-model="store.addCookieForm.cookie" type="textarea" :rows="10" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="store.dialogs.addCookie = false">取消</el-button>
      <el-button type="primary" @click="store.submitAddCookie">保存</el-button>
    </template>
  </el-dialog>

  <el-dialog v-model="store.dialogs.addKeys" title="新增 KEY" width="560px">
    <el-form label-width="100px">
      <el-form-item label="KEY CODE">
        <el-input v-model="store.addKeysForm.key_code">
          <template #append><el-button @click="store.addKeysForm.key_code = store.randomKeyCode()">随机</el-button></template>
        </el-input>
      </el-form-item>
      <el-form-item label="对应用户">
        <el-select v-model="store.addKeysForm.user_id" clearable filterable style="width: 100%">
          <el-option v-for="item in store.adminSummary.users" :key="item.id" :label="`${item.id} - ${item.username}`" :value="item.id" />
        </el-select>
      </el-form-item>
      <el-form-item label="过期时间">
        <el-date-picker v-model="store.addKeysForm.expired_at" type="datetime" value-format="YYYY-MM-DD HH:mm:ss" style="width: 100%" />
      </el-form-item>
      <el-form-item label="已过期">
        <el-input-number v-model="store.addKeysForm.is_expired" :min="0" :max="1" style="width: 100%" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="store.dialogs.addKeys = false">取消</el-button>
      <el-button type="primary" @click="store.submitAddKeys">保存</el-button>
    </template>
  </el-dialog>

  <el-dialog v-model="store.dialogs.editKeys" title="编辑 KEY" width="560px">
    <el-form label-width="100px">
      <el-form-item label="KEY CODE">
        <el-input v-model="store.editKeysForm.key_code">
          <template #append><el-button @click="store.editKeysForm.key_code = store.randomKeyCode()">随机</el-button></template>
        </el-input>
      </el-form-item>
      <el-form-item label="对应用户">
        <el-select v-model="store.editKeysForm.user_id" clearable filterable style="width: 100%">
          <el-option v-for="item in store.adminSummary.users" :key="item.id" :label="`${item.id} - ${item.username}`" :value="item.id" />
        </el-select>
      </el-form-item>
      <el-form-item label="过期时间">
        <el-date-picker v-model="store.editKeysForm.expired_at" type="datetime" value-format="YYYY-MM-DD HH:mm:ss" style="width: 100%" />
      </el-form-item>
      <el-form-item label="已过期">
        <el-input-number v-model="store.editKeysForm.is_expired" :min="0" :max="1" style="width: 100%" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="store.dialogs.editKeys = false">取消</el-button>
      <el-button type="primary" @click="store.submitEditKeys">保存</el-button>
    </template>
  </el-dialog>

  <el-dialog v-model="store.dialogs.editUser" title="编辑用户" width="560px">
    <el-form label-width="100px">
      <el-form-item label="用户名"><el-input v-model="store.editUserForm.username" /></el-form-item>
      <el-form-item label="机器 ID"><el-input v-model="store.editUserForm.machine_id" /></el-form-item>
      <el-form-item label="过期时间">
        <el-date-picker v-model="store.editUserForm.expired_at" type="datetime" value-format="YYYY-MM-DD HH:mm:ss" style="width: 100%" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="store.dialogs.editUser = false">取消</el-button>
      <el-button type="primary" @click="store.submitEditUser">保存</el-button>
    </template>
  </el-dialog>

  <el-dialog v-model="store.dialogs.addKeyStore" title="新增升级码" width="620px">
    <el-form label-width="110px">
      <el-form-item label="关联 KEY">
        <el-select v-model="store.addKeyStoreForm.kid" clearable filterable style="width: 100%">
          <el-option v-for="item in store.adminSummary.keys_manage" :key="item.id" :label="`${item.id} - ${item.key_code}`" :value="item.id" />
        </el-select>
      </el-form-item>
      <el-form-item label="软件 CODE">
        <el-input v-model="store.addKeyStoreForm.key_code">
          <template #append><el-button @click="store.addKeyStoreForm.key_code = store.randomSoftwareCode()">随机</el-button></template>
        </el-input>
      </el-form-item>
      <el-form-item label="有效期"><el-input-number v-model="store.addKeyStoreForm.expiration_date" :min="1" style="width: 100%" /></el-form-item>
      <el-form-item label="单位"><el-input v-model="store.addKeyStoreForm.expiration_date_unit" /></el-form-item>
      <el-form-item label="已使用"><el-input-number v-model="store.addKeyStoreForm.is_used" :min="0" :max="1" style="width: 100%" /></el-form-item>
      <el-form-item label="备注"><el-input v-model="store.addKeyStoreForm.remark" type="textarea" :rows="3" /></el-form-item>
      <el-form-item label="创建时间"><el-date-picker v-model="store.addKeyStoreForm.created_at" type="datetime" value-format="YYYY-MM-DD HH:mm:ss" style="width: 100%" /></el-form-item>
      <el-form-item label="更新时间"><el-date-picker v-model="store.addKeyStoreForm.updated_at" type="datetime" value-format="YYYY-MM-DD HH:mm:ss" style="width: 100%" /></el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="store.dialogs.addKeyStore = false">取消</el-button>
      <el-button type="primary" @click="store.submitAddKeyStore">保存</el-button>
    </template>
  </el-dialog>

  <el-dialog v-model="store.dialogs.editKeyStore" title="编辑升级码" width="620px">
    <el-form label-width="110px">
      <el-form-item label="关联 KEY">
        <el-select v-model="store.editKeyStoreForm.kid" clearable filterable style="width: 100%">
          <el-option v-for="item in store.adminSummary.keys_manage" :key="item.id" :label="`${item.id} - ${item.key_code}`" :value="item.id" />
        </el-select>
      </el-form-item>
      <el-form-item label="软件 CODE">
        <el-input v-model="store.editKeyStoreForm.key_code">
          <template #append><el-button @click="store.editKeyStoreForm.key_code = store.randomSoftwareCode()">随机</el-button></template>
        </el-input>
      </el-form-item>
      <el-form-item label="有效期"><el-input-number v-model="store.editKeyStoreForm.expiration_date" :min="1" style="width: 100%" /></el-form-item>
      <el-form-item label="单位"><el-input v-model="store.editKeyStoreForm.expiration_date_unit" /></el-form-item>
      <el-form-item label="已使用"><el-input-number v-model="store.editKeyStoreForm.is_used" :min="0" :max="1" style="width: 100%" /></el-form-item>
      <el-form-item label="备注"><el-input v-model="store.editKeyStoreForm.remark" type="textarea" :rows="3" /></el-form-item>
      <el-form-item label="创建时间"><el-date-picker v-model="store.editKeyStoreForm.created_at" type="datetime" value-format="YYYY-MM-DD HH:mm:ss" style="width: 100%" /></el-form-item>
      <el-form-item label="更新时间"><el-date-picker v-model="store.editKeyStoreForm.updated_at" type="datetime" value-format="YYYY-MM-DD HH:mm:ss" style="width: 100%" /></el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="store.dialogs.editKeyStore = false">取消</el-button>
      <el-button type="primary" @click="store.submitEditKeyStore">保存</el-button>
    </template>
  </el-dialog>
</template>
