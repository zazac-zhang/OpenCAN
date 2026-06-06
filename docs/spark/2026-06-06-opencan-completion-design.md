# OpenCAN 项目补齐设计文档

> 日期: 2026-06-06
> 状态: Approved
> 作者: MiMo + 用户协作

## 1. 概述

本文档规划 OpenCAN 项目从当前状态（Phase 12 完成，15,808 行 Rust 代码）到可发布状态的补齐工作。

### 1.1 当前状态

| 模块 | 状态 | 代码行数 | 测试数 |
|------|------|---------|--------|
| canopen-core | ✅ 完整 | ~3,500 | 69 |
| canopen-ds301 | ✅ 完整 | ~3,200 | 39 |
| can-traits | ⚠️ 部分 | ~500 | 0 |
| gui | ✅ 大量实现 | ~4,600 | 0 |
| **总计** | | **~15,800** | **~108** |

### 1.2 已知问题（来自审计报告）

| 优先级 | 问题 | 文件 |
|--------|------|------|
| 🔴 P0 | `SdoResponse::decode` segment size 位运算错误 | `frame.rs:310` |
| 🟡 P1 | SDO 逻辑在 SdoClient 和 CanopenStack 中重复 | `sdo.rs` + `stack.rs` |
| 🟡 P1 | HeartbeatTimeout 事件重复报告 | `stack.rs:119` |
| 🟡 P1 | CanError 在 canopen-core 和 can-traits 中重复 | `error.rs` |
| 🟢 P2 | Ds402Device::enable() 不检查当前状态 | `control.rs:81` |
| 🟢 P2 | EDS subindex 解析用十六进制 | `parser.rs:88` |
| 🟢 P2 | try_send 静默丢弃命令 | `backend.rs:106` |
| 🟢 P2 | oneshot channel 响应被丢弃 | `main.rs` |

### 1.3 设计决策

- **保持现有 crate 结构**：不拆分 ds402/eds/后端为独立 crate
- **保持硬件后端 stub**：Kvaser/PCAN/ZLG 不实现真实 FFI
- **分三批推进**：质量加固 → 功能完善 → 文档发布

---

## 2. 第一批：质量加固

### 2.1 P0 Bug 修复

**`SdoResponse::decode` segment size 位运算错误**

文件：`crates/canopen-core/src/frame.rs:310`

```rust
// 当前（错误）
Some(7 - ((cmd >> 4) & 0x07))

// 修正
Some(7 - ((cmd >> 1) & 0x07))
```

SDO segment 的 size 编码在 bits 1-3（`cmd & 0x0E`），正确提取是 `(cmd >> 1) & 0x07`。
对比 `stack.rs:247` 中的正确实现。

### 2.2 P1 问题修复

#### 2.2.1 SDO 逻辑重复

**问题**：SDO upload/download 逻辑在 `SdoClient`（`sdo.rs`）和 `CanopenStack`（`stack.rs`）中重复实现。

**方案**：`CanopenStack` 的 `sdo_upload` / `sdo_download` 方法委托给内部的 `SdoClient`，移除重复代码。

```rust
// stack.rs 修改后
pub async fn sdo_upload(&mut self, node_id: u8, index: u16, subindex: u8) -> Result<Vec<u8>, CanOpenError> {
    let client = SdoClient::new(&mut self.can, self.sdo_timeout);
    client.upload(node_id, index, subindex).await
}
```

#### 2.2.2 HeartbeatTimeout 重复报告

**问题**：`check_timeouts()` 每次调用都报告已超时的节点。

**方案**：添加 `reported_timeouts: HashSet<u8>` 集合，只在首次超时时报告。

```rust
// stack.rs
pub fn process(&mut self, frame: &CanOpenFrame) -> Vec<CanEvent> {
    // ... 收到 heartbeat 时清除 reported_timeouts 中对应节点
    self.reported_timeouts.remove(&node_id);
}

fn check_timeouts(&mut self) -> Vec<CanEvent> {
    // ... 只在节点未报告过超时时发送事件
    if !self.reported_timeouts.contains(&node_id) {
        self.reported_timeouts.insert(node_id);
        events.push(CanEvent::HeartbeatTimeout { node_id });
    }
}
```

