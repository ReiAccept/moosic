## 书签接口

书签用于记录歌曲的播放位置，方便用户从上次位置继续播放。

---

> 所有错误响应遵循统一格式，详见 [错误格式](./error.md)

### 获取书签列表

`POST /api/bookmark/list`

获取当前用户的所有书签。由于用户可能有不同设备，所以可能存在多个书签。

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
    "bookmarks": [
        {
            "id": 1,
            "song_id": 100,
            "title": "Airbag",
            "artist_name": "Radiohead",
            "position_ms": 45000,
            "device_id": "phone",
            "updated_at": 1781700000000
        }
    ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `bookmarks[].id` | i32 | 书签 ID |
| `bookmarks[].song_id` | i32 | 歌曲 ID |
| `bookmarks[].title` | string | 歌曲标题 |
| `bookmarks[].artist_name` | string | 艺术家名称 |
| `bookmarks[].position_ms` | i32 | 播放位置（毫秒） |
| `bookmarks[].device_id` | string\|null | 设备标识（多设备场景下区分不同设备的书签） |
| `bookmarks[].updated_at` | i64 | 最后更新时间 |

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 成功 |
| `401` | 未提供有效令牌 |

---

### 获取特定歌曲书签

`POST /api/bookmark/get`

获取当前用户在特定歌曲上的书签（按设备区分）。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "song_id": 100,
    "device_id": "phone"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `song_id` | i32 | 是 | 歌曲 ID |
| `device_id` | string | 否 | 设备标识。若省略，返回默认/主设备的书签 |

**响应** `200 OK`

```json
{
    "id": 1,
    "song_id": 100,
    "title": "Airbag",
    "artist_name": "Radiohead",
    "position_ms": 45000,
    "device_id": "phone",
    "updated_at": 1781700000000
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 成功 |
| `401` | 未提供有效令牌 |
| `404` | 该歌曲+设备上没有书签 |

---

### 创建/更新书签

`POST /api/bookmark/create`

创建书签。若同一用户在同一设备上已对该歌曲创建过书签，则更新播放位置。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "song_id": 100,
    "position_ms": 45000,
    "device_id": "phone"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `song_id` | i32 | 是 | 歌曲 ID |
| `position_ms` | i32 | 是 | 播放位置（毫秒） |
| `device_id` | string | 否 | 设备标识。同一用户在同一设备上对同一歌曲只能有一个书签 |

**响应** `200 OK`（创建或更新）

```json
{
    "id": 2,
    "song_id": 100,
    "position_ms": 45000,
    "device_id": "phone",
    "updated_at": 1781700000000
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 创建/更新成功 |
| `400` | `position_ms` 超出歌曲时长 |
| `401` | 未提供有效令牌 |
| `404` | 歌曲不存在 |

---

### 删除书签

`POST /api/bookmark/delete`

删除指定的书签。仅书签所有者可删除。

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
| `id` | i32 | 是 | 书签 ID |

**响应** `200 OK`

```json
{
    "message": "Bookmark deleted"
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 删除成功 |
| `401` | 未提供有效令牌 |
| `403` | 不是书签所有者 |
| `404` | 书签不存在 |
