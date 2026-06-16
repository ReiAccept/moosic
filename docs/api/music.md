## 音乐接口

音乐媒体相关接口，包含歌曲信息、流媒体播放、下载、封面和歌词。
---

### 获取歌曲信息

`POST /api/music/info`

获取单首歌曲的完整元数据。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "id": 100
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 歌曲 ID |

**响应** `200 OK`

```json
{
    "id": 100,
    "title": "Airbag",
    "artist_id": 1,
    "artist_name": "Radiohead",
    "album_id": 10,
    "album_name": "OK Computer",
    "genre": "Alternative Rock",
    "track_number": 1,
    "disc_number": 1,
    "duration_secs": 283,
    "bit_rate": 320,
    "size_bytes": 11320000,
    "file_format": "mp3",
    "content_type": "audio/mpeg",
    "year": 1997,
    "created_at": 1781524800000,
    "starred": 1781645000000,
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | i32 | 歌曲 ID |
| `title` | string | 歌曲标题 |
| `artist_id` | i32 | 艺术家 ID |
| `artist_name` | string | 艺术家名称 |
| `album_id` | i32\|null | 专辑 ID |
| `album_name` | string\|null | 专辑名称 |
| `genre` | string\|null | 流派 |
| `track_number` | i32\|null | 音轨号 |
| `disc_number` | i32\|null | 碟号 |
| `duration_secs` | i32 | 时长（秒） |
| `bit_rate` | i32\|null | 比特率（kbps） |
| `size_bytes` | i64\|null | 文件大小（字节） |
| `file_format` | string\|null | 文件格式（如 `mp3`、`flac`） |
| `content_type` | string\|null | MIME 类型（如 `audio/mpeg`） |
| `year` | i32\|null | 年份 |
| `created_at` | i64 | 添加时间 |
| `starred` | i64\|null | 收藏时间，未收藏为 `null` |

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 成功 |
| `404` | 歌曲不存在 |

---

### 流媒体播放

`GET /api/music/stream?id={song_id}&max_bit_rate={kbps}`

流式传输音频文件。响应体为原始音频数据。

**查询参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 歌曲 ID |
| `max_bit_rate` | i32 | 否 | 最大码率限制（kbps）。若歌曲原始码率超过此值，服务端应进行转码或选择较低码率版本。`0`（默认）表示不限制 |

**请求头**

```
Authorization: Bearer <token>
```

**响应** `200 OK`

| 响应头 | 说明 |
|--------|------|
| `Content-Type` | 音频 MIME 类型（如 `audio/mpeg`、`audio/flac`） |
| `Content-Length` | 文件大小 |
| `Accept-Ranges` | `bytes`（支持 Range 请求） |
| `Cache-Control` | `public, max-age=31536000`（静态缓存策略） |

> 支持 HTTP Range 请求（`Range: bytes=0-...`），用于 seek 操作。

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 成功，返回音频流 |
| `206` | 部分内容（Range 请求） |
| `404` | 歌曲不存在或文件丢失 |
| `403` | 当前用户未启用该歌曲对应的音乐库 |
| `416` | Range 请求越界 |

---

### 下载音频文件

`GET /api/music/download?id={song_id}`

下载原始音频文件，Content-Disposition 设置为 attachment。

**查询参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 歌曲 ID |

**请求头**

```
Authorization: Bearer <token>
```

**响应** `200 OK`

| 响应头 | 说明 |
|--------|------|
| `Content-Type` | 音频 MIME 类型 |
| `Content-Length` | 文件大小 |
| `Content-Disposition` | `attachment; filename="Artist - Title.ext"` |

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 成功 |
| `404` | 歌曲不存在 |
| `403` | 无权访问该歌曲所在的音乐库 |

---

### 获取单曲封面图片

`GET /api/music/cover?id={id}&size={size}`

获取歌曲的封面图片

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

### 获取歌词

`GET /api/music/lyrics?id={song_id}`

获取歌曲歌词。优先返回时间同步歌词（LRC），其次返回纯文本歌词。

**查询参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 歌曲 ID |

**请求头**

```
Authorization: Bearer <token>
```

**响应** `200 OK`

```json
{
    "type": "synced",
    "lines": [
        {
            "start_ms": 18000,
            "text": "I am born again"
        },
        {
            "start_ms": 22000,
            "text": "In a fast German car"
        }
    ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `type` | string | `synced`（同步歌词）、`unsynced`（纯文本）或 `none`（无歌词） |
| `lines` | array | 歌词行数组 |
| `lines[].start_ms` | i32\|null | 起始毫秒（仅 synced 类型） |
| `lines[].text` | string | 歌词文本 |

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 成功（`type: "none"` 表示无歌词） |
| `404` | 歌曲不存在 |

---

---

### 随机歌曲

`POST /api/music/rand`

获取随机歌曲列表。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "size": 20,
    "genre_id": null,
    "year_from": null,
    "year_to": null
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `size` | i32 | 否 | 数量，默认 `20`，最大 `500` |
| `genre_id` | i32\|null | 否 | 限定流派 |
| `year_from` | i32\|null | 否 | 起始年份 |
| `year_to` | i32\|null | 否 | 结束年份 |

**响应** `200 OK`

```json
{
    "songs": [
        {
            "id": 501,
            "title": "Paranoid Android",
            "artist_name": "Radiohead",
            "album_name": "OK Computer",
            "track_number": 2,
            "duration_secs": 383,
            "starred": null
        }
    ]
}
```

---

### 正在播放

`POST /api/music/playing`

获取当前正在播放的歌曲（由于可能存在不同设备，所以应该返回列表）。

**请求头**

```
Authorization: Bearer <token>
```

**响应** `200 OK`

```json
{
    "entries": [
        {
            "song_id": 100,
            "title": "Airbag",
            "artist_name": "Radiohead",
            "username": "alice",
            "minutes_ago": 2
        }
    ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `entries[].song_id` | i32 | 歌曲 ID |
| `entries[].title` | string | 歌曲标题 |
| `entries[].artist_name` | string | 艺术家 |
| `entries[].username` | string | 播放用户 |
| `entries[].minutes_ago` | i32 | N 分钟前开始播放 |

---
