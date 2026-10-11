# dy-server-go

`D:/dy_server` 旧 PHP 项目的迁移仓库。

目标：

- 后端迁移到 `Go + gin`
- 前端迁移到 `Vite + Vue 3 + Element Plus + Pinia`
- 保持旧项目里 `ApiController`、`AppController`、`AuthController` 的旧接口路径兼容
- 管理后台围绕旧项目 `CaijiController` 的逻辑迁移

## 目录

- `server/`: Go 后端
- `web/`: Vue 管理后台
- `dy_server.sql`: 当前迁移使用的数据库结构参考
- `AGENTS.md`: 仓库级迁移约束

## 旧项目来源

- 旧项目路径：`D:/dy_server`
- 明确忽略：`D:/dy_server/app/process/CaijiServer.php`
- 明确忽略：旧项目 `rust/` 目录

## 当前技术栈

### 后端

- `gin`
- `gorm`
- `go-redis`

### 前端

- `vite`
- `vue`
- `element-plus`
- `pinia`
- `axios`

## 环境要求

### 后端

- Go `1.26+`
- MySQL
- Redis

### 前端

- Node.js `22+`
- pnpm `10+`

## 后端配置

参考文件：

- [server/.env.example](D:/dy-server-go/server/.env.example)

当前代码默认读取环境变量，核心项：

- `APP_HOST`
- `APP_PORT`
- `MYSQL_DSN`
- `REDIS_ADDR`
- `REDIS_PASSWORD`
- `CAIJI_PROXY_URL`
- `CAIJI_ADMIN_USER`
- `CAIJI_ADMIN_PASSWORD`

示例：

```powershell
$env:APP_HOST="0.0.0.0"
$env:APP_PORT="8080"
$env:MYSQL_DSN="root:root@tcp(127.0.0.1:3306)/dy_server?charset=utf8mb4&parseTime=True&loc=Local"
$env:REDIS_ADDR="127.0.0.1:6379"
$env:CAIJI_PROXY_URL="http://127.0.0.1:28888"
$env:CAIJI_ADMIN_USER="312790696@qq.com"
$env:CAIJI_ADMIN_PASSWORD="zhuxueyi"
```

## 启动方式

### 1. 启动后端

```powershell
cd D:\dy-server-go\server
go run ./cmd/api
```

默认监听：

- `http://127.0.0.1:8080`

### 2. 启动前端开发环境

```powershell
cd D:\dy-server-go\web
pnpm install
pnpm dev
```

默认开发地址：

- `http://127.0.0.1:5173`

Vite 已代理：

- `/api`
- `/auth`
- `/caiji`
- `/app`

### 3. 构建前端

```powershell
cd D:\dy-server-go\web
pnpm build
```

构建产物：

- `D:/dy-server-go/web/dist`

后端会直接发布：

- `/assets/*`
- `/caiji/login`
- `/caiji/caijiSettings`

## 已验证能力

以下能力已经做过真实请求或数据库副作用验证：

- `POST /caiji/login`
- `GET /caiji/cookieList`
- `GET /caiji/keysManageList`
- `GET /caiji/adminSummary`
- `POST /caiji/addCaiji`
- `POST /caiji/deleteCaiji`
- `POST /caiji/allocateCookie`
- `GET /api/getRunningStatus`
- `POST /auth/tokenExpiredAt`
- `POST /api/addRoom`
- `POST /api/fetchRoom`
- `GET /api/fetchOneRoom`
- `POST /api/updateDailyLog`
- `POST /auth/recharge`

## 当前兼容范围

### 已接入旧路径

#### AppController

- `GET /app/version.json`

#### AuthController

- `POST /auth/login`
- `POST /auth/register`
- `POST /auth/unbound`
- `ANY /auth/tokenExpiredAt`
- `POST /auth/recharge`
- `ANY /auth/logout`

#### ApiController

- `GET /api/getRunningStatus`
- `GET /api/serverRooms`
- `POST /api/addRoom`
- `ANY /api/checkToken`
- `ANY /api/fetchCookie`
- `ANY /api/caijiStatus`
- `ANY /api/updateCookie`
- `ANY /api/fetchRoom`
- `ANY /api/fetchOneRoom`
- `ANY /api/consumeRoom`
- `ANY /api/fetchDailyLog`
- `ANY /api/updateDailyLog`
- `ANY /api/redisLoadKey`

#### CaijiController

已接入主要后台页面和核心接口，包括：

- 登录
- 采集器列表
- Cookie 列表
- KEYS 管理
- 用户管理
- 链接管理
- 升级码管理
- 采集器新增/删除
- Cookie 分配

## 当前已知差异和风险

- `key_store` 真实表结构没有 `updated_at`，当前后端已按现库兼容处理，但前端表单仍保留该字段展示语义
- `CaijiController` 还有部分接口只是首版兼容，未逐条完成业务验证
- 管理后台 UI 已可登录和操作一部分核心功能，但还不是旧页面的完整复刻
- `ApiController` 的部分旧逻辑原本依赖 Redis 队列或旧框架生态，目前是以当前仓库可运行优先做兼容实现
- `CaijiServer.php` 未迁移

## 常用检查

### 构建后端

```powershell
cd D:\dy-server-go\server
go build ./...
```

### 构建前端

```powershell
cd D:\dy-server-go\web
pnpm build
```

### 运行本地回归脚本

```powershell
cd D:\dy-server-go
powershell -ExecutionPolicy Bypass -File .\scripts\verify.ps1
```

当前脚本覆盖：

- 后端构建
- 前端构建
- 后端启动
- `caiji` 登录
- `cookieList`
- `keysManageList`
- `adminSummary`
- `addCaiji/deleteCaiji`
- `allocateCookie`
- `getRunningStatus`
- `tokenExpiredAt`
- `addRoom`
- `fetchRoom`
- `fetchOneRoom`
- `updateDailyLog`
- `recharge`

### 查看管理后台

- 登录页：`http://127.0.0.1:8080/caiji/login`
- 后台页：`http://127.0.0.1:8080/caiji/caijiSettings`

## 下一步建议

- 继续逐项验证 `CaijiController` 余下接口
- 把后台页面继续补齐到接近旧页面操作深度
- 清理 `key_store` 前后端字段差异
- 为客户端调用链补更系统的回归脚本
