## 分享接口

创建和管理公开分享链接，允许非注册用户访问歌曲、专辑或歌单。

---

> 所有错误响应遵循统一格式，详见 [错误格式](./error.md)

### 访问分享内容

`GET /api/share/{token}`

通过分享令牌访问分享内容。**无需认证**，面向外部（非注册）用户。

**路径参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `token` | string | 是 | 分享令牌 |

**响应** `200 OK`

```json
{
    "type": "song",
    "item": {
        "id": 100,
        "title": "Airbag",
        "artist_name": "Radiohead",
        "album_name": "OK Computer",
        "duration_secs": 283
    },
    "share": {
        "description": "听听这首！",
        "created_at": 1781640000000
    }
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `type` | string | 分享类型：`song`、`album`、`playlist` |
| `item` | Object | 分享的实际内容（字段随 type 变化） |
| `share.description` | string\|null | 分享描述 |
| `share.created_at` | i64 | 分享创建时间 |

> 当 `type` 为 `playlist` 时，`item` 包含歌单详情及其歌曲列表。

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 成功 |
| `404` | 分享不存在 |
| `410` | 分享已过期 |

---

### 获取分享列表

`POST /api/share/list`

获取当前用户创建的所有分享链接。

**请求头**

```
Authorization: Bearer <token>
```

**响应** `200 OK`

```json
{
    "shares": [
        {
            "id": 1,
            "type": "song",
            "item_id": 100,
            "title": "Airbag",
            "description": "听听这首！",
            "token": "abc123def",
            "url": "https://music.example.com/share/abc123def",
            "visit_count": 12,
            "last_visited_at": 1781715600000,
            "expires_at": null,
            "created_at": 1781640000000
        }
    ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `shares[].id` | i32 | 分享 ID |
| `shares[].type` | string | 类型：`song`、`album`、`playlist` |
| `shares[].item_id` | i32 | 分享内容的 ID |
| `shares[].title` | string | 分享内容的标题 |
| `shares[].description` | string\|null | 分享描述 |
| `shares[].token` | string | 分享令牌（URL 中的唯一标识） |
| `shares[].url` | string | 完整分享 URL |
| `shares[].visit_count` | i32 | 访问次数 |
| `shares[].last_visited_at` | i64\|null | 最近访问时间 |
| `shares[].expires_at` | i64\|null | 过期时间，`null` 表示永不过期 |
| `shares[].created_at` | i64 | 创建时间 |

---

### 创建分享

`POST /api/share/create`

创建新的分享链接。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "type": "song",
    "item_id": 100,
    "description": "听听这首！",
    "expires_in_days": 7
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `type` | string | 是 | 类型：`song`、`album`、`playlist` |
| `item_id` | i32 | 是 | 分享内容的 ID |
| `description` | string | 否 | 描述文字 |
| `expires_in_days` | i32 | 否 | 过期天数，不填则永不过期 |

**响应** `201 Created`

```json
{
    "id": 2,
    "type": "song",
    "item_id": 100,
    "title": "Airbag",
    "description": "听听这首！",
    "token": "xyz789ghi",
    "url": "https://music.example.com/share/xyz789ghi",
    "expires_at": 1782326400000,
    "created_at": 1781726400000
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `201` | 创建成功 |
| `404` | 分享的内容不存在 |

---

### 更新分享

`POST /api/share/update`

更新分享的描述或过期时间。仅创建者可操作。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "id": 1,
    "description": "更新后的描述",
    "expires_in_days": 30
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 分享 ID |
| `description` | string\|null | 否 | 新描述 |
| `expires_in_days` | i32 | 否 | 新的过期天数（从当前时间算起） |

> 未提供的字段保持不变。

**响应** `200 OK`

返回更新后的分享对象。

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 更新成功 |
| `403` | 不是创建者 |
| `404` | 分享不存在 |

---

### 删除分享

`POST /api/share/delete`

删除分享链接。仅创建者或管理员可操作。

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
| `id` | i32 | 是 | 分享 ID |

**响应** `200 OK`

```json
{
    "message": "Share deleted"
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 删除成功 |
| `403` | 不是创建者且不是管理员 |
| `404` | 分享不存在 |

---

### 批量删除分享

`POST /api/share/delete/batch`

批量删除多个分享链接。仅创建者或管理员可操作。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "ids": [1, 2, 3]
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `ids` | array | 是 | 要删除的分享 ID 列表 |

**响应** `200 OK`

```json
{
    "deleted": 3
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `deleted` | i32 | 成功删除的分享数量 |

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 删除成功（部分或全部） |
| `400` | ids 为空 |
| `403` | 不是创建者且不是管理员 |
