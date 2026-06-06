# Crates 功能审查报告 — 2026-06-06

## 概述

OpenCAN 项目包含 3 个核心 crate + 1 个 GUI 应用：
- `can-traits` — CAN 硬件抽象层
- `canopen-core` — CANOpen 协议核心类型
- `canopen-ds301` — DS301 协议栈 + DS402 运动控制
- `opencan-gui` — Tauri v2 GUI 应用

---

## 1. can-traits（硬件抽象层）

| 功能 | 状态 | 说明 |
|------|------|------|
| `CanBus` trait | ✅ 完成 | send/recv/state/set_bitrate，trait object 安全 |
| `CanBusFactory` trait | ✅ 完成 | open/name/available_channels |
| `CanBusDyn` blanket impl | ✅ 完成 | Box::pin() 动态分发 |
| CanFrame (Classic/FD) | ✅ 完成 | 固定 8 字节 Classic + 可变长 FD |
| CanId, CanBitrate, CanConfig | ✅ 完成 | 标准/扩展 ID，FD 比特率 |
| SocketCAN 后端 | ⚠️ 部分 | Classic only，不支持 CAN FD |
| Kvaser 后端 | ❌ Stub | 返回 Unsupported |
| PCAN 后端 | ❌ Stub | 返回 Unsupported |
| ZLG 后端 | ❌ Stub | 返回 Unsupported |

### 问题
1. SocketCAN 不支持 CAN FD（`to_socketcan_frame` 对 FD 帧返回 Unsupported）
2. 三个硬件后端均为 stub，仅返回 `CanError::Unsupported`
3. `set_bitrate` 在 SocketCAN 中返回 Unsupported（需通过 `ip link` 设置）

---

## 2. canopen-core（核心类型）

| 功能 | 状态 | 说明 |
|------|------|------|
| CanOpenFrame | ✅ 完成 | 固定 8 字节，带时间戳 |
| CobId + FunctionCode | ✅ 完成 | 14 种功能码，Sync/Emergency 合并变体 |
| NMT 帧编解码 | ✅ 完成 | 5 种命令 |
| Heartbeat 帧编解码 | ✅ 完成 | 状态机 + 编解码 |
| Emergency 帧编解码 | ✅ 完成 | 错误码 + 寄存器 |
| SYNC 帧 | ✅ 完成 | 可选计数器 |
| Timestamp 帧 | ✅ 完成 | TIME_OF_DAY 6 字节 |
| SDO 请求/响应帧 | ✅ 完成 | expedited + segmented |
| PDO 帧 | ✅ 完成 | TPDO/RPDO 1-4 |
| 帧分类 (`classify_frame`) | ✅ 完成 | 按 COB-ID 自动分类 |
| ObjectDictionary trait | ✅ 完成 | read/write/entry_info |
| ConcreteOd (BTreeMap) | ✅ 完成 | 添加/删除/范围查询 |
| OdBuilder 流式 API | ✅ 完成 | add_var/build |
| OdValue (18 种类型) | ⚠️ 部分 | 缺少 40/48/56-bit 整数 |
| DataType (25 种) | ✅ 完成 | 全部 25 种 CANOpen 数据类型 |
| MockCanDriver | ✅ 完成 | 入队/出队/错误注入 |
| EDS 解析器 | ✅ 完成 | INI 格式，支持 sub-entry |
| EDS → OD 构建器 | ✅ 完成 | 类型转换 + 默认值 |
| PDO 类型 | ✅ 完成 | Direction/TransmissionType/Mapping |
| PDO pack/unpack | ✅ 完成 | 字节对齐打包/解包 |

### 问题
1. `OdValue` 缺少 `Integer40/48/56`、`Unsigned40/48/56` 变体（`from_bytes` 对这些类型返回 None）
2. 不支持 `UnicodeString`、`TimeOfDay`、`TimeDifference` 类型
3. `CanDriver` trait 的 `recv` 使用 `impl Future` 返回类型，限制了动态分发

