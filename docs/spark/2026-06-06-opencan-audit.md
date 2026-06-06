# OpenCAN 设计审查报告

> 审查日期: 2026-06-06
> 审查对象: [docs/spark/2026-06-06-opencan-design.md](2026-06-06-opencan-design.md)
> 状态: Complete

---

## 一、总体评价

**架构思路合理，分层清晰，crate 拆分方向正确。** 但部分技术细节需要更新（iced API 已过时），部分后端选型可优化（PCAN/ZLG 有现成 crate 无需自研绑定），且 CanBus trait 的设计存在 trait object 不可用的问题。

整体可行性：⭐⭐⭐⭐（5 星制）

---

## 二、依赖选型审查

### 2.1 需要更正的依赖

| 组件 | 文档标注 | 实际情况 | 建议 |
|------|---------|---------|------|
| SocketCAN | `socketcan-rs` 3.5.0 | crates.io 名称为 **`socketcan`**（GitHub 仓库叫 `socketcan-rs/socketcan-rs`） | 更正为 `socketcan` 3.5.0 |
| PCAN | 🔧 自研绑定 (bindgen) | 已有 `peak-can` / `peak-can-sys` / `pcanbasic` crate 可用 | 改用 `peak-can` 或 `pcanbasic` |
| ZLG | 🔧 自研绑定 (bindgen) | 已有 `zlgcan` crate 可用（跨平台 Win+Linux） | 改用 `zlgcan` |
| GUI (iced) | `iced` API 使用 0.12 语法 | iced 0.14（2025-12）已将 `Command` 重命名为 `Task`，API 重大变更 | 更新为 iced 0.14+ 语法 |

### 2.2 成熟度确认

| 依赖 | 版本 | 状态 |
|------|------|------|
| `socketcan` | 3.5.0 | ✅ 成熟，2024-12 发布 |
| `can-hal-kvaser` | 0.3.3 | ⚠️ 可用（2026-04），需用户预装 CANlib SDK |
| `peak-can` | 可用 | ✅ 可用，基于 PCAN-Basic FFI |
| `pcanbasic` | MSRV 1.77+ | ✅ 可用，约 11 个月前发布 |
| `zlgcan` | 可用 | ⚠️ 社区小，文档少，API 稳定性未知 |
| `iced` | 0.14 | ⚠️ pre-1.0，API 可能再变 |

### 2.3 CANOpen 生态调研

文档结论"Rust 生态无成熟方案"——**正确但不完整**：

| Crate | SDO | PDO | NMT | DS402 | 状态 |
|-------|-----|-----|-----|-------|------|
| `funcan-rs` | ✓ 完整 | ✓ | 部分 | 无 | v0.1.0, 2025-01 |
| `oze-canopen` | ✓ 异步 | ✓ | 部分 | 无 | tokio + socketcan |
| `zencan-node` | 计划中 | 计划中 | 计划中 | 未知 | no_std 嵌入式 |
| `canopeners` | 基础 | 基础 | 无 | 无 | 2024-01 |
| `canopen` | 基础 | 基础 | 无 | 无 | 2018 年，已停滞 |

**无任何 Rust crate 提供完整 DS301 + DS402 支持。** 自研决策正确。但可参考：
- `funcan-rs` 的 SDO 编解码（segmented/expedited transfer）
- `oze-canopen` 的异步架构（tokio + socketcan stream）

---

## 三、关键问题（必须修改）

### 3.1 `CanBus` trait 不可作为 trait object

```rust
// 文档原文
pub trait CanBus: Send + Sync + 'static {
    fn open(channel: &str) -> Result<Self> where Self: Sized;  // ← 无法作为 trait object
    fn recv(&self) -> Result<CanFrame>;
    async fn recv_async(&self) -> Result<CanFrame>;
}
```

**问题**：`open` 和 `recv_async` 带 `Self: Sized` → `Box<dyn CanBus>` 编译报错。GUI 层无法动态切换后端。

**建议**：分离构造和运行时接口

```rust
pub trait CanBus: Send + Sync + 'static {
    fn send(&self, frame: &CanFrame) -> Result<()>;
    fn recv(&self) -> Result<CanFrame>;
    fn state(&self) -> CanState;
    fn set_bitrate(&self, bitrate: CanBitrate) -> Result<()>;
}

pub trait CanBusFactory: Send + Sync + 'static {
    fn open(&self, channel: &str, bitrate: CanBitrate) -> Result<Box<dyn CanBus>>;
}
```

