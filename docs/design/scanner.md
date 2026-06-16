# Scanner 设计

## 概述

Scanner 是 Moosic 的音乐库扫描模块，负责遍历文件系统、提取音频元数据并将结果写入数据库。

参考 Navidrome 的设计，采用**两阶段扫描**策略：Quick Scan（快速比对）→ Full Scan（标签解析）。

---

## 两阶段扫描

### Phase 1: Walk（目录遍历）

递归遍历音乐库路径，收集所有音频文件信息：

```
library_path/
├── Artist A/
│   ├── Album 1/
│   │   ├── 01 - Song.mp3
│   │   └── 02 - Song.flac
│   └── Album 2/
│       └── ...
└── Artist B/
    └── ...
```

- 使用 `walkdir` 或 `std::fs::read_dir` 递归遍历
- 过滤：仅保留音频文件（mp3/flac/ogg/m4a/wav/aiff/wma/aac/opus/wv/ape）
- 收集每个文件的 `(path, mtime, size_bytes)` 元组
- 更新 `files_total` 为发现的音频文件总数

### Phase 2: Quick Scan（变更检测）

将文件系统中的文件列表与数据库中的 `songs` 表比对：

| 场景 | 判定条件 | 操作 |
|------|---------|------|
| **新增** | `file_path` 不在 `songs` 表中 | 加入待扫描列表 |
| **修改** | `file_path` 存在但 `mtime` 或 `size_bytes` 已变更 | 加入待扫描列表，标记需重新读取标签 |
| **未变** | `file_path` 存在且 `mtime` 和 `size_bytes` 均未变 | 跳过 |
| **删除** | `songs` 中有但文件系统已不存在 | 标记为待清理 |

- 批量加载 `songs` 表的 `(id, file_path, size_bytes, created_at)` 到 HashMap
- 用文件系统的文件列表与之比对
- `files_total` 更新为需要实际处理的数量

### Phase 3: Full Scan（元数据读取）

对 Quick Scan 标记为"新增"或"修改"的文件，调用 `lofty` 读取完整标签：

```
对每个待扫描文件:
  1. 读取元数据 (lofty)
  2. UPSERT artist → 获得 artist_id
  3. UPSERT album → 获得 album_id
  4. UPSERT song
  5. 如果有内嵌封面 → 提取并存入 cover_art 表
  6. 更新 files_scanned 进度
  7. 检查是否被取消
```

**UPSERT 策略：**
- **Artist**: 按 `(name, library_id)` 唯一性匹配。若存在则复用 ID，否则创建。
- **Album**: 按 `(name, artist_id, library_id)` 匹配。若存在则复用 ID，否则创建。
- **Song**: 按 `file_path` 唯一性匹配。若存在则更新所有字段，否则创建。

### Phase 4: Cleanup（清理孤记录）

删除数据库中 `file_path` 已不在文件系统中的歌曲记录，以及关联的孤艺术家和专辑：

```
1. DELETE FROM songs WHERE library_id = ? AND file_path NOT IN (当前文件列表)
2. DELETE FROM albums WHERE library_id = ? AND id NOT IN (SELECT DISTINCT album_id FROM songs WHERE album_id IS NOT NULL)
3. DELETE FROM artists WHERE library_id = ? AND id NOT IN (SELECT DISTINCT artist_id FROM songs)
```

---

## 并发与取消

### 后台执行

扫描在 `tokio::spawn` 中异步执行，不阻塞 HTTP 请求处理：

```rust
pub async fn start_scan(db: DatabaseConnection, scan_state: Arc<RwLock<ScanState>>, library_ids: Vec<i32>) {
    tokio::spawn(async move {
        run_scan(&db, &scan_state, &library_ids).await;
    });
}
```

### 取消机制

- 扫描循环中每隔 N 个文件（默认 10）检查一次 `scan_state`
- 若状态变为 `Cancelled`，立即停止并清理
- 取消时已完成的操作不回滚（部分扫描结果保留）

### 并发扫描拒绝

- 同一时间只允许一个扫描任务运行
- 新请求到达时若已有扫描在进行，返回 409 Conflict

---

## 增量扫描

### 基于 mtime 的增量

- 比较文件的 `mtime`（最后修改时间）和 `size_bytes`（文件大小）
- `mtime` 和 `size` 都未变的文件跳过标签重读
- 仅新增和修改的文件触发标签解析

### 文件监听（后续）

- `services/watcher.rs` 使用 `notify` crate 监听文件变更
- 文件变更事件触发单文件增量更新
- 批量变更（如移动文件夹）建议触发全量扫描

---

## 数据结构

### ScanProgress

```rust
pub struct ScanProgress {
    pub scan_id: String,        // 扫描任务 ID
    pub library_ids: Vec<i32>,  // 要扫描的库 ID 列表（空=全库）
    pub status: ScanStatus,     // 当前状态
    pub files_scanned: i64,     // 已扫描文件数
    pub files_total: i64,       // 待处理文件总数
    pub started_at: i64,        // 开始时间 ms
    pub error: Option<String>,  // 错误信息
}

pub enum ScanStatus {
    Scanning,
    Completed,
    Failed,
    Cancelled,
}
```

### WalkEntry（内部）

```rust
struct WalkEntry {
    file_path: String,   // 绝对路径
    mtime: i64,          // 13位毫秒时间戳
    size_bytes: i64,     // 文件大小
}
```

---

## 扫描生命周期

```
┌──────────┐    POST /api/admin/library/scan    ┌──────────────┐
│  Handler  │ ─────────────────────────────────> │   Scanner    │
│  (admin)  │ <────────── { scan_id } ────────── │   Service    │
└──────────┘                                    └──────┬───────┘
                                                       │
                                                       │ tokio::spawn
                                                       ▼
┌──────────────────────────────────────────────────────────────────┐
│                      Background Task                              │
│                                                                   │
│  Walk ───> Quick Scan ───> Full Scan ───> Cleanup ───> Done      │
│                                                                   │
│  更新 scan_state.files_total    │  每处理一个文件:                  │
│                                 │  scan_state.files_scanned += 1   │
│                                 │  检查 scan_state.status          │
│                                 │  若 Cancelled → 退出             │
└──────────────────────────────────────────────────────────────────┘
```

---

## 错误处理

- **不可读文件**: 跳过，记录 warning 日志，继续扫描
- **标签缺失字段**: 使用默认值（空字符串、0 等）
- **数据库写入失败**: 记录 error 日志，继续下一个文件
- **目录不存在**: 扫描开始前检查，返回 404
- **整个扫描失败**: 设置 status=Failed，记录 error 信息

---

## 文件结构

```
src/services/
├── scanner.rs          # 扫描编排（start/cancel/status + 后台任务）
└── metadata.rs         # lofty 音频标签读取（已实现）
```

扫描核心逻辑全部在 `scanner.rs` 中，因为它与 scan_state 紧密耦合，不宜拆散。

---

