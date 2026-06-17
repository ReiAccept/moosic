# 概览

## 概述

Moosic 是一款自托管音乐服务器，使用 Rust 语言构建。

---

## 分层架构

```mermaid
graph TB
    subgraph Clients["客户端"]
        WEB["Web UI / Mobile App"]
        API_C["第三方 API 客户端"]
        LB["负载均衡器 / 反向代理"]
    end

    subgraph Transport["传输层"]
        ROUTER["Axum Router<br/>路由分发"]
        AUTH_MW["Auth Middleware<br/>Token 验证 + 权限检查"]
        CORS["CORS / 日志 Middleware"]
    end

    subgraph Handlers["处理器层 (Handlers)"]
        H_ROOT["root"]
        H_HEALTH["health"]
        H_USER["user"]
        H_MUSIC["music"]
        H_ALBUM["album"]
        H_ARTIST["artist"]
        H_PLAYLIST["playlist"]
        H_SEARCH["search"]
        H_SHARE["share"]
        H_BOOKMARK["bookmark"]
        H_ANNOTATION["annotation"]
        H_LIBRARY["library"]
        H_ADMIN["admin/*"]
    end

    subgraph Services["服务层 (Services)"]
        S_AUTH["auth<br/>认证 + 会话管理"]
        S_USER["user<br/>用户管理"]
        S_LIBRARY["library<br/>音乐库管理"]
        S_SCANNER["scanner<br/>文件扫描 + 元数据提取"]
        S_WATCHER["watcher<br/>文件变更监听"]
        S_MEDIA["media<br/>流媒体 + 转码"]
        S_METADATA["metadata<br/>音频标签解析"]
        S_COVER["cover<br/>封面提取 + 缩放"]
        S_PLAYLIST["playlist<br/>歌单管理"]
        S_SEARCH["search<br/>全文搜索"]
        S_ANNOTATION["annotation<br/>收藏/评分/播放记录"]
        S_SHARE["share<br/>分享管理"]
    end

    subgraph DataAccess["数据访问层"]
        ORM["SeaORM Entities"]
        MIGRATION["SeaORM Migration"]
        CACHE["Cache Backend<br/>Memory / Redis"]
    end

    subgraph Storage["存储层"]
        SQLITE[("SQLite<br/>moosic.db")]
        REDIS[("Redis<br/>(可选)")]
        FS[("文件系统<br/>音乐文件 + 封面缓存")]
    end

    subgraph Background["后台任务"]
        SCAN_TASK["扫描任务 (tokio::spawn)"]
        WATCH_TASK["文件监听 (notify)"]
        CLEANUP["过期会话/分享清理"]
    end

    WEB --> LB
    API_C --> LB
    LB --> ROUTER
    ROUTER --> AUTH_MW
    AUTH_MW --> CORS
    CORS --> Handlers
    Handlers --> Services
    Services --> ORM
    Services --> CACHE
    ORM --> SQLITE
    CACHE --> REDIS
    CACHE --> |"Memory (DashMap)"| CACHE
    S_MEDIA --> FS
    S_SCANNER --> FS
    S_WATCHER --> FS
    SCAN_TASK --> S_SCANNER
    WATCH_TASK --> S_WATCHER
    CLEANUP --> ORM
```

---

## 模块结构