### 3.2 iced API 已过时

```rust
// 文档中的写法 (iced ≤0.12)
fn update(&mut self, message: Message) -> iced::Command<Message> {
    iced::Command::perform(...)
}
```

**实际情况**：iced 0.14 已将 `Command` 重命名为 `Task`。

```rust
// 正确写法 (iced ≥0.14)
fn update(&mut self, message: Message) -> Task<Message> {
    Task::perform(...)
}
```

### 3.3 `FunctionCode` 枚举值冲突

```rust
pub enum FunctionCode {
    Nmt       = 0x000,
    Sync      = 0x080,
    Emergency = 0x080,  // ← 与 Sync 同值，Rust enum 不允许重复
    // ...
}
```

**建议**：

```rust
// 方案 A：使用 function code 序号（不含 base offset）
#[repr(u8)]
pub enum FunctionCode {
    Nmt       = 0x00,
    Sync      = 0x01,    // COB-ID = 0x080 + 0*node_id
    Emergency = 0x01,    // COB-ID = 0x080 + node_id (通过 node_id 区分)
    Timestamp = 0x02,
    Tpdo1     = 0x03,
    // ...
}

impl FunctionCode {
    pub fn cob_id(&self, node_id: u8) -> u16 {
        (self.base_offset() << 7) | (node_id as u16 & 0x7F)
    }
}

// 方案 B：使用 const
pub const FC_NMT: u8 = 0x00;
pub const FC_SYNC_EMCY: u8 = 0x01;  // 通过 node_id 区分
```

### 3.4 `recv` 同步方法与 async 模型冲突

```rust
fn recv(&self) -> Result<CanFrame>;           // 阻塞调用
async fn recv_async(&self) -> Result<CanFrame>; // 异步
```

**问题**：
- GUI 层只用异步路径，同步 `recv` 永远不会被调用
- PCAN/ZLG 的 C SDK 全是阻塞调用，要实现 `recv_async` 需要额外线程包装
- 同时维护两条路径增加工作量

**建议**：Phase 1 只保留 `recv_async`。同步路径后续在 canopen-core 层按需添加。

---

## 四、架构层面问题

### 4.1 `CanDriver` 与 `CanBus` 职责未明确衔接

两层 trait 接口几乎一样，只是帧类型不同（`CanOpenFrame` vs `CanFrame`）。文档未说明如何衔接。

**建议**：定义 adapter

```rust
pub struct CanOpenAdapter<B: CanBus> {
    bus: B,
}

impl<B: CanBus> CanDriver for CanOpenAdapter<B> {
    // 内部将 CanOpenFrame 编码为 CanFrame 发送
}
```

或 Phase 1 不在 canopen-core 中定义 `CanDriver` trait，仅定义 CANOpen 帧类型和对象字典。

### 4.2 tokio + iced 集成缺少验证

文档声称"无需桥接层"，但 iced 0.14 使用 `futures` executor，`socketcan::tokio::CanSocket` 需要 tokio runtime。两者之间需要桥接。

**推荐方案**：

```rust
fn subscription(&self) -> Subscription<Message> {
    subscription::channel(conn.id(), 100, |mut output| {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        tokio::spawn(async move {
            loop {
                if let Ok(frame) = socket.read_frame().await {
                    let _ = tx.send(frame).await;
                }
            }
        });
        async move {
            while let Some(frame) = rx.recv().await {
                let _ = output.send(Message::FrameReceived(frame)).await;
            }
        }
    })
}
```

**强烈建议**：Phase 5 之前做一个 PoC 验证此方案可行性。

### 4.3 ClassicFrame data 字段效率问题

```rust
pub struct ClassicFrame {
    pub data: Vec<u8>,        // max 8 bytes — 不必要的堆分配
}
```

**建议**：

```rust
pub struct ClassicFrame {
    pub id: CanId,
    pub data: [u8; 8],
    pub len: u8,              // 0..=8
    pub timestamp: Option<Instant>,
}
```

### 4.4 Feature flag 设计过于排他

```toml
socketcan = ["opencan-can-socketcan"]
pcan = ["opencan-can-pcan"]
```

同一时间只能启用一个后端。但用户可能同时拥有多种硬件。

