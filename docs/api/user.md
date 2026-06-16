## 用户接口

用户自服务接口，用于登录、注销、修改密码和获取个人信息。

---

> 所有错误响应遵循统一格式，详见 [错误格式](./error.md)

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

---

### 编辑个人信息

`POST /api/user/edit`

当前用户修改自己的个人设置（邮箱、播放记录、码率限制）。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "email": "new@example.com",
    "scrobbling_enabled": false,
    "max_bit_rate": 320
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `email` | string\|null | 否 | 新邮箱地址 |
| `scrobbling_enabled` | bool | 否 | 是否启用播放记录 |
| `max_bit_rate` | i32 | 否 | 最大码率限制（kbps），0 = 无限制 |

> 未提供的字段保持不变。

**响应** `200 OK`

返回更新后的用户对象（格式同 `info`）。

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 修改成功 |
| `400` | 请求体格式错误 |
| `401` | 未提供有效令牌 |

---

### 刷新令牌

`POST /api/user/token/refresh`

使用当前有效令牌换取新令牌，延长会话有效期。

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
    "token": "new_token_string..."
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 刷新成功 |
| `401` | 当前令牌无效或已过期 |

---

### 管理活跃会话

`POST /api/user/sessions`

查看当前用户的所有活跃会话。

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
    "sessions": [
        {
            "id": "sess_abc123",
            "created_at": 1781700000000,
            "last_used_at": 1781700500000,
            "device_info": "Chrome on Linux"
        }
    ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `sessions[].id` | string | 会话 ID |
| `sessions[].created_at` | i64 | 会话创建时间 |
| `sessions[].last_used_at` | i64 | 最近使用时间 |
| `sessions[].device_info` | string\|null | 设备/客户端信息 |

---

### 撤销会话

`POST /api/user/session/revoke`

撤销指定的活跃会话（在其他设备上登出）。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "session_id": "sess_abc123"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `session_id` | string | 是 | 要撤销的会话 ID |

**响应** `200 OK`

```json
{
    "message": "Session revoked"
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 撤销成功 |
| `400` | 不能撤销当前会话 |
| `404` | 会话不存在 |

---

### 注销账号

`POST /api/user/delete`

用户自行注销账号。需要密码确认。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "password": "current_password"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `password` | string | 是 | 当前密码 |

**响应** `200 OK`

```json
{
    "message": "Account deleted"
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 注销成功 |
| `401` | 密码错误 |
| `400` | 存在关联数据无法删除（如拥有的歌单） |

---

### 请求密码重置

`POST /api/user/password/reset/request`

向注册邮箱发送密码重置验证码。

**请求体** `application/json`

```json
{
    "email": "alice@example.com"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `email` | string | 是 | 注册邮箱地址 |

**响应** `200 OK`

```json
{
    "message": "If the email is registered, a reset code has been sent"
}
```

> 无论邮箱是否已注册，始终返回 200，防止邮箱枚举攻击。

---

### 确认密码重置

`POST /api/user/password/reset/confirm`

使用验证码完成密码重置。

**请求体** `application/json`

```json
{
    "code": "123456",
    "new_password": "newpass456"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `code` | string | 是 | 邮箱收到的验证码 |
| `new_password` | string | 是 | 新密码（最小长度 8） |

**响应** `200 OK`

```json
{
    "message": "Password reset successful"
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 重置成功 |
| `400` | 验证码无效/已过期，或密码不符合要求 |