```
moosic/
├── src/
│   ├── main.rs                     # 入口：初始化 tracing、config、db、cache、router
│   ├── config.rs                   # 配置加载（JSON → Config struct）
│   ├── state.rs                    # AppState（共享状态）
│   ├── router.rs                   # 路由注册 + State 注入
│   │
│   ├── middleware/
│   │   ├── mod.rs
│   │   └── auth.rs                 # Token 验证中间件 + 权限检查
│   │
│   ├── entities/                   # SeaORM 实体（数据库表映射）
│   │   ├── mod.rs                  # 注册所有实体
│   │   ├── prelude.rs              # 便捷 re-export
│   │   ├── users.rs
│   │   ├── libraries.rs
│   │   ├── user_libraries.rs
│   │   ├── artists.rs
│   │   ├── albums.rs
│   │   ├── songs.rs
│   │   ├── playlists.rs
│   │   ├── playlist_songs.rs
│   │   ├── stars.rs
│   │   ├── ratings.rs
│   │   ├── scrobbles.rs
│   │   ├── bookmarks.rs
│   │   ├── shares.rs
│   │   ├── sessions.rs
│   │   ├── scan_tasks.rs
│   │   ├── lyrics.rs
│   │   └── cover_art.rs
│   │
│   ├── handlers/                   # HTTP 请求处理器（Controller 层）
│   │   ├── mod.rs                  # 模块声明 + 公共 re-export
│   │   ├── root.rs                 # GET /
│   │   ├── health.rs              # GET /api/health
│   │   ├── user.rs                # /api/user/*
│   │   ├── music.rs               # /api/music/*
│   │   ├── album.rs               # /api/album/*
│   │   ├── artist.rs              # /api/artist/*
│   │   ├── playlist.rs            # /api/playlist/*
│   │   ├── search.rs              # /api/search/*
│   │   ├── share.rs               # /api/share/*
│   │   ├── bookmark.rs            # /api/bookmark/*
│   │   ├── annotation.rs          # /api/annotation/*
│   │   ├── library.rs             # /api/library/*
│   │   └── admin/
│   │       ├── mod.rs
│   │       ├── user.rs            # /api/admin/user/*
│   │       ├── library.rs         # /api/admin/library/*
│   │       └── server.rs          # /api/admin/server/status
│   │
│   ├── services/                   # 业务逻辑层
│   │   ├── mod.rs
│   │   ├── auth.rs                # 登录/登出/token 管理/权限
│   │   ├── user.rs                # 用户 CRUD + 自服务
│   │   ├── library.rs             # 音乐库 CRUD + 配置
│   │   ├── scanner.rs             # 文件系统扫描 + 变更检测
│   │   ├── watcher.rs             # notify 文件监听服务, 用于监听所有音乐库目录下的文件变更, 若有音乐文件发生变更则进行更新
│   │   ├── media.rs               # 流媒体传输 + Range 支持
│   │   ├── transcoding.rs         # ffmpeg 转码子进程管理
│   │   ├── metadata.rs            # 音频元数据读取（ID3/Vorbis/FLAC）
│   │   ├── cover.rs               # 封面提取 + 缩放缓存
│   │   ├── playlist.rs            # 歌单 CRUD + 排序
│   │   ├── search.rs              # 全文搜索 + 建议
│   │   ├── annotation.rs          # 收藏/评分/scrobble
│   │   └── share.rs               # 分享链接管理
│   │
│   ├── db/
│   │   ├── mod.rs                  # 数据库连接 + 迁移执行
│   │   └── sqlite.rs               # SQLite 连接实现
│   │
│   └── cache/
│       ├── mod.rs                  # CacheBackend 枚举
│       ├── memory.rs               # DashMap 内存缓存
│       └── redis.rs                # Redis 缓存
│
├── migration/                      # SeaORM 迁移 crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                  # Migrator 入口
│       ├── main.rs                 # 独立 CLI 运行迁移
│       └── m*.rs                   # 各版本迁移文件
│
├── config.json                     # 运行时配置
├── Cargo.toml                      # Workspace 根 + 主 crate 依赖
└── docs/
    ├── api/                        # API 文档
    ├── db/                         # 数据库设计文档
    └── design/                     # 架构设计文档
```

---

## 核心流程

### 1. 请求处理流程

```mermaid
sequenceDiagram
    participant C as Client
    participant R as Router
    participant AM as Auth Middleware
    participant H as Handler
    participant S as Service
    participant DB as SeaORM / SQLite
    participant CA as Cache

    C->>R: HTTP Request
    R->>AM: 路由匹配 → 进入中间件栈
    alt 需要认证
        AM->>AM: 提取 Authorization 头
        AM->>DB: 查询 sessions (token)
        alt token 无效/过期
            AM-->>C: 401 Unauthorized
        else token 有效
            AM->>AM: 注入用户信息到 Request Extensions
            AM->>H: 传递给 Handler
        end
    else 公开端点 (/api/health, /api/share/{token})
        AM->>H: 跳过认证
    end

    H->>H: 解析请求参数 (Query/Path/Json)
    H->>S: 调用业务逻辑
    S->>CA: 尝试读缓存
    alt 缓存命中
        CA-->>S: 返回缓存数据
    else 缓存未命中
        S->>DB: 查询/写入数据库
        DB-->>S: 返回结果
        S->>CA: 写入缓存
    end
    S-->>H: 返回业务结果
    H-->>C: JSON 响应
```

