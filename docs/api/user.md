## 用户接口

用户自服务接口，用于登录、注销、修改密码和获取个人信息。

---

### 用户登录

`POST /api/user/login`

使用用户名和密码登录，返回会话令牌。

**请求体** `application/json`

```json
{
    "username": "alice",
    "password": "secret123"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `username` | string | 是 | 用户名 |
| `password` | string | 是 | 明文密码，传输层依赖 HTTPS 保护 |

**响应** `200 OK`

```json
{
    "token": "a1b2c3d4...",
    "user": {
        "id": 1,
        "username": "alice",
        "privs": {},
        "email": "alice@example.com",
        "scrobbling_enabled": true,
        "max_bit_rate": 0,
        "created_at": 1781524800000
    }
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `token` | string | 会话令牌，后续请求通过 `Authorization: Bearer <token>` 携带 |
| `user.id` | i32 | 用户 ID |
| `user.username` | string | 用户名 |
| `user.privs` | Object | 权限 |
| `user.email` | string\|null | 邮箱 |
| `user.scrobbling_enabled` | bool | 是否启用 scrobble 记录 |
| `user.max_bit_rate` | i32 | 最大码率限制（kbps），0 表示无限制 |
| `user.created_at` | i64 | 创建时间（13 位 Unix 毫秒时间戳） |

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 登录成功 |
| `401` | 用户名或密码错误 |
| `400` | 请求体格式错误 |

---

### 用户注销

`GET /api/user/logout`

使当前会话令牌失效。

**请求头**

```
Authorization: Bearer <token>
```

**响应** `200 OK`

```json
{
    "message": "Logged out"
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 注销成功 |
| `401` | 未提供有效令牌 |

---

### 修改密码

`POST /api/user/password/edit`

当前用户修改自己的密码，需要提供旧密码验证。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "old_password": "secret123",
    "new_password": "newsecret456"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `old_password` | string | 是 | 当前密码 |
| `new_password` | string | 是 | 新密码（建议最小长度 8） |

**响应** `200 OK`

```json
{
    "message": "Password updated"
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 修改成功 |
| `401` | 旧密码错误或未登录 |
| `400` | 新密码不符合要求 |

---

### 获取个人信息

`GET /api/user/info`

返回当前登录用户的详细信息。

**请求头**

```
Authorization: Bearer <token>
```

**响应** `200 OK`

```json
{
    "id": 1,
    "username": "alice",
    "privs": {},
    "email": "alice@example.com",
    "scrobbling_enabled": true,
    "max_bit_rate": 320,
    "created_at": 1781524800000,
    "updated_at": 1781598000000
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | i32 | 用户 ID |
| `username` | string | 用户名 |
| `privs` | Object | 权限 |
| `email` | string\|null | 邮箱地址 |
| `scrobbling_enabled` | bool | 是否启用播放记录 |
| `max_bit_rate` | i32 | 最大码率限制（kbps），0 = 无限制 |
| `created_at` | i64 | 创建时间（13 位 Unix 毫秒时间戳） |
| `updated_at` | i64 | 最后更新时间（13 位 Unix 毫秒时间戳） |

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 成功 |
| `401` | 未提供有效令牌 |
