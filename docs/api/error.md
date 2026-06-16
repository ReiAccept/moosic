## 错误响应格式

所有 4xx/5xx 响应遵循统一的 JSON 结构。

---

### 响应体格式

```json
{
    "error": {
        "code": "not_found",
        "message": "Human-readable error description"
    }
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `error.code` | string | 机器可读错误码，用于客户端分支判断 |
| `error.message` | string | 人类可读的错误描述 |

> 部分接口可能在 `error` 对象中包含 `details` 字段（JSON Object），用于携带额外的上下文信息（如字段校验失败时指出具体字段）。

---

### 标准错误码

| HTTP 状态码 | `error.code` | 含义 |
|-------------|-------------|------|
| `400` | `validation_error` | 请求参数不合法 |
| `401` | `unauthorized` | 缺少有效令牌或令牌已过期 |
| `403` | `forbidden` | 无权限执行此操作 |
| `404` | `not_found` | 请求的资源不存在 |
| `409` | `conflict` | 资源冲突（如名称已存在、扫描已在进行中） |
| `410` | `gone` | 资源已过期（如分享链接过期） |
| `416` | `range_not_satisfiable` | Range 请求越界（流媒体） |
| `429` | `rate_limited` | 请求频率超限 |
| `500` | `internal_error` | 服务端内部错误 |

---

### 示例

**400 — 参数校验失败**

```json
{
    "error": {
        "code": "validation_error",
        "message": "评分必须在 1-5 之间",
        "details": {
            "field": "rating",
            "value": 10
        }
    }
}
```

**404 — 资源不存在**

```json
{
    "error": {
        "code": "not_found",
        "message": "歌曲不存在"
    }
}
```
