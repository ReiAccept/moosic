## 音乐库接口

音乐库浏览接口，用于获取音乐库、艺术家、专辑、歌曲列表和搜索。

---

> 所有错误响应遵循统一格式，详见 [错误格式](./error.md)

### 获取音乐库列表

`POST /api/library/list`

返回当前用户可访问的音乐库（文件夹）列表。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{}
```

**响应** `200 OK`

```json
{
    "libraries": [
        {
            "id": 1,
            "name": "我的音乐",
            "path": "/data/music",
            "is_enabled": true,
            "song_count": 1523,
            "created_at": 1781524800000
        },
        {
            "id": 2,
            "name": "无损收藏",
            "path": "/data/flac",
            "is_enabled": false,
            "song_count": 340,
            "created_at": 1781598000000
        }
    ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `libraries` | array | 音乐库对象数组 |
| `libraries[].id` | i32 | 音乐库 ID |
| `libraries[].name` | string | 显示名称 |
| `libraries[].path` | string | 文件系统路径 |
| `libraries[].is_enabled` | bool | 当前用户是否启用此库（此为每位用户的个人设置，不同于管理员接口中的全局 `is_enabled`） |
| `libraries[].song_count` | i32 | 歌曲数量 |
| `libraries[].created_at` | i64 | 创建时间 |

---

### 启用音乐库

`POST /api/library/enable`

为当前用户启用指定音乐库。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "id": 2
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 音乐库 ID |

**响应** `200 OK`

```json
{
    "message": "Library enabled"
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 启用成功 |
| `404` | 音乐库不存在 |

---

### 禁用音乐库

`POST /api/library/disable`

为当前用户禁用指定音乐库。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "id": 1
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 音乐库 ID |

**响应** `200 OK`

```json
{
    "message": "Library disabled"
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 禁用成功 |
| `404` | 音乐库不存在 |

---

### 重新扫描音乐库

`POST /api/library/rescan`

触发对指定音乐库的重新扫描（异步任务）。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "id": 1
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 音乐库 ID |

**响应** `200 OK`

```json
{
    "message": "Scan started",
    "scan_id": "scan_abc123"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `scan_id` | string | 扫描任务 ID，可用于查询进度 |

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 扫描已启动 |
| `404` | 音乐库不存在 |
| `409` | 该库已有扫描任务在执行中 |

---
