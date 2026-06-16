## 标注接口

收藏（Star）、评分（Rating）和播放记录（Scrobble）

---

### 收藏/取消收藏

`POST /api/annotation/star`

切换收藏状态。若已收藏则取消收藏，未收藏则添加收藏。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "type": "song",
    "id": 100
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `type` | string | 是 | 类型：`song`、`album`、`artist` |
| `id` | i32 | 是 | 对应类型的 ID |

**响应** `200 OK`

```json
{
    "starred": true
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `starred` | bool | `true` = 已收藏，`false` = 已取消收藏 |

---

### 设置评分

`POST /api/annotation/rate`

对歌曲或专辑进行 1-5 星评分。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "type": "song",
    "id": 100,
    "rating": 5
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `type` | string | 是 | 类型：`song`、`album` |
| `id` | i32 | 是 | 对应类型的 ID |
| `rating` | i32 | 是 | 评分，1-5 |

**响应** `200 OK`

```json
{
    "rating": 5
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 评分成功 |
| `400` | rating 不在 1-5 范围内 |

---

### Scrobble（记录播放）

`POST /api/annotation/scrobble`

记录歌曲播放。客户端应在开始播放时调用一次（`submission: false`），在播放完成（超过一半时长或 4 分钟）时再调用一次（`submission: true`）。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "song_id": 100,
    "submission": true,
    "played_at": 1781700000000
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `song_id` | i32 | 是 | 歌曲 ID |
| `submission` | bool | 否 | `true` 为正式 scrobble（已听完），`false` 为"正在播放"。默认 `true` |
| `played_at` | i64 | 否 | 播放时间（13 位 Unix 毫秒时间戳），默认当前时间 |

**响应** `200 OK`

```json
{
    "message": "Scrobbled"
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 记录成功 |
| `404` | 歌曲不存在 |

---

### 获取收藏列表

`POST /api/annotation/starred`

获取当前用户的所有收藏，按类型分组。

**请求头**

```
Authorization: Bearer <token>
```

**响应** `200 OK`

```json
{
    "artists": [
        {
            "id": 1,
            "name": "Radiohead",
        }
    ],
    "albums": [
        {
            "id": 10,
            "name": "OK Computer",
            "artist_name": "Radiohead",
        }
    ],
    "songs": [
        {
            "id": 100,
            "title": "Airbag",
            "artist_name": "Radiohead",
            "album_name": "OK Computer",
        }
    ]
}
```

---

### 获取评分的项目

`POST /api/annotation/rated`

获取当前用户评分过的歌曲和专辑。

**请求头**

```
Authorization: Bearer <token>
```

**响应** `200 OK`

```json
{
    "songs": [
        {
            "id": 100,
            "title": "Airbag",
            "artist_name": "Radiohead",
            "rating": 5
        }
    ],
    "albums": [
        {
            "id": 10,
            "name": "OK Computer",
            "artist_name": "Radiohead",
            "rating": 4
        }
    ]
}
```
