## 搜索

### 搜索

`POST /api/search`

按关键词搜索所有符合的艺术家/专辑/歌曲

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "query": "radiohead",
    "offset": 0,
    "limit": 20
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `query` | string | 是 | 搜索关键词 |
| `offset` | i32 | 否 | 分页偏移，默认 `0` |
| `limit` | i32 | 否 | 每类结果上限，默认 `20` |

**响应** `200 OK`

```json
{
    "artists": [
        {
            "id": 1,
            "name": "Radiohead",
            "album_count": 9,
        }
    ],
    "albums": [
        {
            "id": 10,
            "name": "OK Computer",
            "artist_name": "Radiohead",
            "year": 1997,
            "song_count": 12,
        }
    ],
    "songs": [
        {
            "id": 100,
            "title": "Airbag",
            "artist_name": "Radiohead",
            "album_name": "OK Computer",
            "track_number": 1,
            "duration_secs": 283,
        }
    ]
}
```