---

## 3. canopen-ds301（DS301 协议栈）

| 功能 | 状态 | 说明 |
|------|------|------|
| CanopenStack 主循环 | ✅ 完成 | 帧分类 + 事件发射 |
| SDO 客户端 | ✅ 完成 | expedited + segmented upload/download |
| SDO 服务端 | ✅ 完成 | 响应远程 SDO 请求 |
| NMT Master | ✅ 完成 | start/stop/reset/broadcast |
| Heartbeat Consumer | ✅ 完成 | 超时检测 + 状态跟踪 |
| Heartbeat Producer | ✅ 完成 | 周期性发送 |
| SYNC Producer | ✅ 完成 | 可选计数器 |
| SYNC Consumer | ✅ 完成 | 触发同步 PDO |
| Emergency Handler | ✅ 完成 | 事件记录 + 错误描述 |
| PDO 处理 | ✅ 完成 | 解析 + 时间戳 |
| PDO 配置管理器 | ✅ 完成 | 通过 SDO 读写映射 |
| CanDriverAdapter | ✅ 完成 | CanBus ↔ CanDriver 桥接 |
| DS402 状态机 | ✅ 完成 | 8 状态 + 转换命令 |
| DS402 设备控制 | ✅ 完成 | enable/transition/mode |
| DS402 CSP 模式 | ✅ 完成 | 位置目标 + 实际值 |
| DS402 CST 模式 | ✅ 完成 | 力矩目标 + 实际值 |
| DS402 CSV 模式 | ✅ 完成 | 速度目标 + 实际值 |
| DS402 PP 模式 | ✅ 完成 | Profile Position |
| DS402 PV 模式 | ✅ 完成 | Profile Velocity |
| DS402 PT 模式 | ✅ 完成 | Profile Torque |
| DS402 Homing 模式 | ✅ 完成 | 回零操作 |

### 问题
1. DS402 操作模式 handler 缺少配置逻辑（加速度、减速度、速度限制等）
2. SDO 不支持 block transfer
3. Stack 中 `TimestampFrame` 分支标记为 TODO
4. `NmtMaster` 是独立 struct，但 Stack 内部也有 NMT 方法，存在冗余
5. SDO 客户端的 `upload` 默认使用 `DataType::Unsigned32`，可能不准确

---

## 4. opencan-gui（Tauri 应用）

| 功能 | 状态 | 说明 |
|------|------|------|
| Workspace 配置 | ❌ 错误 | `opencan-gui` 无 src/main.rs |
| Tauri v2 框架 | ✅ 完成 | 5 个插件 |
| Connection 命令 | ✅ 存在 | connect/disconnect/get_backends |
| NMT 命令 | ✅ 存在 | scan_nodes/nmt_command |
| SDO 命令 | ✅ 存在 | upload/download |
| DS402 命令 | ✅ 存在 | enable/fault_reset/set_mode/set_target |
| PDO 命令 | ✅ 存在 | read_pdo_mapping |
| SYNC 命令 | ✅ 存在 | start/stop |
| EDS 命令 | ✅ 存在 | load_eds_file |
| Recording 命令 | ✅ 存在 | start/stop/load/playback |

### 问题
1. **Workspace 成员路径错误**：`Cargo.toml` 列出 `opencan-gui` 但实际 crate 在 `opencan-gui/src-tauri/`
2. 导致整个 workspace 无法 `cargo check/build/test`

---

## 5. 测试覆盖

