## 用户管理接口

以下接口需要具有 `edit_user` 权限（管理员默认拥有）

---

> 所有错误响应遵循统一格式，详见 [错误格式](./error.md)

### 添加用户

`POST /api/admin/user/add`

创建新用户。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "username": "bob",
    "password": "initial123",
    "email": "bob@example.com",
    "privs": {"edit_user": true},
    "scrobbling_enabled": true,
    "max_bit_rate": 0
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `username` | string | 是 | 用户名，必须唯一 |
| `password` | string | 是 | 初始密码 |
| `email` | string | 否 | 邮箱 |
| `privs` | Object | 否 | 权限，默认 `{}` |
| `scrobbling_enabled` | bool | 否 | 启用播放记录，默认 `true` |
| `max_bit_rate` | i32 | 否 | 最大码率限制（kbps），0 = 无限制，默认 `0` |

**响应** `201 Created`

```json
{
    "id": 2,
    "username": "bob",
    "privs": {"edit_user": true},
    "email": "bob@example.com",
    "scrobbling_enabled": true,
    "max_bit_rate": 0,
    "created_at": 1781697600000
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `201` | 创建成功 |
| `400` | 请求体格式错误 |
| `409` | 用户名已存在 |
| `403` | 无 `edit_user` 权限 |

---

### 删除用户

`POST /api/admin/user/del`

删除指定用户。不能删除自己。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "id": 2
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 要删除的用户 ID |

**响应** `200 OK`

```json
{
    "message": "User deleted"
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 删除成功 |
| `403` | 无 `edit_user` 权限 |
| `404` | 用户不存在 |
| `400` | 不能删除自己 |

---

### 修改用户密码（管理员）

`POST /api/admin/user/password/edit`

管理员直接为用户重置密码，无需旧密码。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "id": 2,
    "new_password": "newpass789"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 目标用户 ID |
| `new_password` | string | 是 | 新密码 |

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
| `403` | 无 `edit_user` 权限 |
| `404` | 用户不存在 |

---

### 获取用户列表

`POST /api/admin/user/list`

获取所有用户的列表。

**请求头**

```
Authorization: Bearer <token>
```

**响应** `200 OK`

```json
{
    "users": [
        {
            "id": 1,
            "username": "alice",
            "privs": {"edit_user": true, "edit_library": true},
            "email": "alice@example.com",
            "scrobbling_enabled": true,
            "max_bit_rate": 0,
            "created_at": 1781524800000
        },
        {
            "id": 2,
            "username": "bob",
            "privs": {},
            "email": null,
            "scrobbling_enabled": true,
            "max_bit_rate": 320,
            "created_at": 1781697600000
        }
    ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `users` | array | 用户对象数组 |
| `users[].id` | i32 | 用户 ID |
| `users[].username` | string | 用户名 |
| `users[].privs` | Object | 权限 |
| `users[].email` | string\|null | 邮箱 |
| `users[].scrobbling_enabled` | bool | 是否启用播放记录 |
| `users[].max_bit_rate` | i32 | 最大码率限制 |
| `users[].created_at` | i64 | 创建时间（13 位 Unix 毫秒时间戳） |

---

### 获取特定用户信息

`POST /api/admin/user/info`

获取某个用户的详细信息。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "id": 2
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 目标用户 ID |

**响应** `200 OK`

```json
{
    "id": 2,
    "username": "bob",
    "privs": {},
    "email": null,
    "scrobbling_enabled": true,
    "max_bit_rate": 320,
    "created_at": 1781697600000,
    "updated_at": 1781697600000
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | i32 | 用户 ID |
| `username` | string | 用户名 |
| `privs` | Object | 权限 |
| `email` | string\|null | 邮箱 |
| `scrobbling_enabled` | bool | 是否启用播放记录 |
| `max_bit_rate` | i32 | 最大码率限制 |
| `created_at` | i64 | 创建时间 |
| `updated_at` | i64 | 最后更新时间 |

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 成功 |
| `403` | 无 `edit_user` 权限 |
| `404` | 用户不存在 |

---

### 编辑用户权限

`POST /api/admin/user/priv/edit`

修改用户的权限和设置。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "id": 2,
    "privs": {}
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 目标用户 ID |
| `privs` | Object | 是 | 权限 |

**响应** `200 OK`

```json
{
    "id": 2,
    "username": "bob",
    "privs": {},
    "updated_at": 1781715600000
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 修改成功 |
| `403` | 无 `edit_user` 权限 |
| `404` | 用户不存在 |

---

### 编辑用户信息

`POST /api/admin/user/edit`

修改用户的基本设置（邮箱、播放记录开关、最大码率限制）。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "id": 2,
    "email": "new@example.com",
    "scrobbling_enabled": false,
    "max_bit_rate": 320
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 目标用户 ID |
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
| `403` | 无 `edit_user` 权限 |
| `404` | 用户不存在 |

---

### 启用用户

`POST /api/admin/user/enable`

启用被禁用的用户。用户被禁用后无法登录，但数据保留。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "id": 2
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 目标用户 ID |

**响应** `200 OK`

```json
{
    "message": "User enabled"
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 启用成功 |
| `403` | 无 `edit_user` 权限 |
| `404` | 用户不存在 |

---

### 禁用用户

`POST /api/admin/user/disable`

禁用用户登录。不能禁用自己。

**请求头**

```
Authorization: Bearer <token>
```

**请求体** `application/json`

```json
{
    "id": 2
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 目标用户 ID |

**响应** `200 OK`

```json
{
    "message": "User disabled"
}
```

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 禁用成功 |
| `400` | 不能禁用自己 |
| `403` | 无 `edit_user` 权限 |
| `404` | 用户不存在 |
