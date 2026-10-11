import { reactive, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import http from "../api/http";

const caijiList = ref([]);
const cookieList = ref([]);
const keysManageList = ref([]);
const usersList = ref([]);
const roomUrlList = ref([]);
const keyStoreList = ref([]);
const selectedCaiji = ref([]);
const selectedCookies = ref([]);
const autoRefresh = ref(false);

const adminSummary = reactive({
  keys_manage: [],
  users: [],
  room_url: [],
  key_store: []
});

const spiderOptions = reactive({
  thread_number: 15,
  people_limit: 60,
  min_thread_time: 0.9,
  max_thread_time: 2.2,
  min_avg_diamond: 4,
  min_diamond: 50,
  min_packet_number: 2,
  token: "SPARKS1"
});

const paging = reactive({
	cookies: { page: 1, page_size: 10, total: 0 },
  keys: { page: 1, page_size: 10, total: 0 },
  users: { page: 1, page_size: 10, total: 0 },
  rooms: { page: 1, page_size: 10, total: 0 },
  store: { page: 1, page_size: 10, total: 0 }
});

const dialogs = reactive({
  addCaiji: false,
  addCookie: false,
  addKeys: false,
  editKeys: false,
  editUser: false,
  addKeyStore: false,
  editKeyStore: false
});

const addCaijiForm = reactive({ cj_name: "" });
const addCookieForm = reactive({ dy_name: "", cookie: "" });
const addKeysForm = reactive({ key_code: "", user_id: null, expired_at: "", is_expired: 0 });
const editKeysForm = reactive({ id: null, key_code: "", user_id: null, expired_at: "", is_expired: 0 });
const editUserForm = reactive({ id: null, username: "", machine_id: "", expired_at: "" });
const addKeyStoreForm = reactive({
  kid: null,
  key_code: "",
  expiration_date: 1,
  expiration_date_unit: "m",
  is_used: 0,
  remark: "",
  created_at: "",
  updated_at: ""
});
const editKeyStoreForm = reactive({
  id: null,
  kid: null,
  key_code: "",
  expiration_date: 1,
  expiration_date_unit: "m",
  is_used: 0,
  remark: "",
  created_at: "",
  updated_at: ""
});

let refreshTimer = null;
const OPT_STORE_KEY = "dy_caiji_spider_options";

function ensureArray(value) {
  return Array.isArray(value) ? value : [];
}

function formatDate(date) {
  const y = date.getFullYear();
  const m = `${date.getMonth() + 1}`.padStart(2, "0");
  const d = `${date.getDate()}`.padStart(2, "0");
  const h = `${date.getHours()}`.padStart(2, "0");
  const i = `${date.getMinutes()}`.padStart(2, "0");
  const s = `${date.getSeconds()}`.padStart(2, "0");
  return `${y}-${m}-${d} ${h}:${i}:${s}`;
}

function normalizeDateTime(value) {
  if (!value) return "";
  const text = String(value).replace("T", " ").replace(/\.\d+Z?$/, "");
  if (/^\d+$/.test(text)) {
    const date = new Date(Number(text) * 1000);
    return formatDate(date);
  }
  const date = new Date(text.replace(" ", "T"));
  return Number.isNaN(date.getTime()) ? text.slice(0, 19) : formatDate(date);
}

function nowDateTime() {
  return formatDate(new Date());
}

function randomKeyCode() {
  const chars = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
  return Array.from({ length: 8 }, () => chars[Math.floor(Math.random() * chars.length)]).join("");
}

function randomSoftwareCode() {
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  return Array.from({ length: 16 }, () => chars[Math.floor(Math.random() * chars.length)]).join("");
}

function storeSpiderOption() {
  localStorage.setItem(OPT_STORE_KEY, JSON.stringify(spiderOptions));
  ElMessage.success("参数已保存到本地");
}

function restoreSpiderOption() {
  const raw = localStorage.getItem(OPT_STORE_KEY);
  if (!raw) return;
  try {
    Object.assign(spiderOptions, JSON.parse(raw));
  } catch {
    localStorage.removeItem(OPT_STORE_KEY);
  }
}

async function request(method, url, payload, successMsg) {
  try {
    let response;
    if (method === "get") {
      response = await http.get(url, { params: payload || {} });
    } else {
      response = await http[method](url, payload || {});
    }
    const { data } = response;
    if (data.code !== 0) {
      throw new Error(data.msg || "请求失败");
    }
    if (successMsg) ElMessage.success(successMsg);
    return data.data;
  } catch (error) {
    const message = error?.message || "请求失败";
    ElMessage.error(message);
    throw error;
  }
}

async function loadAdminSummary() {
  const data = await request("get", "/caiji/adminSummary", null);
  adminSummary.keys_manage = ensureArray(data?.keys_manage).map((item) => ({ ...item, expired_at: normalizeDateTime(item.expired_at) }));
  adminSummary.users = ensureArray(data?.users).map((item) => ({ ...item, expired_at: normalizeDateTime(item.expired_at) }));
  adminSummary.room_url = ensureArray(data?.room_url);
  adminSummary.key_store = ensureArray(data?.key_store);
}

async function loadCaiji() {
  const data = await request("get", "/caiji/getCaijiList", null);
  caijiList.value = ensureArray(data?.cj_list).map((item) => ({
    ...item,
    cookies: ensureArray(item?.cookies)
  }));
}

async function loadCookies() {
	const data = await request("get", "/caiji/cookieList", paging.cookies);
	const list = Array.isArray(data) ? data : data?.list;
	cookieList.value = ensureArray(list).map((item) => ({
		...item,
		caiji: ensureArray(item?.caiji)
	}));
	if (!Array.isArray(data)) {
		Object.assign(paging.cookies, {
			total: data?.total || 0,
			page: data?.page || 1,
			page_size: data?.page_size || 10
		});
	}
}

async function loadKeys() {
  const data = await request("get", "/caiji/keysManageList", paging.keys);
  keysManageList.value = ensureArray(data?.list).map((item) => ({
    ...item,
    expired_at: normalizeDateTime(item.expired_at),
    created_at: normalizeDateTime(item.created_at),
    updated_at: normalizeDateTime(item.updated_at)
  }));
  Object.assign(paging.keys, {
    total: data?.total || 0,
    page: data?.page || 1,
    page_size: data?.page_size || 10
  });
}

async function loadUsers() {
  const data = await request("get", "/caiji/usersList", paging.users);
  usersList.value = ensureArray(data?.list).map((item) => ({
    ...item,
    expired_at: normalizeDateTime(item.expired_at),
    created_at: normalizeDateTime(item.created_at),
    updated_at: normalizeDateTime(item.updated_at)
  }));
  Object.assign(paging.users, {
    total: data?.total || 0,
    page: data?.page || 1,
    page_size: data?.page_size || 10
  });
}

async function loadRooms() {
  const data = await request("get", "/caiji/roomUrlList", paging.rooms);
  roomUrlList.value = ensureArray(data?.list).map((item) => ({
    ...item,
    send_time_text: normalizeDateTime(item.send_time),
    expired_at_text: normalizeDateTime(item.expired_at)
  }));
  Object.assign(paging.rooms, {
    total: data?.total || 0,
    page: data?.page || 1,
    page_size: data?.page_size || 10
  });
}

async function loadKeyStore() {
  const data = await request("get", "/caiji/keyStoreList", paging.store);
  keyStoreList.value = ensureArray(data?.list).map((item) => ({
    ...item,
    keys_manage_key_code: item.keys_manage?.key_code || "",
    created_at: normalizeDateTime(item.created_at),
    updated_at: normalizeDateTime(item.updated_at || item.created_at)
  }));
  Object.assign(paging.store, {
    total: data?.total || 0,
    page: data?.page || 1,
    page_size: data?.page_size || 10
  });
}

async function refreshBase() {
  const results = await Promise.allSettled([loadCaiji(), loadCookies()]);
  if (results.every((item) => item.status === "rejected")) {
    throw new Error("基础数据加载失败");
  }
}

async function refreshCaijiAndCookies() {
  await refreshBase();
  ElMessage.success("数据已刷新");
}

function setAutoRefresh(enabled) {
  if (refreshTimer) {
    clearInterval(refreshTimer);
    refreshTimer = null;
  }
  if (enabled) {
    refreshTimer = setInterval(() => {
      loadCaiji().catch(() => {});
    }, 2000);
  }
}

async function confirmCaijiAction(title, url, extraPayload = {}) {
  if (!selectedCaiji.value.length) {
    ElMessage.warning("请先选择采集器");
    return;
  }
  await ElMessageBox.confirm(title);
  const cj_name = selectedCaiji.value.map((item) => item.cj_name);
  await request("post", url, { cj_name, ...extraPayload }, "操作成功");
  await loadCaiji();
}

async function startCaiji() {
  await confirmCaijiAction("确定开始采集？", "/caiji/startCaiji", spiderOptions);
  storeSpiderOption();
}

async function stopCaiji() {
  await confirmCaijiAction("确定停止采集？", "/caiji/stopCaiji");
}

async function reloadOptions() {
  await confirmCaijiAction("确定重载参数？", "/caiji/reloadOptions");
}

async function syncCaijiStatus() {
  await confirmCaijiAction("确定同步状态？", "/caiji/syncCaijiStatus");
}

async function allocateCaijiCookie() {
  if (!selectedCaiji.value.length) {
    ElMessage.warning("请先选择采集器");
    return;
  }
  if (!selectedCookies.value.length) {
    ElMessage.warning("请先选择 Cookie");
    return;
  }
  await ElMessageBox.confirm("确认分配选中的 Cookie 到采集器？");
  await request("post", "/caiji/allocateCookie", {
    caiji: selectedCaiji.value.map((item) => item.id),
    cookie: selectedCookies.value.map((item) => item.id)
  }, "分配成功");
  await Promise.all([loadCaiji(), loadCookies()]);
}

async function deleteSelectedCaiji() {
  if (!selectedCaiji.value.length) {
    ElMessage.warning("请先选择采集器");
    return;
  }
  await ElMessageBox.confirm("确定删除选中的采集器？");
  await request("post", "/caiji/deleteCaiji", { id: selectedCaiji.value.map((item) => item.id) }, "删除成功");
  selectedCaiji.value = [];
  await Promise.all([loadCaiji(), loadCookies()]);
}

async function deleteSelectedCookie() {
  if (!selectedCookies.value.length) {
    ElMessage.warning("请先选择 Cookie");
    return;
  }
  await ElMessageBox.confirm("确定删除选中的 Cookie？");
  await request("post", "/caiji/deleteCookie", { id: selectedCookies.value.map((item) => item.id) }, "删除成功");
  selectedCookies.value = [];
  await loadCookies();
}

async function submitAddCaiji() {
  await request("post", "/caiji/addCaiji", { cj_name: addCaijiForm.cj_name }, "保存成功");
  addCaijiForm.cj_name = "";
  dialogs.addCaiji = false;
  await loadCaiji();
}

async function submitAddCookie() {
  await request("post", "/caiji/addCookie", { dy_name: addCookieForm.dy_name, cookie: addCookieForm.cookie }, "保存成功");
  addCookieForm.dy_name = "";
  addCookieForm.cookie = "";
  dialogs.addCookie = false;
  await loadCookies();
}

function openAddKeysDialog() {
  addKeysForm.key_code = randomKeyCode();
  addKeysForm.expired_at = nowDateTime();
  addKeysForm.user_id = null;
  addKeysForm.is_expired = 0;
  dialogs.addKeys = true;
}

async function submitAddKeys() {
  await request("post", "/caiji/addKeysManage", { ...addKeysForm }, "保存成功");
  dialogs.addKeys = false;
  await Promise.all([loadKeys(), loadAdminSummary()]);
}

function openEditKeysDialog(row) {
  Object.assign(editKeysForm, {
    id: row.id,
    key_code: row.key_code,
    user_id: row.user_id,
    expired_at: row.expired_at,
    is_expired: Number(row.is_expired || 0)
  });
  dialogs.editKeys = true;
}

async function submitEditKeys() {
  await request("post", "/caiji/updateKeysManage", { ...editKeysForm }, "保存成功");
  dialogs.editKeys = false;
  await Promise.all([loadKeys(), loadAdminSummary()]);
}

async function deleteKeysItem(row) {
  await ElMessageBox.confirm("确定删除这条 KEY 记录？");
  await request("post", "/caiji/deleteKeysManage", { id: row.id }, "删除成功");
  await loadKeys();
}

function openEditUserDialog(row) {
  Object.assign(editUserForm, {
    id: row.id,
    username: row.username || "",
    machine_id: row.machine_id || "",
    expired_at: row.expired_at || ""
  });
  dialogs.editUser = true;
}

async function submitEditUser() {
  await request("post", "/caiji/updateUser", { ...editUserForm }, "保存成功");
  dialogs.editUser = false;
  await Promise.all([loadUsers(), loadAdminSummary()]);
}

function openAddKeyStoreDialog() {
  Object.assign(addKeyStoreForm, {
    kid: null,
    key_code: randomSoftwareCode(),
    expiration_date: 1,
    expiration_date_unit: "m",
    is_used: 0,
    remark: "",
    created_at: nowDateTime(),
    updated_at: nowDateTime()
  });
  dialogs.addKeyStore = true;
}

async function submitAddKeyStore() {
  await request("post", "/caiji/addKeyStore", { ...addKeyStoreForm }, "保存成功");
  dialogs.addKeyStore = false;
  await Promise.all([loadKeyStore(), loadAdminSummary()]);
}

function openEditKeyStoreDialog(row) {
  Object.assign(editKeyStoreForm, {
    id: row.id,
    kid: row.kid,
    key_code: row.key_code,
    expiration_date: row.expiration_date,
    expiration_date_unit: row.expiration_date_unit,
    is_used: row.is_used,
    remark: row.remark,
    created_at: row.created_at,
    updated_at: row.updated_at
  });
  dialogs.editKeyStore = true;
}

async function submitEditKeyStore() {
  await request("post", "/caiji/updateKeyStore", { ...editKeyStoreForm }, "保存成功");
  dialogs.editKeyStore = false;
  await Promise.all([loadKeyStore(), loadAdminSummary()]);
}

async function deleteKeyStoreItem(row) {
  await ElMessageBox.confirm("确定删除这条升级码记录？");
  await request("post", "/caiji/deleteKeyStore", { id: row.id }, "删除成功");
  await loadKeyStore();
}

export function useCaijiAdminStore() {
  return {
    caijiList,
    cookieList,
    keysManageList,
    usersList,
    roomUrlList,
    keyStoreList,
    selectedCaiji,
    selectedCookies,
    autoRefresh,
    adminSummary,
    spiderOptions,
    paging,
    dialogs,
    addCaijiForm,
    addCookieForm,
    addKeysForm,
    editKeysForm,
    editUserForm,
    addKeyStoreForm,
    editKeyStoreForm,
    normalizeDateTime,
    nowDateTime,
    randomKeyCode,
    randomSoftwareCode,
    restoreSpiderOption,
    storeSpiderOption,
    refreshBase,
    refreshCaijiAndCookies,
    loadAdminSummary,
    loadCaiji,
    loadCookies,
    loadKeys,
    loadUsers,
    loadRooms,
    loadKeyStore,
    setAutoRefresh,
    startCaiji,
    stopCaiji,
    reloadOptions,
    syncCaijiStatus,
    allocateCaijiCookie,
    deleteSelectedCaiji,
    deleteSelectedCookie,
    submitAddCaiji,
    submitAddCookie,
    openAddKeysDialog,
    submitAddKeys,
    openEditKeysDialog,
    submitEditKeys,
    deleteKeysItem,
    openEditUserDialog,
    submitEditUser,
    openAddKeyStoreDialog,
    submitAddKeyStore,
    openEditKeyStoreDialog,
    submitEditKeyStore,
    deleteKeyStoreItem
  };
}
