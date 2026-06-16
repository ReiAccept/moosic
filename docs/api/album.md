## 专辑

### 获取专辑详情

`POST /api/album/info`

获取专辑详细信息及其歌曲列表。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "id": 10
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 专辑 ID |

**响应** `200 OK`

```json
{
    "id": 10,
    "name": "OK Computer",
    "artist_id": 1,
    "artist_name": "Radiohead",
    "year": 1997,
    "genre": "Alternative Rock",
    "song_count": 12,
    "duration_secs": 3204,
    "cover_url": "/api/album/cover?id=10",
    "starred": null,
    "songs": [
        {
            "id": 100,
            "title": "Airbag",
            "artist_name": "Radiohead",
            "album_name": "OK Computer",
            "track_number": 1,
            "disc_number": 1,
            "duration_secs": 283,
            "bit_rate": 320,
            "file_format": "mp3",
            "size_bytes": 11320000,
            "year": 1997,
            "starred": 1781645000000
        }
    ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | i32 | 专辑 ID |
| `name` | string | 专辑名称 |
| `artist_id` | i32 | 艺术家 ID |
| `artist_name` | string | 艺术家名称 |
| `year` | i32\|null | 发行年份 |
| `genre` | string\|null | 流派 |
| `song_count` | i32 | 歌曲数 |
| `duration_secs` | i32 | 总时长（秒） |
| `cover_url` | string | 封面图片 URL |
| `starred` | i64\|null | 收藏时间 |
| `songs` | array | 歌曲列表（按 disc_number, track_number 排序） |
| `songs[].id` | i32 | 歌曲 ID |
| `songs[].title` | string | 歌曲标题 |
| `songs[].artist_name` | string | 艺术家名称 |
| `songs[].album_name` | string | 专辑名称 |
| `songs[].track_number` | i32\|null | 音轨号 |
| `songs[].disc_number` | i32\|null | 碟号 |
| `songs[].duration_secs` | i32 | 时长（秒） |
| `songs[].bit_rate` | i32\|null | 比特率（kbps） |
| `songs[].file_format` | string\|null | 文件格式 |
| `songs[].size_bytes` | i64\|null | 文件大小 |
| `songs[].year` | i32\|null | 年份 |
| `songs[].starred` | i64\|null | 收藏时间 |

---


### 专辑列表视图

`POST /api/album/list`

按类型获取专辑列表（最近添加、最近播放、最常播放、随机等）。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "type": "newest",
    "offset": 0,
    "limit": 20
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `type` | string | 是 | 列表类型：`newest`、`recent`、`frequent`、`random`、`alphabeticalByName`、`alphabeticalByArtist`、`starred`、`byYear`（从 `year_from` 到 `year_to`）、`byGenre` |
| `genre_id` | i32 | 否 | 按流派过滤（仅 `byGenre` 类型需要） |
| `year_from` | i32 | 否 | 起始年份（仅 `byYear` 类型需要） |
| `year_to` | i32 | 否 | 结束年份（仅 `byYear` 类型需要） |
| `offset` | i32 | 否 | 分页偏移，默认 `0` |
| `limit` | i32 | 否 | 每页数量，默认 `20`，最大 `500` |

**响应** `200 OK`

```json
{
    "albums": [
        {
            "id": 10,
            "name": "OK Computer",
            "artist_name": "Radiohead",
            "year": 1997,
            "song_count": 12,
            "duration_secs": 3204,
            "cover_url": "/api/album/cover?id=10",
            "starred": null
        }
    ],
    "total": 156
}
```

---

### 获取专辑封面

`GET /api/album/cover?id={id}&size={size}`

获取专辑封面图片

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