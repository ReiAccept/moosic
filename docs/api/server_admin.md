## 服务器管理接口

### 获取服务器状态

GET /api/admin/server/status

返回当前服务器概览，包含系统资源、数据库与缓存的启用状态、版本号和服务监听地址。

**缓存** — 响应缓存 10 秒，缓存期间跳过数据库探测和内存采集。

**请求参数** — 无。

**响应格式** `application/json`

```json
{
    "version": "0.1.0",
    "system": {
        "memory_usage": 15163392
    },
    "database": {
        "backend": "sqlite",
        "connected": true
    },
    "cache": {
        "backend": "memory"
    },
    "server": {
        "host": "0.0.0.0",
        "port": 3000
    }
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `version` | string | 应用版本号（来自 Cargo.toml） |
| `system.memory_usage` | usize | 进程物理内存占用，单位字节 |
| `database.backend` | string | 数据库后端，例如 `"sqlite"` |
| `database.connected` | bool | 数据库连接是否正常（以 `SELECT 1` 探测） |
| `cache.backend` | string | 缓存后端 — `"memory"`（DashMap）或 `"redis"` |
| `server.host` | string | 服务监听地址 |
| `server.port` | u16 | 服务监听端口 |

**可能的错误**

| 状态码 | 含义 |
|--------|------|
| `200` | 成功返回服务器状态 |
| `500` | 无法获取系统内存信息 |