#### 2.2.3 CanError 类型统一

**问题**：`canopen-core/error.rs` 和 `can-traits/error.rs` 各自定义了错误类型。

**方案**：
- `can-traits` 保留 `CanError`（硬件层错误）
- `canopen-core` 保留 `CanOpenError`（协议层错误）
- 添加 `impl From<CanError> for CanOpenError` 转换
- `canopen-ds301` 使用 `CanOpenError`，通过 `?` 自动转换

### 2.3 P2 问题修复

| 问题 | 修复方案 |
|------|---------|
| Ds402Device::enable() 不检查状态 | 添加 `if self.current_state == Ds402State::OperationEnabled { return Ok(()); }` |
| EDS subindex 十六进制 | `u8::from_str(subindex_str)` 替代 `u8::from_str_radix(s, 16)` |
| try_send 静默丢弃 | 添加 `tracing::warn!("Backend command dropped: channel full")` |
| oneshot 响应被丢弃 | 移除未使用的 oneshot channel，简化为 fire-and-forget |

### 2.4 补齐缺失测试

#### 2.4.1 SdoResponse::decode 测试（P0）

```rust
#[test]
fn test_sdo_response_decode_segment() {
    // 正常 segment response
    let cmd = 0x00; // size=7, n=0 → (0 >> 1) & 0x07 = 0 → 7-0=7
    let response = SdoResponse::decode(/* ... */);
    assert_eq!(response.data_len(), Some(7));

    // size=3 的 segment
    let cmd = 0x08; // n=4 → (0x08 >> 1) & 0x07 = 4 → 7-4=3
    // ...
}

#[test]
fn test_sdo_response_decode_abort() {
    // abort code 解析
}
```

#### 2.4.2 CanDriverAdapter roundtrip 测试（P1）

```rust
#[test]
fn test_canopen_frame_to_can_frame_roundtrip() {
    let original = CanOpenFrame::new(/* ... */);
    let can_frame = CanDriverAdapter::to_can_frame(&original);
    let restored = CanDriverAdapter::from_can_frame(&can_frame);
    assert_eq!(original, restored);
}
```

#### 2.4.3 EDS subindex ≥ 10 测试（P2）

```rust
#[test]
fn test_eds_subindex_decimal_parsing() {
    let entry = parse_entry("[1000]SubNumber=10");
    // subindex 10 应该是十进制 10，不是十六进制 0x10
}
```

#### 2.4.4 DS402 transition_to 无效路径测试（P1）

```rust
#[test]
fn test_ds402_invalid_transition() {
    let mut device = Ds402Device::new(/* ... */);
    // NotReadyToSwitchOn → OperationEnabled 应该失败
    assert!(device.transition_to(Ds402State::OperationEnabled).is_err());
}
```

#### 2.4.5 HeartbeatConsumer::check_timeouts 测试（P1）

```rust
#[test]
fn test_heartbeat_timeout_detection() {
    let mut consumer = HeartbeatConsumer::new(Duration::from_millis(100));
    consumer.update(1, Instant::now());
    // 等待超时
    let timeouts = consumer.check_timeouts(Instant::now() + Duration::from_millis(150));
    assert_eq!(timeouts, vec![1]);
}

#[test]
fn test_heartbeat_no_duplicate_timeout() {
    // 连续两次 check_timeouts 不应该重复报告
}
```

#### 2.4.6 SocketCan frame 转换测试（P1）

```rust
#[test]
fn test_socketcan_frame_conversion() {
    let can_frame = CanFrame::Classic(ClassicFrame { /* ... */ });
    let socketcan_frame = SocketCanBus::to_socketcan_frame(&can_frame).unwrap();
    let restored = SocketCanBus::from_socketcan_frame(&socketcan_frame);
    assert_eq!(can_frame, restored);
}
```

### 2.5 第一批交付标准