### 2. 音乐库扫描流程

```mermaid
sequenceDiagram
    participant Admin as Admin API
    participant LS as LibraryService
    participant Scan as Scanner
    participant FS as 文件系统
    participant Meta as MetadataService
    participant DB as SQLite

    Admin->>LS: POST /api/admin/library/scan
    LS->>LS: 检查是否有扫描正在执行
    LS->>Scan: tokio::spawn 异步扫描任务
    LS-->>Admin: 200 { scan_id, "Scan started" }

    Note over Scan: 后台异步执行
    Scan->>DB: INSERT scan_tasks (status=scanning)
    Scan->>FS: 遍历 library.path 下所有音频文件
    loop 每个音频文件
        Scan->>Meta: 读取元数据 (ID3/Vorbis/FLAC)
        Meta-->>Scan: { title, artist, album, track, ... }
        Scan->>DB: UPSERT artist / album / song
        Scan->>DB: UPDATE scan_tasks (files_scanned++)
    end
    Scan->>DB: UPDATE scan_tasks (status=completed)

    Note over Scan: 清理孤记录（文件已删除的歌曲）
    Scan->>DB: DELETE songs WHERE file_path NOT IN (当前文件列表)
```

**增量扫描**: 比较文件的 `mtime` 与数据库中 `songs.updated_at`，仅重新扫描变更过的文件。

**扫描策略**: 同 Navidrome，采用 Quick Scan（基于 mtime + size）→ Full Scan（重新读取标签）的两阶段策略。Quick Scan 快速发现新增/删除文件，Full Scan 在标签变更时触发。

### 3. 流媒体播放流程

```mermaid
sequenceDiagram
    participant C as Client
    participant H as MusicHandler
    participant MS as MediaService
    participant TC as TranscodingService
    participant FS as 文件系统
    participant DB as SQLite

    C->>H: GET /api/music/stream?id=100&max_bit_rate=320
    H->>DB: 查询 songs WHERE id=100
    DB-->>H: { file_path, bit_rate, content_type, ... }

    alt max_bit_rate > 0 AND song.bit_rate > max_bit_rate
        H->>TC: 启动 ffmpeg 转码
        TC->>FS: 读取原始文件
        TC->>TC: ffmpeg 转码到目标码率
        TC-->>C: 流式输出转码后的音频 (chunked)
    else 无需转码
        H->>MS: 流式传输原始文件
        MS->>FS: 打开文件 + Range 解析
        MS-->>C: 206 Partial Content / 200 OK (chunked)
    end

    H->>DB: INSERT scrobbles (submission=false, "now playing")
```

**Range 请求支持**: 通过解析 `Range: bytes=start-end` 请求头实现 seek 操作。使用 `tokio::fs::File` 的 `seek` + `take` 读取指定范围的字节。

**转码缓存**: 转码后的数据可按 key `transcode:{song_id}:{bit_rate}` 缓存到文件系统（`/var/cache/moosic/transcodes/`）或 Redis，避免重复转码。

### 4. 认证流程

```mermaid
sequenceDiagram
    participant C as Client
    participant H as UserHandler
    participant AS as AuthService
    participant DB as SQLite

    Note over C,DB: === 登录 ===
    C->>H: POST /api/user/login { username, password }
    H->>AS: login(username, password)
    AS->>DB: SELECT users WHERE username = ?
    DB-->>AS: { id, password_hash, is_enabled, ... }
    AS->>AS: argon2::verify(password, password_hash)
    alt 密码错误或用户被禁用
        AS-->>H: 401 Unauthorized
        H-->>C: 401
    else 验证通过
        AS->>AS: 生成随机 token (32 bytes hex)
        AS->>DB: INSERT sessions (token, user_id, expires_at, ...)
        AS-->>H: { token, user }
        H-->>C: 200 { token, user }
    end

    Note over C,DB: === 后续请求 ===
    C->>H: GET /api/music/... (Authorization: Bearer <token>)
    H->>AS: validate_token(token)
    AS->>DB: SELECT sessions WHERE token = ?
    AS->>DB: UPDATE sessions SET last_used_at = NOW
    alt token 过期
        AS->>DB: DELETE sessions WHERE token = ? (清理)
        AS-->>H: 401
    else token 有效
        AS-->>H: User { id, username, privs, ... }
    end
```