**建议**：

```toml
[features]
default = ["backend-socketcan"]
backend-socketcan = ["dep:opencan-can-socketcan"]
backend-kvaser = ["dep:opencan-can-kvaser"]
backend-pcan = ["dep:opencan-can-pcan"]
backend-zlg = ["dep:opencan-can-zlg"]
```

GUI 运行时通过 `CanBusFactory` 注册表选择后端。

---

## 五、Phase 逐项审查

### Phase 1：canopen-core + can-traits + MockCanDriver + 单元测试

#### 5.1 缺失的类型定义

| 缺失项 | 说明 | 归属 |
|--------|------|------|
| `CanError` | 文档引用但未定义 | can-traits |
| `OdError` | `CanOpenError` 依赖但未定义 | canopen-core |
| `SdoAbortCode` | 文档说"已内置"但未定义 | canopen-core |
| `EntryInfo` | `ObjectDictionary` 依赖但未定义 | canopen-core |
| `DataType` / `AccessType` / `EntryKind` | `EntryInfo` 的组成 | canopen-core |
| `CanState` | `CanBus::state()` 返回值 | can-traits |

#### 5.2 `ObjectDictionary` 设计

```rust
pub trait ObjectDictionary: Send {
    fn read(&self, index: u16, subindex: u8) -> Result<OdValue, OdError>;
    fn write(&mut self, index: u16, subindex: u8, value: OdValue) -> Result<(), OdError>;
    fn entry_info(&self, index: u16, subindex: u8) -> Result<EntryInfo, OdError>;
}
```

**建议补充**：

```rust
pub struct EntryInfo {
    pub data_type: DataType,
    pub access: AccessType,
    pub kind: EntryKind,  // Var, Record, Array
}

pub enum DataType {
    Boolean, Integer8, Integer16, Integer32, Integer64,
    Unsigned8, Unsigned16, Unsigned32, Unsigned64,
    Real32, Real64, VisibleString, OctetString, Domain,
}

pub enum AccessType { ReadOnly, WriteOnly, ReadWrite, Const }
pub enum EntryKind { Var, Record { sub_count: u8 }, Array { max_sub: u8 } }

pub enum OdValue {
    Boolean(bool),
    Integer8(i8), Integer16(i16), Integer32(i32), Integer64(i64),
    Unsigned8(u8), Unsigned16(u16), Unsigned32(u32), Unsigned64(u64),
    Real32(f32), Real64(f64),
    VisibleString(String),
    OctetString(Vec<u8>),
    Domain(Vec<u8>),
    None,            // entry 存在但无值
    Raw(Vec<u8>),    // 未知类型的原始数据
}
```

#### 5.3 `CanError` 建议定义

```rust
#[derive(Error, Debug)]
pub enum CanError {
    #[error("bus off")]
    BusOff,
    #[error("device not found: {0}")]
    NotFound(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("frame data too large: {0} bytes (max {1})")]
    DataTooLong(usize, usize),
}
```

#### 5.4 `OdError` 建议定义

```rust
#[derive(Error, Debug)]
pub enum OdError {
    #[error("object {0:#06x} not found in dictionary")]
    NoSuchObject(u16),
    #[error("subindex {1} not found in object {0:#06x}")]
    NoSuchSubindex(u16, u8),
    #[error("attempt to write read-only object {0:#06x}")]
    ReadOnly(u16),
    #[error("attempt to read write-only object {0:#06x}")]
    WriteOnly(u16),
    #[error("data type mismatch: expected {expected:?}, got {actual:?}")]
    TypeMismatch { expected: DataType, actual: DataType },
    #[error("value out of range")]
    OutOfRange,
}
```

#### 5.5 MockCanDriver 设计

```rust
pub struct MockCanDriver {
    pub rx_queue: VecDeque<CanOpenFrame>,
    pub tx_log: Vec<CanOpenFrame>,
    pub rx_delay: Duration,
    pub error_inject: Option<CanOpenError>,
}
```

注意：canopen-core 是 `[no_std]`，MockCanDriver **不应放在 canopen-core 中**。建议放在 `can-traits` 或独立的 `canopen-test-utils` crate（可带 tokio 依赖）。

#### 5.6 Phase 1 时间评估