- [ ] P0 bug 修复 + 测试通过
- [ ] P1 问题全部修复
- [ ] P2 问题全部修复
- [ ] 6 个缺失测试区域全部补齐
- [ ] `cargo clippy --workspace --all-features -- -D warnings` 零警告
- [ ] `cargo test --workspace` 全部通过
- [ ] `cargo fmt --check` 通过

---

## 3. 第二批：功能完善

### 3.1 GUI SocketCAN 真正连接

**问题**：`ConnectionConnect` 消息处理中，SocketCAN 分支只设置 status message，不会真正建立连接。

**方案**：

```
用户点击连接
  → Message::ConnectionConnect
  → 创建 BackendCommand::Connect { interface, config }
  → Backend 任务中：
      1. SocketCanBus::open(interface)
      2. 创建 CanDriverAdapter
      3. 创建 CanopenStack
      4. 发送 BackendEvent::Connected
  → GUI 更新状态为 Connected
  → 启动 50ms 轮询 Subscription
```

**错误处理**：
- 连接失败 → 显示错误对话框（`Message::ShowError`）
- 运行中断开 → 发送 `BackendEvent::Disconnected`，GUI 重置状态

### 3.2 GUI 测试

#### 3.2.1 Backend Mock 测试

```rust
#[tokio::test]
async fn test_backend_connect_disconnect() {
    let (mut backend, mut rx) = Backend::new_mock();
    backend.send(BackendCommand::Connect { /* ... */ });
    assert!(matches!(rx.recv().await, Some(BackendEvent::Connected)));
    backend.send(BackendCommand::Disconnect);
    assert!(matches!(rx.recv().await, Some(BackendEvent::Disconnected)));
}
```

#### 3.2.2 State 逻辑测试

```rust
#[test]
fn test_app_update_sdo_result() {
    let mut app = App::default();
    app.update(Message::SdoResult { /* ... */ });
    assert_eq!(app.sdo_entries.len(), 1);
}
```

### 3.3 协议栈增强

#### 3.3.1 EMCY 解析

添加 EMCY error code 语义解析：

```rust
// canopen-core/src/emcy.rs
pub struct EmcyCode {
    pub error_code: u16,
    pub error_register: u8,
    pub vendor_specific: [u8; 5],
}

impl EmcyCode {
    pub fn description(&self) -> &'static str {
        match self.error_code {
            0x0000 => "Error Reset / No Error",
            0x1000 => "Generic Error",
            0x2000 => "Current Error",
            // ... 完整 DS301 EMCY code 映射
        }
    }
}
```

#### 3.3.2 NMT 状态机增强

添加节点 NMT 状态跟踪：

```rust
// canopen-ds301/src/nmt.rs
pub struct NmtStateMachine {
    state: NmtState,
    last_heartbeat: Option<Instant>,
}

pub enum NmtState {
    Initializing,
    PreOperational,
    Operational,
    Stopped,
}
```

### 3.4 第二批交付标准

- [ ] GUI 能真正连接 SocketCAN 接口
- [ ] GUI 连接/断开流程完整（含错误处理）
- [ ] Backend mock 测试通过
- [ ] State 逻辑测试通过
- [ ] EMCY 解析功能 + 测试
- [ ] NMT 状态机跟踪 + 测试
- [ ] `cargo test --workspace` 全部通过

---

## 4. 第三批：文档 + 发布准备

### 4.1 项目文档

#### 4.1.1 API 文档

为所有 `pub` 类型、trait、函数添加 `///` 文档注释。重点模块：

- `canopen-core`: `CanOpenFrame`, `CobId`, `ObjectDictionary`, `OdValue`
- `canopen-ds301`: `CanopenStack`, `SdoClient`, `CanEvent`
- `can-traits`: `CanBus`, `CanBusFactory`, `CanFrame`
- `gui`: `App`, `Message`, `BackendCommand`, `BackendEvent`

#### 4.1.2 CONTRIBUTING.md

```markdown
# Contributing to OpenCAN

## Development Setup
1. Install Rust 1.75+ (2024 edition)
2. Clone repository
3. `cargo check --workspace`

## Code Style
- `cargo fmt` 格式化
- `cargo clippy --workspace --all-features -- -D warnings` 零警告
- 所有 pub 类型必须有文档注释

## Testing
- 新功能必须有单元测试
- 协议栈改动需要集成测试
- GUI 改动需要 state/backend 测试

## Pull Request
1. Fork & branch
2. 确保 CI 通过
3. 提交 PR，描述变更内容
```

