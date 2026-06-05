# OpenCAN 设计文档

> 日期: 2026-06-06
> 状态: Draft (v2 — 审查修订)

## 1. 概述

OpenCAN 是一个 CAN/CANOpen 调试工具 GUI 应用，支持：

- **CAN 硬件适配** — Linux SocketCAN、Kvaser、Peak PCAN、ZLG 致远电子
- **CANOpen 协议栈** — DS301 (NMT/SDO/PDO/EMCY/Heartbeat) + DS402 运动控制
- **GUI 界面** — 基于 iced 的跨平台桌面应用 (Windows + Linux)
- **纯 Rust 实现** — CANOpen 协议栈作为可独立发布的 crate，零 FFI

**设计原则：优先使用成熟库。**

## 2. 整体架构

```
OpenCAN (Cargo Workspace)
│
├── crates/
│   ├── can-traits/             ← 统一 CAN trait 抽象
│   ├── can-socketcan/          ← Linux SocketCAN 后端 (socketcan crate)
│   ├── can-kvaser/             ← Kvaser CANlib 后端 (can-hal-kvaser crate)
│   ├── can-pcan/               ← Peak PCAN 后端 (peak-can crate)
│   ├── can-zlg/                ← ZLG 致远后端 (zlgcan crate)
│   ├── canopen-core/           ← CANOpen 核心 traits + 对象字典 [no_std]
│   ├── canopen-ds301/          ← DS301 协议实现
│   ├── canopen-ds402/          ← DS402 运动控制
│   ├── canopen-eds/            ← EDS 文件解析 (可选)
│   └── gui/                    ← iced 主应用
└── vendor/                     ← (仅硬件 SDK 头文件/库文件)
```

## 3. 核心依赖选择

| 组件 | 选用库 | 版本 | 理由 |
|---|---|---|---|
| CAN 收发 (Linux) | `socketcan` | 3.5.0 | 最成熟的 Rust SocketCAN 库，async/tokio 支持。**注意**：GitHub 仓库叫 socketcan-rs/socketcan-rs，crates.io 上的 crate 名是 `socketcan` |
| CAN 收发 (Kvaser) | `can-hal-kvaser` | 0.4+ | 基于 can-hal-rs traits。**外部依赖**：用户需预先安装 Kvaser CANlib SDK (Windows: canlib32.dll, Linux: Kvaser 驱动) |
| CAN 收发 (PCAN) | `peak-can` 或 `pcanbasic` | latest | crates.io 上已有可用的 PCAN 绑定。**外部依赖**：用户需安装 PCAN-Basic SDK (Windows: PCANBasic.dll, Linux: libpcanbasic.so) |
| CAN 收发 (ZLG) | `zlgcan` | latest | rust-can 项目的一部分，跨平台 Windows+Linux。**外部依赖**：用户需安装 ZLG 致远电子 CAN 驱动 |
| CANOpen 协议栈 | 纯 Rust 自研 | — | Rust 生态无完整 DS301+DS402 方案。可参考 funcan-rs (SDO 编解码) 和 oze-canopen (异步架构) |
| EDS 解析 | `ini` crate + 自研模型 | — | EDS 是 INI 格式，解析简单 |
| GUI | `iced` | >= 0.14 | Elm 架构，原生异步支持 (Task/Subscription)，跨平台。**0.14 将 Command 重命名为 Task** |
| 异步运行时 | `tokio` | 1.x | socketcan 原生支持，生态最成熟 |
| 错误处理 | `thiserror` | 2.x | 成熟的 derive 宏，统一错误类型 |

**各后端外部依赖说明：**

所有 CAN 硬件后端都需要用户在系统上预先安装厂商提供的驱动/SDK。Rust crate 仅提供 FFI 绑定，不包含驱动本身。各厂商 SDK 的安装方式：