| 任务 | 文档预估 | 实际评估 |
|------|---------|---------|
| can-traits（CanBus + CanFrame + CanError） | 含在 1-2 周 | 2-3 天 |
| canopen-core（类型 + ObjectDictionary + 错误） | 含在 1-2 周 | 3-4 天 |
| FunctionCode / CobId 编解码 | 未单独列出 | 1-2 天 |
| MockCanDriver | 含在 1-2 周 | 1-2 天 |
| OdValue ↔ bytes 序列化 | 未单独列出 | 1-2 天 |
| 测试（约 15-20 用例） | 含在 1-2 周 | 2-3 天 |
| **总计** | **1-2 周** | **10-16 天 ≈ 2-3 周** |

**建议拆分**：
- **Phase 1a（1 周）**：can-traits + canopen-core 基础类型 + ObjectDictionary
- **Phase 1b（1 周）**：CobId 编解码 + SDO 数据序列化 + MockCanDriver + 测试

#### 5.7 Phase 1 修正后的文件结构

```
crates/
├── can-traits/
│   └── src/
│       ├── lib.rs           ← CanBus, CanBusFactory, CanError, CanBitrate
│       ├── frame.rs         ← ClassicFrame, FdFrame, CanId
│       └── state.rs         ← CanState
│
├── canopen/
│   └── canopen-core/
│       └── src/
│           ├── lib.rs       ← re-exports
│           ├── frame.rs     ← CanOpenFrame, CobId, FunctionCode
│           ├── od/
│           │   ├── mod.rs   ← ObjectDictionary trait
│           │   ├── value.rs ← OdValue, DataType
│           │   ├── entry.rs ← EntryInfo, AccessType, EntryKind
│           │   └── map.rs   ← HashMap-based 默认实现
│           ├── error.rs     ← CanOpenError, OdError, SdoAbortCode
│           └── codec.rs     ← COB-ID 编解码, OdValue ↔ bytes
│
├── canopen-test-utils/      ← [dev-dependency 或独立 crate]
│   └── src/
│       └── mock.rs          ← MockCanDriver
```

---

### Phase 2：canopen-ds301（SDO/NMT/Heartbeat/EMCY）

#### 5.8 SDO 客户端 — 状态机复杂度被低估

```rust
// 文档签名
pub async fn upload<T: OdEntry>(&mut self, node_id: u8, index: u16, subindex: u8) -> Result<T, SdoError>;
```

**问题 1**：SDO segmented transfer 是有状态的双向通信（toggle bit 翻转），内部需要状态机：

```rust
enum SdoTransferState {
    Idle,
    AwaitingResponse { request: SdoRequest, retries: u8 },
    Segmenting { remaining: Vec<u8>, toggle: bool },
    Complete,
}
```

**问题 2**：`T: OdEntry` bound 不清晰。SDO upload 返回原始字节流，由调用方解析：

```rust
pub async fn upload(&mut self, node_id: u8, index: u16, subindex: u8) -> Result<OdValue, SdoError>;
pub async fn upload_raw(&mut self, node_id: u8, index: u16, subindex: u8) -> Result<Vec<u8>, SdoError>;
```

#### 5.9 NMT Master 需要状态跟踪

```rust
// 文档中 NmtMaster 是无状态的
pub struct NmtMaster;
```

实际需要跟踪节点状态并通过 Heartbeat 确认状态转换：

```rust
pub struct NmtMaster {
    node_states: HashMap<u8, NmtState>,
}

pub enum NmtState {
    Initialization,
    PreOperational,
    Operational,
    Stopped,
}
```

#### 5.10 HeartbeatConsumer 缺少超时配置

每个节点的 heartbeat 超时时间不同（由 OD 0x1016 定义），需要为每个节点配置阈值：

```rust
struct NodeHeartbeat {
    last_seen: Instant,
    timeout: Duration,
    current_state: NmtState,
}
```

#### 5.11 `scan_nodes` 设计不可行

CANOpen 协议**没有标准的节点发现机制**。文档中的 `scan_nodes` 签名过于简化。

建议提供多种扫描策略：

```rust
pub enum ScanStrategy {
    SdoQuery { timeout: Duration },       // 遍历 1-127，SDO upload 0x1000:0
    HeartbeatListen { duration: Duration }, // 非侵入，监听一段时间
    NmtBroadcast,                          // 会中断运行中的节点
}
```

