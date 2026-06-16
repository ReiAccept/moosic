## 用户管理接口

以下接口需要具有 `edit_user` 权限（管理员默认拥有）

---

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
| `privs` | bool | Object | 权限 |
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
    "privs": {},
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | i32 | 是 | 目标用户 ID |
| `privs` | Obejct | 是 | 权限 |

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
