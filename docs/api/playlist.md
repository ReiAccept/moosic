## 歌单接口

歌单的创建、编辑、删除和歌曲管理。

---

### 获取歌单列表

`POST /api/playlist/list`

获取当前用户可见的歌单（包含自己创建的和公开的）。

**请求头**

```
Authorization: Bearer <token>
```

**响应** `200 OK`

```json
{
    "playlists": [
        {
            "id": 1,
            "name": "睡前音乐",
            "owner_name": "alice",
            "is_public": false,
            "song_count": 42,
            "duration_secs": 9100,
            "created_at": 1781611200000,
            "updated_at": 1781683200000
        }
    ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `playlists[].id` | i32 | 歌单 ID |
| `playlists[].name` | string | 名称 |
| `playlists[].owner_name` | string | 创建者用户名 |
| `playlists[].is_public` | bool | 是否公开 |
| `playlists[].song_count` | i32 | 歌曲数 |
| `playlists[].duration_secs` | i32 | 总时长（秒） |
| `playlists[].created_at` | i64 | 创建时间 |
| `playlists[].updated_at` | i64 | 最后更新时间 |

---

### 获取歌单详情

`POST /api/playlist/info`

获取歌单及其包含的歌曲列表。

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
| `id` | i32 | 是 | 歌单 ID |

**响应** `200 OK`

```json
{
    "id": 1,
    "name": "睡前音乐",
    "owner_name": "alice",
    "comment": "适合睡前听的轻柔音乐",
    "is_public": false,
    "song_count": 42,
    "duration_secs": 9100,
    "created_at": 1781611200000,
    "songs": [
        {
            "position": 1,
            "song_id": 100,
            "title": "Airbag",
            "artist_name": "Radiohead",
            "album_name": "OK Computer",
            "duration_secs": 283,
            "starred": null
        }
    ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `comment` | string\|null | 备注/描述 |
| `songs` | array | 歌曲列表（按 position 排序） |
| `songs[].position` | i32 | 在列表中的序号（从 1 开始） |

---

### 创建歌单

`POST /api/playlist/create`

创建新歌单，可同时指定初始歌曲列表。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "name": "新歌单",
    "comment": "我的描述",
    "is_public": false,
    "song_ids": [100, 501, 230]
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 是 | 歌单名称 |
| `comment` | string | 否 | 备注 |
| `is_public` | bool | 否 | 是否公开，默认 `false` |
| `song_ids` | array | 否 | 初始歌曲 ID 列表 |

**响应** `201 Created`

```json
{
    "id": 3,
    "name": "新歌单",
    "comment": "我的描述",
    "is_public": false,
    "song_count": 3,
    "duration_secs": 720,
    "created_at": 1781697600000
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `201` | 创建成功 |
| `400` | 名称不能为空 |

---

### 编辑歌单

`POST /api/playlist/update`

更新歌单的名称、备注和公开状态。仅所有者可编辑。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "id": 3,
    "name": "重命名列表",
    "comment": "更新后的描述",
    "is_public": true
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 歌单 ID |
| `name` | string | 否 | 新名称 |
| `comment` | string\|null | 否 | 新备注（传 `null` 清空） |
| `is_public` | bool | 否 | 是否公开 |

> 未提供的字段保持不变。

**响应** `200 OK`

返回更新后的歌单详情（格式同 `info`）。

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 更新成功 |
| `403` | 不是所有者 |
| `404` | 歌单不存在 |

---

### 删除歌单

`POST /api/playlist/del`

删除歌单。仅所有者可删除。

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
| `id` | i32 | 是 | 歌单 ID |

**响应** `200 OK`

```json
{
    "message": "Playlist deleted"
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 删除成功 |
| `403` | 不是所有者 |
| `404` | 歌单不存在 |

---

### 添加歌曲到歌单

`POST /api/playlist/music/add`

向歌单追加歌曲。仅所有者可操作。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "id": 1,
    "song_ids": [350, 420]
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 歌单 ID |
| `song_ids` | array | 是 | 要添加的歌曲 ID 列表（追加到列表末尾） |

**响应** `200 OK`

返回更新后的歌单详情（格式同 `info`）。

---

### 从歌单移除歌曲

`POST /api/playlist/music/remove`

从歌单移除指定序号的歌曲。仅所有者可操作。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "id": 1,
    "positions": [1, 3]
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 歌单 ID |
| `positions` | array | 是 | 要移除的歌曲位置（序号，从 1 开始） |

> 移除后剩余歌曲的 position 会自动重新编号。

**响应** `200 OK`

返回更新后的歌单详情（格式同 `info`）。


### 获取歌单封面图片

`GET /api/playlist/cover?id={id}&size={size}`

获取歌单的封面图片，默认是第一首歌的封面

**查询参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 对应类型的 ID |
| `size` | i32 | 否 | 图片缩放尺寸（像素）。不指定则返回原始尺寸。常用值：`50`、`120`、`300`、`600` |

**请求头**

```
Authorization: Bearer <token>
```

**响应** `200 OK`

| 响应头 | 说明 |
|--------|------|
| `Content-Type` | 图片 MIME 类型（`image/jpeg`、`image/png` 等） |
| `Cache-Control` | `public, max-age=604800`（7 天缓存） |

> 若歌曲/专辑/艺术家没有封面，返回默认占位图（`image/svg+xml`）。

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 成功 |
| `404` | 指定 ID 的资源不存在 |

---