#### 5.12 CanEvent 缺少 Sync 事件

PDO 的触发依赖 Sync 帧（COB-ID 0x080）。GUI 需要知道 Sync 是否到达：

```rust
pub enum CanEvent {
    // ... existing variants
    Sync,
}
```

---

### Phase 3：canopen-ds402 + canopen-eds

#### 5.13 DS402 状态机 — 缺少 StatusWord / ControlWord

```rust
// 文档仅定义了 Ds402State 枚举
pub enum Ds402State { ... }
```

DS402 实际依赖两个 16-bit 寄存器：
- **Status Word** (0x6041:00) — 位级别编码，需解析 bit 0-5
- **Control Word** (0x6040:00) — 写入以触发状态转换

```rust
pub struct StatusWord(pub u16);
impl StatusWord {
    pub fn state(&self) -> Ds402State { ... }
    pub fn is_fault(&self) -> bool { ... }
    pub fn warning(&self) -> bool { ... }
}

pub struct ControlWord(pub u16);
impl ControlWord {
    pub fn shutdown_command(&self) -> Self { ... }
    pub fn enable_operation_command(&self) -> Self { ... }
}
```

#### 5.14 缺少 ModeOfOperation 寄存器区分

DS402 有两个 mode 寄存器：
- `0x6060` (ModesOfOperation) — 写入目标模式
- `0x6061` (ModeOfOperationDisplay) — 读取实际确认的模式

`set_mode` 需要等待 `0x6061` 确认。

#### 5.15 EDS 解析 — ObjectType 不能忽略

文档 §6 明确说"忽略 ObjectType"。但 ObjectType 是区分 VAR/RECORD/ARRAY 的关键：

```ini
[1003]
ObjectType=0x8   ← ARRAY，有 255 个子索引

[1008]
ObjectType=0x7   ← VAR，无子索引
```

忽略 ObjectType 会导致 ARRAY 和 RECORD 类型被当作 VAR 处理，丢失子条目。

**建议**：至少区分三种类型

```rust
pub enum ObjectType {
    Var,       // 0x07
    Record,    // 0x09
    Array,     // 0x08
}
```

---

### Phase 4：硬件后端

#### 5.16 工作量重估

| 后端 | 文档预估 | 实际评估 | 说明 |
|------|---------|---------|------|
| SocketCAN | 含在 2-3 周 | 2-3 天 | `socketcan` crate 成熟 |
| Kvaser | 含在 2-3 周 | 3-4 天 | `can-hal-kvaser` 可用 |
| PCAN | 含在 2-3 周 | 2-3 天 | `peak-can` crate 可用 |
| ZLG | 含在 2-3 周 | 3-5 天 | `zlgcan` 可用，文档少 |
| 统一适配层 | 未单独列出 | 2-3 天 | 将 4 个后端统一为 `CanBus` trait |

2-3 周的预估合理，前提是**使用现有 crate 而非自研绑定**。

#### 5.17 各后端外部依赖

| 后端 | 外部依赖 | 安装要求 |
|------|---------|---------|
| SocketCAN | Linux 内核 | 无需额外安装 |
| Kvaser | CANlib SDK | Windows 需安装 `canlib32.dll`，Linux 需驱动 |
| PCAN | PCAN-Basic | Windows 需安装驱动，Linux 需 `libpcanbasic` |
| ZLG | ZLG SDK | Windows 需 `zlgcan.dll`，Linux 需对应 .so |

文档需标注这些要求，用户必须自行安装厂商 SDK。

---

### Phase 5：GUI

#### 5.18 iced + tokio 集成风险

如 4.2 节所述，iced 使用 futures executor，socketcan 使用 tokio。需要桥接方案。**强烈建议 Phase 5 前先做 PoC 验证。**

#### 5.19 实时数据曲线 — iced 缺少图表组件

| 方案 | 状态 | 适用性 |
|------|------|--------|
| `plotters-iced` | 可用 | 适合静态图，实时刷新性能一般 |
| `iced_aw` Graph | 可用 | 功能有限 |
| 自绘 widget | 需实现 `Widget` | 工作量大 |

**建议**：Phase 5 先用 `iced_aw` 的 `Graph` 做基础折线图，"实时数据曲线"可降级为后续 Phase。

#### 5.20 PDO 高频刷新 — GUI 性能

