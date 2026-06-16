## 音乐库管理接口

以下接口需要具有 `edit_library` 权限（管理员默认拥有）。

---

### 添加音乐库路径

`POST /api/admin/library/add`

注册新的音乐库扫描路径, 注册后触发全库扫描

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "name": "主音乐库",
    "path": "/data/music"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 是 | 显示名称 |
| `path` | string | 是 | 文件系统绝对路径 |

**响应** `201 Created`

```json
{
    "id": 3,
    "name": "主音乐库",
    "path": "/data/music",
    "is_enabled": true,
    "created_at": 1781697600000
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | i32 | 音乐库 ID |
| `name` | string | 显示名称 |
| `path` | string | 文件系统路径 |
| `is_enabled` | bool | 初始启用状态，默认 `true` |
| `created_at` | i64 | 创建时间 |

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `201` | 创建成功 |
| `400` | 请求体格式错误 |
| `403` | 无 `edit_library` 权限 |
| `409` | 路径已存在 |

---

### 删除音乐库路径

`POST /api/admin/library/del`

移除音乐库扫描路径及其关联的艺术家、专辑、歌曲数据。

> 此操作会级联删除该库下所有扫描到的音乐数据，不可逆。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "id": 3
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 音乐库 ID |

**响应** `200 OK`

```json
{
    "message": "Library deleted",
    "songs_removed": 1523,
    "albums_removed": 156,
    "artists_removed": 78
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `songs_removed` | i32 | 删除的歌曲数 |
| `albums_removed` | i32 | 删除的专辑数 |
| `artists_removed` | i32 | 删除的艺术家数 |

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 删除成功 |
| `403` | 无 `edit_library` 权限 |
| `404` | 音乐库不存在 |

---

### 启动音乐库扫描

`POST /api/admin/library/scan`

重扫描传递的多个音乐库。扫描为异步任务，完成后更新数据库。


### 启动全库扫描

`POST /api/admin/library/scan/all`

触发对所有音乐库的完整扫描。扫描为异步任务，完成后更新数据库。

**请求头**

```
Authorization: Bearer <token>
```

**响应** `200 OK`

```json
{
    "message": "Full scan started",
    "scan_id": "scan_def456"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `scan_id` | string | 扫描任务 ID |

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 扫描已启动 |
| `403` | 无 `edit_library` 权限 |
| `409` | 已有扫描任务在执行中 |

---

### 查询扫描状态

`POST /api/admin/library/scan/status`

查询当前或最近一次扫描的进度。

**请求头**

```
Authorization: Bearer <token>
```

**响应** `200 OK`

```json
{
    "scan_id": "scan_def456",
    "status": "scanning",
    "files_scanned": 540,
    "files_total": 1523,
    "started_at": 1781700000000
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `scan_id` | string | 扫描任务 ID |
| `status` | string | 状态：`idle`、`scanning`、`completed`、`failed` |
| `files_scanned` | i32 | 已扫描文件数 |
| `files_total` | i32 | 总文件数（估算） |
| `started_at` | i64\|null | 扫描开始时间 |
| `error` | string\|null | 错误信息（仅 `failed` 状态） |

---

### 启用音乐库文件文件监听

将某个音乐库加入 notify 变更监听列表，若音乐库此时处于被扫描状态则不可用，需要等待扫描完成

`POST /api/admin/library/notify/enable`


### 关闭音乐库文件文件监听

`POST /api/admin/library/notify/disable`