### 已有测试
- `canopen-core/src/frame.rs` — 15 个测试（编解码 + 分类）
- `canopen-core/src/od.rs` — 10 个测试（类型转换 + 便捷方法）
- `canopen-core/src/concrete_od.rs` — 8 个测试（CRUD + Builder）
- `canopen-core/src/pdo.rs` — 12 个测试（pack/unpack + 验证）
- `canopen-core/src/eds/parser.rs` — 4 个测试
- `canopen-core/src/eds/builder.rs` — 4 个测试
- `canopen-core/src/testing.rs` — 4 个测试（MockCanDriver）
- `canopen-ds301/src/heartbeat.rs` — 6 个测试（SYNC consumer）
- `canopen-ds301/src/emcy.rs` — 5 个测试
- `canopen-ds301/src/sdo_server.rs` — 7 个测试
- `canopen-ds301/src/pdo.rs` — 4 个测试
- `canopen-ds301/src/pdo_config.rs` — 4 个测试
- `canopen-ds301/src/adapter.rs` — 4 个测试
- `canopen-ds301/src/ds402/state_machine.rs` — 测试

### 缺失测试
- SDO 客户端端到端测试（需要 MockCanDriver 集成）
- Stack 集成测试（process 循环）
- DS402 设备控制测试
- EDS 完整文件解析测试

---

## 下一阶段任务规划

### P0 — 紧急修复

1. **修复 Workspace 配置**
   - 将 `Cargo.toml` 中的 `opencan-gui` 改为 `opencan-gui/src-tauri`
   - 确保 `cargo check --workspace` 通过

### P1 — 核心完善

2. **补充 OdValue 类型支持**
   - 添加 `Integer40/48/56`、`Unsigned40/48/56` 变体
   - 实现对应的 `from_bytes`/`to_bytes`
   - 添加 `try_as_i40` 等便捷方法

3. **SDO 客户端集成测试**
   - 使用 MockCanDriver 测试 expedited upload/download
   - 测试 segmented upload/download 全流程
   - 测试超时和错误处理

4. **Stack 集成测试**
   - 测试 process() 帧分类
   - 测试 NMT 命令发送
   - 测试 SDO 通过 Stack 的端到端

5. **DS402 模式配置完善**
   - 为每个操作模式添加 configure() 实现
   - 支持加速度 (0x6083)、减速度 (0x6084)、速度限制 (0x607F) 等参数
   - 添加 quick_stop_deceleration (0x8500) 支持

### P2 — 功能增强

6. **CAN FD 支持**
   - SocketCAN 后端添加 FD 帧支持
   - CanDriverAdapter 支持 FD 帧
   - 测试 CAN FD 编解码

7. **Block Transfer 支持**
   - SDO 客户端 block upload
   - SDO 服务端 block upload/download
   - 提高大数据传输效率

8. **PDO 动态配置**
   - 通过 SDO 运行时修改 PDO 映射
   - 支持 PDO 禁用/启用（COB-ID bit 31）
   - 支持 inhibit time 和 event timer

9. **TIME_STAMP 处理**
   - Stack 中处理 TimestampFrame
   - 提供时间同步 API

### P3 — 硬件后端

10. **Kvaser 后端实现**
    - 集成 Kvaser CANlib SDK
    - 实现 CanBus + CanBusFactory
    - Windows + Linux 支持

11. **PCAN 后端实现**
    - 集成 PCAN-Basic API
    - 实现 CanBus + CanBusFactory
    - Windows + Linux 支持

12. **ZLG 后端实现**
    - 集成 ZLG CAN API
    - 实现 CanBus + CanBusFactory

### P4 — GUI 完善

13. **GUI 架构审查**
    - 检查 Tauri 命令实现完整性
    - 前端状态管理
    - 错误处理和用户反馈

14. **GUI 功能测试**
    - 连接/断开流程
    - SDO 读写界面
    - DS402 控制面板
    - EDS 文件加载和显示

---

## 建议优先级

```
P0 (立即)  → 修复 workspace 配置
P1 (本周)  → OdValue 补充 + 测试覆盖
P2 (下周)  → CAN FD + Block Transfer
P3 (后续)  → 硬件后端
P4 (并行)  → GUI 完善
```