PDO 可能以 10ms-100ms 周期到达。在 iced Elm 架构中，每条 PDO 触发 `update → view` 重渲染。

**缓解措施**：
1. subscription 层做**节流**（throttle）：每 50ms 批量投递
2. 使用 iced 0.14 reactive rendering（如已稳定）
3. PDO 表格使用 `virtualized_list`（iced_aw 提供）

---

## 六、CI/CD 审查

### 6.1 平台兼容性

```yaml
# 文档中的配置 — 有问题
matrix:
  features: ["socketcan", "pcan", "kvaser", "zlg", ""]
```

- `socketcan` 只能在 Linux 上编译/测试
- PCAN/Kvaser/ZLG 的 C SDK 需要安装在 runner 上

**建议**：

```yaml
test-linux:
  runs-on: ubuntu-latest
  steps:
    - cargo test --workspace
    - cargo test --workspace --features backend-socketcan

test-macos:
  runs-on: macos-latest
  steps:
    - cargo test --workspace

test-windows:
  runs-on: windows-latest
  steps:
    - cargo test --workspace

lint:
  runs-on: ubuntu-latest
  steps:
    - cargo clippy --workspace -- -D warnings
    - cargo fmt --check
    - cargo doc --workspace --no-deps
```

---

## 七、Phase 0 建议：PoC 验证

建议新增一个 **Phase 0（1 周）**，与 Phase 1 并行进行：

```
Phase 0：PoC 验证（1 周）
- 验证 tokio + iced 集成方案
- 验证 socketcan → iced subscription 的帧流
- 验证 PDO 高频刷新下的 GUI 性能
- 产出：技术可行性报告
```

---

## 八、风险评估汇总

| 风险 | 等级 | 说明 | 缓解措施 |
|------|------|------|---------|
| iced API 不稳定 | 🟡 中 | 0.14 仍 pre-1.0 | 锁定版本，关注 release notes |
| tokio + iced 集成 | 🟡 中 | 文档未提供具体方案 | Phase 5 前做 PoC |
| ZLG crate 质量 | 🟡 中 | `zlgcan` 社区小，文档少 | 提前评估 API 稳定性 |
| DS402 状态转换 | 🔴 高 | 不同厂商驱动器行为可能不一致 | 设计可扩展的状态机 |
| SDO 超时处理 | 🟡 中 | 不同设备响应时间差异大 | 可配置 timeout |
| PDO 高频刷新卡顿 | 🟡 中 | iced 重渲染模型不适合高频更新 | 节流 + virtualization |
| EDS 文件格式不一致 | 🟢 低 | EDS 是标准 INI 格式 | 保留 ObjectType 解析 |

---

## 九、整体评分

| 维度 | Phase 1 | Phase 2 | Phase 3 | Phase 4 | Phase 5 |
|------|:-------:|:-------:|:-------:|:-------:|:-------:|
| 需求清晰度 | ⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| 技术可行性 | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| 时间合理性 | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| 风险可控度 | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |

**最大风险项**：Phase 5（GUI）技术不确定性最高。

---

## 十、修正后的 workspace 依赖管理

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "crates/can-traits",
    "crates/can-socketcan",
    "crates/can-kvaser",
    "crates/can-pcan",
    "crates/can-zlg",
    "crates/canopen/canopen-core",
    "crates/canopen/canopen-ds301",
    "crates/canopen/canopen-ds402",
    "crates/canopen/canopen-eds",
    "crates/gui",
]
resolver = "2"