#### 4.1.3 ARCHITECTURE.md

系统架构图、模块职责、数据流、设计决策文档。

#### 4.1.4 CHANGELOG.md

按 [Keep a Changelog](https://keepachangelog.com/) 格式：

```markdown
# Changelog

## [Unreleased]
### Fixed
- SdoResponse::decode segment size 位运算错误
- HeartbeatTimeout 事件重复报告

### Added
- GUI SocketCAN 真正连接
- EMCY error code 解析
- NMT 状态机跟踪
```

### 4.2 版本策略

| Crate | 版本 | 说明 |
|-------|------|------|
| can-traits | 0.1.0 | API 可能变动 |
| canopen-core | 0.1.0 | 核心库 |
| canopen-ds301 | 0.1.0 | 协议栈 |
| gui | 0.0.1 | 未达到 0.1.0 |

### 4.3 发布流程

```bash
# 1. 更新版本号
# 2. 更新 CHANGELOG.md
# 3. 按依赖顺序发布
cargo publish -p opencan-can-traits
cargo publish -p opencan-canopen-core
cargo publish -p opencan-canopen-ds301
# gui 不发布到 crates.io（应用而非库）
```

### 4.4 项目文件

| 文件 | 说明 |
|------|------|
| LICENSE-MIT | MIT 许可证全文 |
| LICENSE-APACHE | Apache 2.0 许可证全文 |
| .editorconfig | 编辑器配置 |
| deny.toml | `cargo-deny` 依赖审计配置 |

### 4.5 第三批交付标准

- [ ] 所有 pub 类型有文档注释
- [ ] `cargo doc --workspace --no-deps` 零警告
- [ ] CONTRIBUTING.md 完整
- [ ] ARCHITECTURE.md 完整
- [ ] CHANGELOG.md 完整
- [ ] LICENSE 文件存在
- [ ] 版本号更新
- [ ] crates.io 发布流程验证

---

## 5. 实施顺序

```
第一批（质量加固）
  ├── 5.1 修复 P0 SdoResponse::decode bug + 测试
  ├── 5.2 修复 P1 SDO 逻辑重复
  ├── 5.3 修复 P1 HeartbeatTimeout 重复
  ├── 5.4 修复 P1 CanError 统一
  ├── 5.5 修复 P2 级别问题（4 个）
  └── 5.6 补齐 6 个缺失测试区域

第二批（功能完善）
  ├── 5.7 GUI SocketCAN 真正连接
  ├── 5.8 GUI 测试（backend mock + state 逻辑）
  ├── 5.9 EMCY 解析
  └── 5.10 NMT 状态机增强

第三批（文档发布）
  ├── 5.11 API 文档注释
  ├── 5.12 CONTRIBUTING.md + ARCHITECTURE.md
  ├── 5.13 CHANGELOG.md + LICENSE
  └── 5.14 版本策略 + 发布流程
```

---

## 6. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| SDO 逻辑重构引入回归 | 🔴 高 | 先补齐测试，再重构 |
| GUI iced API 变动 | 🟡 中 | 锁定 iced 0.13 版本 |
| SocketCAN 仅 Linux 可用 | 🟡 中 | CI 中用 vcan0 测试 |
| EMCY code 映射不完整 | 🟢 低 | 按需添加，不阻塞发布 |

---

## 7. 验收标准

### 每批验收

- `cargo check --workspace` 通过
- `cargo test --workspace` 全部通过
- `cargo clippy --workspace --all-features -- -D warnings` 零警告
- `cargo fmt --check` 通过

### 最终验收

- 所有 P0/P1 问题已修复
- 测试覆盖率提升（目标：120+ 测试）
- 所有 pub 类型有文档注释
- CHANGELOG.md 记录所有变更
- 可通过 `cargo publish` 发布 can-traits / canopen-core / canopen-ds301