---

## 后台任务

所有后台任务通过 `tokio::spawn` 在应用启动时创建，注册到 `AppState` 中统一管理生命周期。

```
┌─────────────────────────────────────────────────────────┐
│                      AppState                            │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │ ScannerHandle│  │ WatcherHandle│  │ CleanupHandle │  │
│  │ (JoinHandle) │  │ (JoinHandle) │  │ (JoinHandle)  │  │
│  └──────────────┘  └──────────────┘  └───────────────┘  │
│  ┌──────────────────────────────────────────────────┐   │
│  │        scan_task: Option<ScanTask>                │   │
│  │        (当前扫描状态，Arc<RwLock<...>>)            │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### 定时清理任务

```mermaid
graph LR
    CLEANUP["Cleanup Task<br/>(tokio::time::interval)"] --> A["DELETE expired sessions"]
    CLEANUP --> B["DELETE expired shares"]
    CLEANUP --> D["清理转码缓存文件"]
    CLEANUP --> E["清理过期封面缓存"]
```

---

## 数据流图

```mermaid
flowchart LR
    subgraph Input["数据输入"]
        FS_SCAN["文件系统扫描"]
        FS_WATCH["文件变更事件"]
        API["REST API 请求"]
    end

    subgraph Processing["处理"]
        SCANNER["Scanner<br/>元数据提取"]
        WATCHER["Watcher<br/>增量更新"]
        HANDLERS["Handlers<br/>请求处理"]
    end

    subgraph Storage["存储"]
        DB[("SQLite")]
        CACHE_LAYER["Cache<br/>Memory / Redis"]
        COVER_FS[("封面缓存<br/>文件系统")]
        TRANSCODE_FS[("转码缓存<br/>文件系统")]
    end

    subgraph Output["输出"]
        JSON["JSON 响应"]
        STREAM["音频流"]
        COVER["封面图片"]
    end

    FS_SCAN --> SCANNER
    FS_WATCH --> WATCHER
    API --> HANDLERS

    SCANNER --> DB
    WATCHER --> DB
    HANDLERS --> DB
    HANDLERS --> CACHE_LAYER

    DB --> HANDLERS
    CACHE_LAYER --> HANDLERS

    HANDLERS --> JSON
    HANDLERS --> STREAM
    HANDLERS --> COVER

    COVER_FS --> COVER
    TRANSCODE_FS --> STREAM
```

---


## 音频元数据解析库

采用 lofty 

## 转码策略

- 通过 `std::process::Command` 调用系统 `ffmpeg`
- 转码命令示例: `ffmpeg -i input.flac -ab 320k -f mp3 -`
- 输出到 stdout，通过 `tokio::process::ChildStdout` 流式读取
- 转码结果可缓存到磁盘（按 `song_id + bit_rate` 作为 key）

---

## 状态管理

```mermaid
classDiagram
    class AppState {
        +DatabaseConnection db
        +CacheBackend cache
        +String server_host
        +u16 server_port
        +Option~ScanHandle~ scanner
        +Option~WatcherHandle~ watcher
    }

    class CacheBackend {
        +kind() &'static str
        +get(key) Option~String~
        +set(key, value, ttl)
        +del(key)
        +exists(key) bool
    }

    class DatabaseConnection {
        (SeaORM 连接池)
    }

    AppState --> CacheBackend
    AppState --> DatabaseConnection
```

`AppState` 通过 `axum::extract::State` 注入到每个 Handler 中，内部的可变状态（如扫描任务进度）使用 `Arc<RwLock<T>>` 保护。

---

## 安全设计

| 领域 | 措施 |
|------|------|
| 密码存储 | argon2id 哈希（推荐参数: m=19456, t=2, p=1） |
| 密码重置 | 6 位数字验证码 + 10 分钟过期 + 防枚举（统一返回 200） |
| Token | 32 字节随机 hex 字符串，可配置过期时间 |
| 文件访问 | 流媒体/下载时验证歌曲所属 library 是否对当前用户启用 |
| CORS | 可配置允许的来源 |
| 速率限制 | 登录端点 / 密码重置端点基于 IP 限流（可选） |
| 路径遍历 | 验证 `file_path` 在 `library.path` 前缀内，防止目录穿越 |