[workspace.dependencies]
thiserror = "2.0"
socketcan = "3.5"
peak-can = "0.6"
can-hal-kvaser = "0.3"
zlgcan = "0.3"
iced = { version = "0.14", features = ["tokio"] }
tokio = { version = "1.43", features = ["full"] }
futures = "0.3"
```

---

## 十一、版本策略

| Crate | 建议起始版本 | 说明 |
|-------|------------|------|
| can-traits | 0.1.0 | API 可能变动 |
| canopen-core | 0.1.0 | no_std 兼容性待验证 |
| canopen-ds301 | 0.1.0 | 需实际硬件验证 |
| canopen-ds402 | 0.1.0 | 依赖 ds301 稳定性 |
| canopen-eds | 0.1.0 | 可选模块 |
| gui | 0.0.1 | 未达到 0.1.0 之前 |

---

## 十二、实现代码审查（2026-06-06 代码已读取）

### 12.1 编译与测试状态

| 检查项 | 结果 |
|--------|------|
| `cargo check --workspace` | ✅ 通过，无错误 |
| `cargo clippy --workspace -- -D warnings` | ✅ 通过，无警告 |
| `cargo test --workspace` | ✅ 通过，**46 个测试全部通过** |
| 测试分布 | canopen-core: 15, ds301: 17, ds402: 8, eds: 3, gui: 3 |

### 12.2 整体评价

代码实现质量远高于设计文档。**审计报告中指出的绝大多数问题已被修复。**

### 12.3 can-traits — 审查

#### ✅ 审计问题已修复

| 审计问题 | 文档设计 | 实际实现 |
|---------|---------|---------|
| trait object 不可用 | `fn open() -> Result<Self> where Self: Sized` | 分离为 `CanBus` + `CanBusFactory` + `CanBusDyn` blanket impl |
| 同步/异步冲突 | 同时有 `recv()` 和 `recv_async()` | 仅保留 `recv() -> impl Future` |
| ClassicFrame 效率 | `data: Vec<u8>` | `data: [u8; 8]` + `len: u8` |
| 缺少错误类型 | 未定义 `CanError` | 完整定义（8 种变体） |
| timestamp 格式 | `Option<Instant>` | `timestamp_us: Option<u64>` — 硬件友好 |

#### ⚠️ 新发现

**`CanError::Io(String)` 缺少 `#[from]` 转换**

各后端实现中到处是 `.map_err(|e| CanError::Io(format!(...)))`。建议至少提供 `From<std::io::Error>` 或 `impl From<&std::io::Error> for CanError`。

**`CanBusDyn` blanket impl 的 `Box::pin()` 开销**

每次 `recv()` 调用都分配一个 `Box`，PDO 高频场景下可能成为瓶颈。

### 12.4 canopen-core — 审查

#### ✅ 审计问题已修复

| 审计问题 | 实际实现 |
|---------|---------|
| FunctionCode 冲突 | 合并为 `SyncOrEmergency`，用 `is_sync()` / `is_emergency()` 区分 |
| OdError 未定义 | 完整定义（3 种变体） |
| ObjectType 被忽略 | EDS parser 正确解析 ObjectType |
| EntryInfo 未定义 | 完整定义 |

#### 🔴 P0 Bug：`SdoResponse::decode` segment size 位运算错误

```rust
// frame.rs:310 — SDO response segment size 解码
Some(7 - ((cmd >> 4) & 0x07))  // ← 错误：应该是 (cmd >> 1) & 0x07
```

SDO segment 的 size 在 bits 1-3（`cmd & 0x0E`）。正确提取是 `(cmd >> 1) & 0x07`。

**对比 stack.rs:247 中的正确实现**：

```rust
// stack.rs:247 — 正确的
let n = (cmd >> 1) & 0x07;
7 - n
```

`SdoResponse::decode` 从未被测试覆盖（集成测试通过 mock 直接构造 CanOpenFrame 而非通过 decode），所以这个 bug 没有被发现。**segmented upload 的 response 解码会返回错误的数据长度。**

### 12.5 canopen-ds301 — 审查

#### 🟡 P1：SDO 逻辑重复

`SdoClient::upload()` (sdo.rs) 和 `CanopenStack::sdo_upload()` (stack.rs) 有**几乎相同的实现**。`CanopenStack` 没有复用 `SdoClient`，而是重新实现了一遍 SDO 协议。

#### 🟡 P1：`HeartbeatTimeout` 事件重复报告

`stack.rs:119` — 每次 `process()` 都会检查所有节点超时。持续超时的节点每一条新帧都会产生新事件。

#### ✅ 审计建议已实现

- `scan_nodes()` 使用 SDO query 方式遍历 1-127
- `HeartbeatConsumer` 支持 per-node 超时配置
- `HeartbeatProducer` 不拥有 CAN driver

### 12.6 canopen-ds402 — 审查

#### ✅ StatusWord / ControlWord 正确实现

```rust
let bits = word & 0x006F;  // 0x006F = 0b0110_1111 — bits 0,1,2,3,5,6 正确
```

#### 🟢 P2：`Ds402Device::enable()` 不检查当前状态

