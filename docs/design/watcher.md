# Watcher 设计

## 概述

Watcher 是 Moosic 的文件系统监听模块，使用 `notify` crate 实时监听所有音乐库目录下的文件变更，并在音频文件发生创建、修改或删除时自动增量更新数据库。

与 Scanner 的全量扫描不同，Watcher 提供**低延迟、细粒度**的增量更新能力。

---

## 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                      Watcher Task (tokio::spawn)                 │
│                                                                   │
│  ┌──────────┐    ┌──────────────┐    ┌──────────────────────┐   │
│  │  notify   │    │  Debounce    │    │  Event Processor      │   │
│  │  Watcher  │───▶│  (2s window) │───▶│                      │   │
│  │  (per-lib)│    │  Dedup paths │    │  Create/Modify →      │   │
│  └──────────┘    └──────────────┘    │    scan_single_file()  │   │
│                                      │  Delete →              │   │
│                                      │    delete_song()       │   │
│                                      └──────────────────────┘   │
│                                                                   │
│  Shutdown: oneshot::Sender<()>  ───▶  graceful stop              │
└─────────────────────────────────────────────────────────────────┘
```

---

## 事件处理策略

### 监听的事件类型

| notify EventKind | 含义 | 处理方式 |
|------------------|------|---------|
| `Create(Any)` / `Create(File)` | 新文件创建 | 调用 `scan_single_file()` 读取元数据并写入 DB |
| `Modify(Any)` / `Modify(Data)` / `Modify(Metadata)` | 文件内容或元数据变更 | 调用 `scan_single_file()` 重新读取标签并更新 DB |
| `Remove(Any)` / `Remove(File)` | 文件被删除 | 从 `songs` 表删除对应记录，清理孤 artist/album |

### 忽略的事件

- 非音频文件（通过 `metadata::is_audio_file()` 过滤）
- 临时文件（`.tmp`、开头的 `.` 隐藏文件等）
- 目录事件（交由全量扫描处理）

### 防抖机制

文件系统事件可能在短时间内大量到达（如编辑器保存时触发多次 write）。使用 **2 秒时间窗口** 收集事件并去重：

```
Event A ──┐
          ├── [收集 2s] ──▶ 去重(path) ──▶ 批量处理
Event A ──┤
Event B ──┘
```

同一路径在窗口内多次出现只处理一次，以最后一次事件类型为准。

---

## 与 Scanner 的协作

### 复用 Scanner 的单文件处理能力

Watcher 直接调用 `scanner::scan_single_file()` 处理单个文件的创建/修改事件，复用其元数据读取、UPSERT 逻辑和封面提取能力。

### 冲突避免

- Watcher 处理文件前检查是否有扫描任务正在运行
- 若 Scanner 正在扫描同一 library，Watcher **跳过**该事件（Scanner 会覆盖它）
- 反之，Watcher 处理过程中不阻塞 Scanner

### 批量变更降级

以下场景 Watcher 不自行处理，而是建议触发全量扫描：
- 大量文件同时变更（> 50 个事件/窗口）
- 目录移动/重命名

---

## 生命周期

```
main.rs
  │
  ├── start_watching(db, scan_state)
  │     │
  │     ├── 查询所有 watch_enabled=1 的 libraries
  │     ├── 为每个 library.path 创建 notify Watcher (Recursive)
  │     └── 返回 WatcherHandle { shutdown_tx }
  │
  │  tokio::spawn → run_watcher_loop()
  │     │
  │     ├── loop { rx.recv() → debounce → process }
  │     ├── shutdown signal → break
  │     └── drop watchers
  │
  └── graceful_shutdown → handle.shutdown()
```

---

## 错误处理

- **不可读文件**: 跳过，记录 warn 日志
- **标签解析失败**: 跳过，记录 warn（与 Scanner 一致）
- **数据库写入失败**: 记录 error，继续处理后续事件
- **Watch 失败**（目录被删除等）: 记录 error，停止该 library 的监听

---

## 数据结构

```rust
/// Handle to the running watcher task.
pub struct WatcherHandle {
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl WatcherHandle {
    /// Signal the watcher to stop gracefully.
    pub fn shutdown(self) {
        let _ = self.shutdown_tx.send(());
    }
}
```

### 内部结构

```rust
struct PendingEvent {
    /// The absolute path of the changed file.
    path: PathBuf,
    /// The library this file belongs to.
    library_id: i32,
    /// Whether the file currently exists on disk.
    exists: bool,
}
```

---

## 配置

监控启用/禁用通过数据库 `libraries.watch_enabled` 字段控制（由管理员 API 管理）：

- `POST /api/admin/library/notify/enable` — 启用监听
- `POST /api/admin/library/notify/disable` — 禁用监听

> 当前实现：Watcher 在启动时加载所有 `watch_enabled=1` 的 library 路径。运行时变更 `watch_enabled` 需要重启服务才能生效（后续版本可通过 channel 动态添加/移除监听路径）。

---

## 文件结构

```
src/services/
├── watcher.rs          # notify 文件监听服务（本文件）
├── scanner.rs          # 扫描编排（提供 scan_single_file 给 watcher 复用）
└── metadata.rs         # 音频标签读取
```