- **SocketCAN**：Linux 内核自带，无需额外安装
- **Kvaser**：[Kvaser SDK](https://www.kvaser.com/developer-downloads/)
- **PCAN**：[PCAN-Basic API](https://www.peak-system.com/PCAN-Basic.239.0.html)
- **ZLG**：致远电子 CAN 驱动 (随硬件附带或官网下载)

## 4. CAN Trait 抽象层

### 设计：两层 trait 体系

系统中存在两层 CAN 抽象，职责明确分离：

```
┌─────────────────────────────────────────────────┐
│  canopen-core: CanDriver                        │
│  - 协议栈内部使用的 CAN 帧接口                    │
│  - 操作 CanOpenFrame (COB-ID + 8字节 data)       │
│  - async recv/send                              │
├─────────────────────────────────────────────────┤
│  can-traits: CanBus                             │
│  - 硬件后端实现的物理层接口                       │
│  - 操作 CanFrame (Classic/FD, ID + data)         │
│  - 比特率、通道状态管理                           │
├─────────────────────────────────────────────────┤
│  canopen-ds301: CanDriverAdapter                │
│  - 桥接层：将 CanBus 包装为 CanDriver             │
│  - CanOpenFrame ↔ CanFrame 转换                  │
│  - COB-ID ↔ CAN ID 编解码                        │
└─────────────────────────────────────────────────┘
```

```rust
// canopen-ds301 中的适配器
pub struct CanDriverAdapter<B: CanBus> {
    bus: B,
}

impl<B: CanBus> CanDriver for CanDriverAdapter<B> {
    fn send(&mut self, frame: &CanOpenFrame) -> Result<(), CanOpenError> {
        let can_frame = self.canopen_to_can(frame);
        self.bus.send(&can_frame)?;
        Ok(())
    }

    fn recv(&mut self) -> Result<CanOpenFrame, CanOpenError> {
        let can_frame = self.bus.recv()?;
        self.can_to_canopen(&can_frame)
    }

    async fn recv_async(&mut self) -> Result<CanOpenFrame, CanOpenError> {
        let can_frame = self.bus.recv_async().await?;
        self.can_to_canopen(&can_frame)
    }
}
```

### CanBus — 物理层接口 (can-traits)

**支持 trait object (动态分发)**，以便 GUI 层根据用户选择动态切换后端：

```rust
pub trait CanBus: Send + Sync + 'static {
    fn send(&self, frame: &CanFrame) -> Result<(), CanError>;
    fn recv(&self) -> Result<CanFrame, CanError>;
    fn recv_async(&self) -> impl Future<Output = Result<CanFrame, CanError>> + Send;
    fn state(&self) -> CanState;
    fn set_bitrate(&self, bitrate: CanBitrate) -> Result<(), CanError>;
}

/// 工厂 trait — 用于动态创建后端实例
pub trait CanBusFactory: Send + Sync {
    fn open(&self, channel: &str, config: &CanConfig) -> Result<Box<dyn CanBusDyn>, CanError>;
    fn name(&self) -> &str;
    fn available_channels(&self) -> Vec<String>;
}

/// 动态分发版本 (GUI 层使用)
pub trait CanBusDyn: Send + Sync {
    fn send(&self, frame: &CanFrame) -> Result<(), CanError>;
    fn recv(&self) -> Result<CanFrame, CanError>;
    fn state(&self) -> CanState;
    fn set_bitrate(&self, bitrate: CanBitrate) -> Result<(), CanError>;
}

pub struct CanConfig {
    pub bitrate: CanBitrate,
    pub listen_only: bool,
    pub fd: bool,
}

pub enum CanFrame {
    Classic(ClassicFrame),    // CAN 2.0 (标准/扩展帧)
    Fd(FdFrame),              // CAN FD
}

pub struct ClassicFrame {
    pub id: CanId,
    pub data: Vec<u8>,        // max 8 bytes
    pub timestamp: Option<Instant>,
}

pub struct FdFrame {
    pub id: CanId,
    pub data: Vec<u8>,        // max 64 bytes
    pub flags: FdFlags,       // BRS, ESI
    pub timestamp: Option<Instant>,
}

pub enum CanId {
    Standard(u16),            // 11-bit
    Extended(u32),            // 29-bit
}

pub struct CanBitrate {
    pub nominal: u32,         // 仲裁段波特率 (如 500_000)
    pub data: Option<u32>,    // 数据段波特率 (CAN FD, 如 2_000_000)
}
```

### 各后端实现策略

| 后端 | 实现方式 | 外部依赖 | 成熟度 |
|---|---|---|---|
| **SocketCAN** | 包装 `socketcan` crate 的 `CanSocket` / `tokio::CanSocket` | Linux 内核自带 | ✅ 成熟 |
| **Kvaser** | 使用 `can-hal-kvaser` crate | 需安装 Kvaser CANlib SDK | ⚠️ 基础可用 |
| **PCAN** | 使用 `peak-can` 或 `pcanbasic` crate | 需安装 PCAN-Basic SDK | ✅ 现成方案 |
| **ZLG** | 使用 `zlgcan` crate (rust-can 项目) | 需安装 ZLG 致远驱动 | ✅ 现成方案 |

### Feature Flag 切换

```toml
# crates/gui/Cargo.toml
[features]
default = ["socketcan"]
socketcan = ["opencan-can-socketcan"]
kvaser = ["opencan-can-kvaser"]
pcan = ["opencan-can-pcan"]
zlg = ["opencan-can-zlg"]
eds = ["opencan-eds"]
```

## 5. CANOpen 协议栈 (纯 Rust)

### Crate 拆分

```
crates/
├── canopen-core/                 ← 核心 traits + 对象字典 [no_std], 零依赖
├── canopen-ds301/                ← DS301 协议实现 (NMT/SDO/PDO/EMCY/Heartbeat)
├── canopen-ds402/                ← DS402 运动控制 (状态机 + 操作模式)
└── canopen-eds/                  ← EDS 文件解析 (可选)
```

**依赖关系：**

```
canopen-ds402 → canopen-ds301 → canopen-core
canopen-eds   → (独立, 仅依赖标准库)
```

**参考实现：**
- **funcan-rs** — 参考其 SDO 编解码实现 (segmented/expedited transfer)
- **oze-canopen** — 参考其异步架构 (tokio + stream 模式)

### canopen-core — 核心 traits

```rust
// CAN 帧抽象 — 协议栈内部使用，仅支持 CAN 2.0 (8字节)
// CANOpen 协议基于 CAN 2.0，不使用 CAN FD
pub trait CanDriver: Send {
    fn send(&mut self, frame: &CanOpenFrame) -> Result<(), CanOpenError>;
    fn recv(&mut self) -> Result<CanOpenFrame, CanOpenError>;
    async fn recv_async(&mut self) -> Result<CanOpenFrame, CanOpenError>;
}

/// CANOpen 帧 — 固定 8 字节，对应 CAN 2.0 标准帧
/// CANOpen 协议规范基于 CAN 2.0A，数据段最大 8 字节
pub struct CanOpenFrame {
    pub cob_id: u16,      // COB-ID (11-bit, 包含 Function Code + Node ID)
    pub data: [u8; 8],    // 固定 8 字节 (CAN 2.0)
}

/// COB-ID 结构
pub struct CobId {
    pub function: FunctionCode,
    pub node_id: u8,  // 0-127
}

/// Function Code 枚举
/// 注意：Sync (0x080) 和 Emergency (0x080) 共享同一 Function Code，
/// 通过 Node ID 区分：Sync 使用 Node ID=0，Emergency 使用 Node ID=发送节点 ID
#[repr(u16)]
pub enum FunctionCode {
    Nmt       = 0x000,
    Sync      = 0x080,  // 与 Emergency 共享 FC，Node ID=0 时为 Sync
    Emergency = 0x080,  // 与 Sync 共享 FC，Node ID=节点ID 时为 Emergency
    Timestamp = 0x100,
    Tpdo1     = 0x180,
    Rpdo1     = 0x200,
    Tpdo2     = 0x280,
    Rpdo2     = 0x300,
    Tpdo3     = 0x380,
    Rpdo3     = 0x400,
    Tpdo4     = 0x480,
    Rpdo4     = 0x500,
    SdoServer = 0x580,  // SDO response (server → client)
    SdoClient = 0x600,  // SDO request (client → server)
    Heartbeat = 0x700,
}

// 对象字典
pub trait ObjectDictionary: Send {
    fn read(&self, index: u16, subindex: u8) -> Result<OdValue, OdError>;
    fn write(&mut self, index: u16, subindex: u8, value: OdValue) -> Result<(), OdError>;
    fn entry_info(&self, index: u16, subindex: u8) -> Result<EntryInfo, OdError>;
}

/// 对象字典条目类型 (DS301 Object Type)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectType {
    Var,      // 简单变量 (单个子索引 0)
    Array,    // 数组 (多个子索引，相同数据类型)
    Record,   // 记录 (多个子索引，可不同数据类型)
}

pub struct EntryInfo {
    pub index: u16,
    pub subindex: u8,
    pub object_type: ObjectType,
    pub data_type: DataType,
    pub access: AccessType,
    pub default_value: Option<OdValue>,
}

#[derive(Debug, Clone)]
pub enum OdValue {
    Boolean(bool),
    Integer8(i8), Integer16(i16), Integer32(i32), Integer64(i64),
    Unsigned8(u8), Unsigned16(u16), Unsigned32(u32), Unsigned64(u64),
    Real32(f32), Real64(f64),
    VisibleString(String),
    OctetString(Vec<u8>),
    Domain(Vec<u8>),
}
```

### canopen-ds301 — 协议实现

**SDO 客户端：**

```rust
pub struct SdoClient<C: CanDriver> {
    can: C,
    timeout: Duration,
}

impl<C: CanDriver> SdoClient<C> {
    pub async fn upload<T: OdEntry>(&mut self, node_id: u8, index: u16, subindex: u8) -> Result<T, SdoError>;
    pub async fn download<T: OdEntry>(&mut self, node_id: u8, index: u16, subindex: u8, value: T) -> Result<(), SdoError>;
}
```

**NMT 主站：**

```rust
pub struct NmtMaster;
impl NmtMaster {
    pub async fn start_remote_node(&self, node_id: u8) -> Result<()>;
    pub async fn stop_remote_node(&self, node_id: u8) -> Result<()>;
    pub async fn reset_node(&self, node_id: u8) -> Result<()>;
    pub async fn reset_communication(&self, node_id: u8) -> Result<()>;
}
```

**Heartbeat 消费者：**

```rust
pub struct HeartbeatConsumer;
impl HeartbeatConsumer {
    pub fn update(&mut self, hb: &HeartbeatFrame) -> bool;
    pub fn is_alive(&self, node_id: u8) -> bool;
    pub fn check_timeouts(&self) -> Vec<(u8, Duration)>;
}
```

**主循环：**

```rust
pub struct CanopenStack<C: CanDriver> { /* ... */ }

impl<C: CanDriver> CanopenStack<C> {
    pub async fn process(&mut self) -> Result<Vec<CanEvent>, CanOpenError>;
}

pub enum CanEvent {
    HeartbeatChanged { node_id: u8, alive: bool },
    HeartbeatTimeout { node_id: u8 },
    Emergency { node_id: u8, error_code: u16 },
    PdoReceived { pdo: PdoFrame },
    SdoComplete { request_id: u64, result: Result<OdValue, SdoError> },
}
```

**节点自动扫描：**

```rust
pub async fn scan_nodes<C: CanDriver>(stack: &mut CanopenStack<C>) -> Result<Vec<u8>, CanOpenError>;
```

### canopen-ds402 — 运动控制

**状态机：**

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ds402State {
    NotReadyToSwitchOn,
    SwitchOnDisabled,
    ReadyToSwitchOn,
    SwitchedOn,
    OperationEnabled,
    QuickStopActive,
    FaultReactionActive,
    Fault,
}

pub enum OperationMode {
    ProfilePosition,
    ProfileVelocity,
    ProfileTorque,
    CyclicSyncPosition,
    CyclicSyncVelocity,
    CyclicSyncTorque,
    Homing,
}
```

**设备接口：**

```rust
pub struct Ds402Device<C: CanDriver> {
    sdo: SdoClient<C>,
    node_id: u8,
}

impl<C: CanDriver> Ds402Device<C> {
    pub async fn state(&mut self) -> Result<Ds402State, Ds402Error>;
    pub async fn shutdown(&mut self) -> Result<()>;
    pub async fn switch_on(&mut self) -> Result<()>;
    pub async fn enable_operation(&mut self) -> Result<()>;
    pub async fn disable_voltage(&mut self) -> Result<()>;
    pub async fn quick_stop(&mut self) -> Result<()>;
    pub async fn fault_reset(&mut self) -> Result<()>;

    pub async fn set_mode(&mut self, mode: OperationMode) -> Result<()>;
    pub async fn set_target_position(&mut self, pos: i32) -> Result<()>;
    pub async fn actual_position(&mut self) -> Result<i32>;
    pub async fn set_target_velocity(&mut self, vel: i32) -> Result<()>;
    pub async fn actual_velocity(&mut self) -> Result<i32>;
    pub async fn set_target_torque(&mut self, tq: i16) -> Result<()>;
    pub async fn actual_torque(&mut self) -> Result<i16>;
}
```

**Crate 层级：DS402 不是独立 crate，而是 `canopen-ds402` crate，依赖 `canopen-ds301`。** 每个操作模式是独立文件，可通过 feature flag 选择：

```toml
[features]
default = ["ds402"]
ds402 = []
ds402-pp = ["ds402"]
ds402-pv = ["ds402"]
ds402-csp = ["ds402"]
```

### 错误处理

```rust
#[derive(Error, Debug)]
pub enum CanOpenError {
    #[error("CAN bus error: {0}")]
    Can(#[from] CanError),
    #[error("SDO abort: {code:#06x} - {reason}")]
    SdoAbort { code: u32, reason: &'static str },
    #[error("SDO timeout after {0:?}")]
    SdoTimeout(Duration),
    #[error("Object dictionary error: {0}")]
    Od(#[from] OdError),
    #[error("DS402 state transition invalid: {from:?} -> {to:?}")]
    Ds402InvalidTransition { from: Ds402State, to: Ds402State },
    #[error("DS402 fault: code={code:#06x}, register={register:#04x}")]
    Ds402Fault { code: u16, register: u8 },
}
```

SDO Abort Codes (DS301 Table 35) 完整映射已内置，可直接输出可读的错误原因。

## 6. EDS 解析 (可选)

**定位：可选辅助模块，不作为核心依赖。** 无 EDS 时用户可通过 SDO 手动读写，有 EDS 时自动填充 OD 描述。

```toml
# crates/gui/Cargo.toml
[features]
eds = ["opencan-eds"]  # 可选，不默认启用
```

**精简范围，保留对调试有用的字段，去掉不常用字段：**

```rust
pub struct EdsFile {
    pub file_info: FileInfo,
    pub device_info: DeviceInfo,
    pub dummy_usage: DummyUsage,
    pub entries: BTreeMap<u16, EdsEntry>,
    pub sub_entries: BTreeMap<(u16, u8), EdsSubEntry>,
}

pub struct EdsEntry {
    pub index: u16,
    pub subindex: u8,
    pub parameter_name: String,
    pub object_type: ObjectType,      // VAR / ARRAY / RECORD — 必须保留，影响 OD 构建逻辑
    pub data_type: Option<u16>,
    pub access_type: Option<AccessType>,
    pub default_value: Option<String>,
    pub pdo_mapping: Option<bool>,
}
```

**保留的字段**：ParameterName, ObjectType, DataType, AccessType, DefaultValue, PDOMapping

**忽略的字段**：LowLimit, HighLimit, ObjFlags, CompactPDO (DS302 扩展)

## 7. GUI (iced >= 0.14)

### Elm 架构

```rust
fn main() -> iced::Result {
    iced::application("OpenCAN", App::update, App::view)
        .subscription(App::subscription)
        .run()
}

// === 核心类型定义 ===

struct App {
    // 连接状态
    connection: Option<ActiveConnection>,
    can_backend: Option<Box<dyn CanBusDyn>>,

    // CANOpen 网络状态
    nodes: BTreeMap<u8, NodeState>,
    selected_node: Option<u8>,

    // UI 状态
    current_view: View,
    log_entries: VecDeque<LogEntry>,  // 环形缓冲区
    max_log_entries: usize,

    // DS402 面板状态
    ds402_panels: HashMap<u8, Ds402Panel>,
}

/// 活动连接信息
struct ActiveConnection {
    backend_name: String,     // "SocketCAN", "Kvaser", "PCAN", "ZLG"
    channel: String,          // "can0", "PCAN_USBBUS1", ...
    bitrate: CanBitrate,
    node_id: u8,              // 本节点 ID (主站通常 0)
    connected_at: Instant,
}

/// 节点状态
struct NodeState {
    node_id: u8,
    nmt_state: NmtState,
    last_heartbeat: Option<Instant>,
    device_type: Option<u32>,           // 0x1000
    vendor_id: Option<u32>,             // 0x1018sub1
    product_name: Option<String>,       // 0x1008
    od_cache: BTreeMap<(u16, u8), OdValue>,  // 已读取的 OD 值缓存
    eds_loaded: bool,                   // 是否加载了 EDS
}

/// 视图路由
#[derive(Debug, Clone, Copy, PartialEq)]
enum View {
    NetworkOverview,    // 网络概览
    NodeDetail,         // 节点详情
    Ds402,              // DS402 面板
    PdoMonitor,         // PDO 监控
    CanLog,             // CAN 日志
}

/// 日志条目
struct LogEntry {
    timestamp: Instant,
    direction: Direction,       // Tx / Rx
    cob_id: u16,
    data: [u8; 8],
    description: Option<String>, // 解码后的描述 (如 "SDO upload response")
}

enum Direction { Tx, Rx }

/// DS402 面板状态
struct Ds402Panel {
    node_id: u8,
    state: Ds402State,
    control_word: u16,
    status_word: u16,
    mode: OperationMode,
    target_position: i32,
    actual_position: i32,
    target_velocity: i32,
    actual_velocity: i32,
    target_torque: i16,
    actual_torque: i16,
    position_history: VecDeque<(Instant, i32)>,  // 用于绘制曲线
    velocity_history: VecDeque<(Instant, i32)>,
}
```

### 异步模型 — iced 0.14 Task/Subscription

**iced 0.14 使用 `Task` (原 `Command`) 和 `Subscription` 处理异步：**

```rust
// CAN 帧接收: iced subscription 对接 CAN 帧流
fn subscription(&self) -> iced::Subscription<Message> {
    match &self.connection {
        Some(conn) => {
            // subscription::channel 创建一个异步通道
            // 在内部使用 tokio runtime 接收 CAN 帧
            iced::Subscription::run_with_id(
                "can-frames",
                can_frame_stream(self.can_backend.as_ref().unwrap()),
            )
        }
        None => iced::Subscription::none(),
    }
}

/// 异步 CAN 帧流生成器
fn can_frame_stream(bus: &dyn CanBusDyn) -> impl Stream<Item = Message> {
    // 通过 tokio::sync::mpsc 桥接
    // iced 0.14 内部使用 futures executor，
    // 对于 tokio stream 需要通过 channel 转换
    futures::stream::unfold((), |()| async {
        // 实际实现中，这里从 mpsc receiver 接收帧
        // mpsc sender 在后台 tokio task 中由 CanBusDyn::recv_async 驱动
        todo!()
    })
}

// SDO 操作: iced 0.14 的 Task::perform 直接发起异步任务
fn update(&mut self, message: Message) -> iced::Task<Message> {
    match message {
        Message::SdoRead { node_id, index, subindex } => {
            let stack = self.stack.clone();
            // Task::perform 等价于旧版 Command::perform
            iced::Task::perform(
                sdo_upload(stack, node_id, index, subindex),
                Message::SdoReadResult,
            )
        }
        Message::SdoReadResult(Ok(value)) => {
            self.update_od_cache(value);
            iced::Task::none()
        }
        Message::SdoReadResult(Err(e)) => {
            self.show_error(e.to_string());
            iced::Task::none()
        }
        // ...
    }
}
```

**tokio ↔ iced 桥接说明：**

iced 0.14 内部使用 futures executor 而非 tokio runtime。对于依赖 tokio 的 CAN 后端：
- 在应用启动时创建全局 tokio runtime (`tokio::runtime::Runtime`)
- CAN 帧接收通过 `tokio::sync::mpsc` channel 发送到 iced subscription
- SDO 操作通过 `Task::perform` 在 tokio runtime 上执行，结果自动回传

```rust
// crates/gui/src/backend/tokio_bridge.rs

use tokio::sync::mpsc;
use tokio::runtime::Runtime;

/// 全局 tokio runtime (CAN 硬件操作需要)
static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime")
});

/// 将 CanBusDyn 的异步接收包装为 mpsc stream
pub fn spawn_can_receiver(
    bus: Box<dyn CanBusDyn>,
) -> mpsc::Receiver<CanOpenFrame> {
    let (tx, rx) = mpsc::channel(1024);

    RUNTIME.spawn(async move {
        loop {
            match bus.recv_async().await {
                Ok(frame) => {
                    if tx.send(frame).await.is_err() { break; }
                }
                Err(_) => { /* handle error */ }
            }
        }
    });

    rx
}
```

### 页面结构

| 页面 | 功能 |
|---|---|
| **网络概览** | 节点列表 + 状态卡片 + NMT 控制 |
| **节点详情** | OD 浏览器 + SDO 读写表单 |
| **DS402 面板** | 状态机图 + 控制按钮 + 实时数据曲线 |
| **PDO 监控** | PDO 实时表格 |
| **CAN 日志** | 帧日志 + 过滤/搜索 |

### 界面布局

```
┌─────────────────────────────────────────────────────────────┐
│  菜单栏:  连接  CANOpen  视图  工具  帮助                    │
├────────┬────────────────────────────────────────────────────┤
│        │  ┌─ 标签页 ──────────────────────────────────────┐ │
│ 侧边栏 │  │  网络概览  │  节点详情  │  PDO 监控  │  日志   │ │
│        │  ├───────────┴───────────┴──────────┴───────────┤ │
│ 硬件   │  │                                               │ │
│ 连接   │  │              主内容区域                         │ │
│ 状态   │  │                                               │ │
│ 节点   │  │                                               │ │
│ 列表   │  │                                               │ │
│        │  └───────────────────────────────────────────────┘ │
├────────┴────────────────────────────────────────────────────┤
│  状态栏:  硬件状态 │ 总线负载 │ 错误计数 │ 帧率              │
└─────────────────────────────────────────────────────────────┘
```

## 8. 测试策略

| 层 | 测试内容 | 方式 |
|---|---|---|
| 帧编解码 | SDO/PDO/NMT/Heartbeat/Emergency 编码解码 | 纯单元测试 |
| DS402 状态机 | 所有状态转换路径 + 异常路径 | 状态表驱动测试 |
| SDO 客户端 | expedited/segmented 传输、超时、abort | MockCanDriver |
| Heartbeat | 生产者/消费者、超时检测 | MockCanDriver + tokio::time |
| EDS 解析 | 各类 EDS 文件 + 边界情况 | 样本 EDS 文件 |
| 集成 | 完整 SDO 读写、NMT 控制流程 | MockCanDriver |
| 端到端 | GUI → 协议栈 → 硬件 | Linux vcan0 / 真实设备 |

**MockCanDriver：** 用于测试的 mock 实现，预置响应帧，记录发送帧。

## 9. 开发阶段

| Phase | 内容 | 预计时间 |
|---|---|---|
| **Phase 1** | canopen-core + can-traits + MockCanDriver + 单元测试 | 1-2 周 |
| **Phase 2** | canopen-ds301 (SDO/NMT/Heartbeat/EMCY) + CanDriverAdapter + 集成测试 | 2-3 周 |
| **Phase 3** | canopen-ds402 + canopen-eds | 1-2 周 |
| **Phase 4** | 硬件后端 (SocketCAN/PCAN/Kvaser/ZLG) + 端到端测试 | 3-4 周 |
| **Phase 5** | GUI (iced) + 全部页面 | 4-5 周 |

**说明：**
- Phase 4 预估增加 buffer：PCAN/ZLG 虽有现成 crate，但 FFI 调试 + Windows 跨平台编译仍有不确定性
- Phase 5 预估增加 buffer：iced 0.14 API 可能继续变动，tokio↔iced 桥接需要调试

## 10. CI/CD

```yaml
# .github/workflows/ci.yml
jobs:
  test:
    strategy:
      matrix:
        features: ["socketcan", "pcan", "kvaser", "zlg", ""]
    steps:
      - cargo test --workspace
      - cargo test --workspace --features ${{ matrix.features }}

  lint:
    steps:
      - cargo clippy --workspace -- -D warnings
      - cargo fmt --check

  vcan-test:   # Linux-only 端到端
    runs-on: ubuntu-latest
    steps:
      - sudo ip link add dev vcan0 type vcan
      - sudo ip link set up vcan0
      - cargo test --workspace --features socketcan -- --ignored
```
