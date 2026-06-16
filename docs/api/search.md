## 搜索

> 所有错误响应遵循统一格式，详见 [错误格式](./error.md)

---

### 搜索

`POST /api/search`

按关键词搜索所有符合的艺术家/专辑/歌曲。支持按类型筛选和高级过滤。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "query": "radiohead",
    "type": "all",
    "year_from": null,
    "year_to": null,
    "offset": 0,
    "limit": 20
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `query` | string | 是 | 搜索关键词 |
| `type` | string | 否 | 限定搜索类别：`all`（默认，全部）、`artist`、`album`、`song` |
| `year_from` | i32 | 否 | 起始年份 |
| `year_to` | i32 | 否 | 结束年份 |
| `offset` | i32 | 否 | 分页偏移，默认 `0` |
| `limit` | i32 | 否 | 每类结果的返回上限，默认 `20`，最大 `100`。例如 limit=20 时，艺术家、专辑、歌曲各最多返回 20 条 |

**响应** `200 OK`

```json
{
    "artists": [
        {
            "id": 1,
            "name": "Radiohead",
            "album_count": 9
        }
    ],
    "albums": [
        {
            "id": 10,
            "name": "OK Computer",
            "artist_name": "Radiohead",
            "year": 1997,
            "song_count": 12
        }
    ],
    "songs": [
        {
            "id": 100,
            "title": "Airbag",
            "artist_name": "Radiohead",
            "album_name": "OK Computer",
            "track_number": 1,
            "duration_secs": 283
        }
    ],
    "artist_total": 1,
    "album_total": 1,
    "song_total": 1
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `artist_total` | i32 | 匹配的艺术家总数 |
| `album_total` | i32 | 匹配的专辑总数 |
| `song_total` | i32 | 匹配的歌曲总数 |

---

### 搜索建议

`POST /api/search/suggest`

轻量级前缀匹配，用于搜索框自动补全。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "query": "rad",
    "limit": 5
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `query` | string | 是 | 搜索前缀（至少 1 个字符） |
| `limit` | i32 | 否 | 返回条数上限，默认 `5`，最大 `20` |

**响应** `200 OK`

```json
{
    "suggestions": [
        {
            "type": "artist",
            "id": 1,
            "text": "Radiohead"
        },
        {
            "type": "album",
            "id": 10,
            "text": "Radiohead — OK Computer"
        },
        {
            "type": "song",
            "id": 100,
            "text": "Radiohead — Airbag"
        }
    ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `suggestions[].type` | string | 类型：`artist`、`album`、`song` |
| `suggestions[].id` | i32 | 匹配项的 ID |
| `suggestions[].text` | string | 显示文本 |
