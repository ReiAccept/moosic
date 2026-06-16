## 服务器管理接口

> 所有错误响应遵循统一格式，详见 [错误格式](./error.md)

以下接口需要具有 `read_server` 权限

> 公开的健康检查端点 `/api/health` 无需认证，详见 [健康检查](../health.md)。

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
        "memory_usage": 15163392,
        "memory_total": 16777216,
        "cpu_usage": 12.5,
        "uptime_secs": 86400,
        "disk_total": 107374182400,
        "disk_used": 42949672960
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
| `system.memory_total` | usize\|null | 系统总物理内存，单位字节（无法获取时为空） |
| `system.cpu_usage` | f64\|null | 进程 CPU 使用率百分比（无法获取时为空） |
| `system.uptime_secs` | u64 | 进程运行时长，单位秒 |
| `system.disk_total` | u64\|null | 数据分区总磁盘空间，单位字节 |
| `system.disk_used` | u64\|null | 数据分区已用磁盘空间，单位字节 |
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
