## 艺术家接口

### 获取艺术家详情

`POST /api/artist/info`

获取艺术家详细信息及其专辑列表。

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
| `id` | i32 | 是 | 艺术家 ID |

**响应** `200 OK`

```json
{
    "id": 1,
    "name": "Radiohead",
    "sort_name": "Radiohead",
    "album_count": 9,
    "song_count": 112,
    "starred": 1781640000000,
    "albums": [
        {
            "id": 10,
            "name": "OK Computer",
            "year": 1997,
            "song_count": 12,
            "duration_secs": 3204,
            "starred": null
        }
    ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | i32 | 艺术家 ID |
| `name` | string | 艺术家名称 |
| `album_count` | i32 | 专辑总数 |
| `song_count` | i32 | 歌曲总数 |
| `starred` | i64\|null | 收藏时间 |
| `albums` | array | 专辑列表（按年份逆序） |
| `albums[].id` | i32 | 专辑 ID |
| `albums[].name` | string | 专辑名称 |
| `albums[].year` | i32\|null | 发行年份 |
| `albums[].song_count` | i32 | 歌曲数 |
| `albums[].duration_secs` | i32 | 总时长（秒） |
| `albums[].starred` | i64\|null | 收藏时间 |

---

### 获取艺术家列表

`POST /api/artist/list`

按字母索引返回艺术家列表。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "offset": 0,
    "limit": 20
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `offset` | i32 | 否 | 分页偏移，默认 `0` |
| `limit` | i32 | 否 | 每页数量，默认 `20`，最大 `500` |

**响应** `200 OK`

```json
{
    "artists": [
        {
            "id": 1,
            "name": "Radiohead",
            "sort_name": "Radiohead",
            "album_count": 9,
            "song_count": 112,
            "image_url": "/api/artist/cover?id=1",
            "starred": 1781640000000
        }
    ],
    "total": 230
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `artists` | array | 艺术家对象数组（按 `sort_name` 字母序） |
| `artists[].id` | i32 | 艺术家 ID |
| `artists[].name` | string | 艺术家名称 |
| `artists[].album_count` | i32 | 专辑数量 |
| `artists[].song_count` | i32 | 歌曲数量 |
| `artists[].image_url` | string | 艺术家图片 URL |
| `artists[].starred` | i64\|null | 收藏时间，未收藏为 `null` |
| `total` | i32 | 艺术家总数 |

---

### 获取艺术家照片

`GET /api/artist/cover?id={id}&size={size}`

获取艺术家封面照片

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

> 若没有封面，返回默认占位图（`image/svg+xml`）

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 成功 |
| `404` | 指定 ID 的资源不存在 |