硬编码三步序列（Shutdown → SwitchOn → EnableOperation）。如果设备已经在 OperationEnabled 状态，会发送不必要的降级命令。

#### ✅ `transition_to()` 部分实现

覆盖了主要的状态转换路径，但 NotReadyToSwitchOn 的转换缺失。

### 12.7 canopen-eds — 审查

#### ✅ ObjectType 正确解析

#### 🟢 P2：子条目 subindex 解析用十六进制

```rust
u8::from_str_radix(sub_str, 16)  // EDS 标准 subindex 是十进制
```

对 `sub0`-`sub9` 没问题，但 `sub10` 以上会将十进制 `10` 解析为 `0x10 = 16`。

### 12.8 硬件后端 — 审查

| 后端 | 状态 | 评价 |
|------|------|------|
| SocketCan | ✅ 完整 | tokio::sync::Mutex 正确，available_channels 扫描巧妙 |
| PCAN | ⚠️ stub | TODO 标注正确 |
| Kvaser | ⚠️ stub | TODO 标注正确 |
| ZLG | ⚠️ stub | TODO 标注正确 |

### 12.9 GUI — 审查

#### ✅ tokio + iced 集成方案正确

- 独立 tokio task 运行后端
- mpsc channel 双向通信
- 50ms 轮询事件

#### 🟡 oneshot channel 响应被丢弃

所有 `oneshot::channel()` 的 `rx` 都被丢弃（`_rx`）。GUI 依赖 `BackendEvent` 而非 oneshot 响应。oneshot 字段多余。

#### 🟢 `try_send` 静默丢弃命令

```rust
let _ = self.cmd_tx.try_send(cmd);  // 缓冲区满时无提示失败
```

#### 🟢 SocketCAN 连接未真正实现

`ConnectionConnect` 中 SocketCAN 分支只是设置 status message，不会真正建立连接。

### 12.10 缺失的测试

| 缺失测试 | 优先级 |
|---------|--------|
| `SdoResponse::decode` segment + abort 路径 | 🔴 P0（含已知 bug） |
| `CanDriverAdapter` frame 转换 roundtrip | 🟡 P1 |
| EDS subindex ≥ 10 解析 | 🟢 P2 |
| DS402 `transition_to()` 无效路径 | 🟡 P1 |
| `HeartbeatConsumer::check_timeouts()` | 🟡 P1 |
| SocketCan frame 转换 | 🟡 P1 |

### 12.11 代码质量评分

| 维度 | 评分 | 说明 |
|------|------|------|
| 编译清洁度 | ⭐⭐⭐⭐⭐ | clippy -D warnings 零警告 |
| 测试覆盖率 | ⭐⭐⭐ | 46 测试通过，但缺少 adapter/eds/backend 测试 |
| 错误处理 | ⭐⭐⭐⭐ | thiserror 正确，SDO abort code 完整映射 |
| 架构设计 | ⭐⭐⭐⭐ | trait 抽象合理，GUI-backend 桥接方案正确 |
| 代码复用 | ⭐⭐⭐ | SDO 逻辑重复 |
| 协议正确性 | ⭐⭐⭐⭐ | 一处位运算 bug 在 SdoResponse::decode |

### 12.12 优先级修复清单

| 优先级 | 问题 | 文件 | 影响 |
|--------|------|------|------|
| 🔴 P0 | `SdoResponse::decode` segment size 位运算错误 | `frame.rs:310` | segmented upload 解码错误 |
| 🟡 P1 | SDO 逻辑在 SdoClient 和 CanopenStack 中重复 | `sdo.rs` + `stack.rs` | 维护成本 |
| 🟡 P1 | HeartbeatTimeout 事件重复报告 | `stack.rs:119` | GUI 收到重复通知 |
| 🟡 P1 | CanError 在 canopen-core 和 can-traits 中重复 | `error.rs` | 类型转换冗余 |
| 🟢 P2 | Ds402Device::enable() 不检查当前状态 | `control.rs:81` | 不必要的降级命令 |
| 🟢 P2 | EDS subindex 解析用十六进制 | `parser.rs:88` | subindex ≥ 10 时错误 |
| 🟢 P2 | try_send 静默丢弃命令 | `backend.rs:106` | 高负载时命令丢失 |
| 🟢 P2 | oneshot channel 响应被丢弃 | `main.rs` 多处 | 代码冗余 |
