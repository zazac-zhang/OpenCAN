# Phase 12 设计文档 — Core 功能补全

> 日期: 2026-06-06
> 范围: SDO Server + PDO 配置 + SYNC 消费 + DS402 完整操作模式

## 1. 概述

Phase 12 补全 CANOpen 协议栈的 Core 功能，使 OpenCAN 从"能发起请求的调试工具"升级为"完整的 CANOpen 主站/从站模拟器"。

**依赖链：** SDO Server → PDO 配置 → SYNC 消费 → DS402 模式

## 2. SDO Server

### 目标

让协议栈能**响应**其他节点的 SDO 请求，读写本地 ObjectDictionary。实现设备模拟功能。

### 架构

```
CanopenStack
  ├── SdoClient (现有) — 发起 SDO 请求
  └── SdoServer (新增) — 响应 SDO 请求
         │
         ├── 收到 0x600+node_id 帧 → 解析为 SDO 请求
         ├── 读写本地 ObjectDictionary
         └── 发送 0x580+node_id 响应帧
```

### 类型定义

```rust
/// SDO Server — responds to SDO requests from other nodes.
pub struct SdoServer {
    /// Local object dictionary (shared with stack)
    od: Arc<Mutex<dyn ObjectDictionary>>,
    /// Our node ID (responds on 0x580 + node_id)
    node_id: u8,
}

impl SdoServer {
    pub fn new(od: Arc<Mutex<dyn ObjectDictionary>>, node_id: u8) -> Self;

    /// Process an incoming SDO request frame.
    /// Returns Some(response_frame) if this was an SDO request to us, None otherwise.
    pub fn process(&self, frame: &CanOpenFrame) -> Option<CanOpenFrame>;
}
```

### 支持的传输模式

| 模式 | CS (cmd) | 说明 |
|------|----------|------|
| **Expedited Download** | 0x23-0x2F | 客户端写 ≤4 字节到服务器 OD |
| **Expedited Upload** | 0x43-0x4F | 服务器返回 ≤4 字节 |
| **Segmented Download** | 0x21 + 0x00/0x10 | 客户端分段写 >4 字节 |
| **Segmented Upload** | 0x41 + 0x60/0x70 | 服务器分段返回 >4 字节 |
| **Block Download** | 0xC4 + 0xA2 | 批量写入（高速传输） |
| **Block Upload** | 0xC5 + 0xA2 | 批量读取（高速传输） |

### 请求处理流程

```rust
fn process(&self, frame: &CanOpenFrame) -> Option<CanOpenFrame> {
    // 1. 检查 COB-ID 是否是发给我们的 (0x600 + node_id)
    if frame.cob_id != 0x600 + self.node_id as u16 {
        return None;
    }

    // 2. 解析命令字节
    let cmd = frame.data[0];
    let index = u16::from_le_bytes([frame.data[1], frame.data[2]]);
    let subindex = frame.data[3];

    match cmd & 0xE0 {
        0x20 => self.handle_download(frame, index, subindex),
        0x40 => self.handle_upload(index, subindex),
        0x80 => None, // Abort from client — ignore
        _ => Some(self.abort(index, subindex, 0x0504_0001)), // CS not valid
    }
}
```

### Abort 码映射

复用现有 `sdo_abort_reason()` 函数，新增服务器端特有的 abort 码：

```rust
// 新增
0x0601_0001 => "Attempt to read a write only object",
0x0601_0002 => "Attempt to write a read only object",
0x0604_0041 => "Object cannot be mapped to the PDO",
0x0604_0042 => "Number and length of objects exceed PDO",
```

### 集成到 Stack

```rust
pub struct CanopenStack<C: CanDriver> {
    // ... 现有字段 ...
    od: Option<Arc<Mutex<ConcreteOd>>>,
    sdo_server: Option<SdoServer>,
}

impl<C: CanDriver> CanopenStack<C> {
    /// 启用 SDO Server，加载 ObjectDictionary
    pub fn enable_sdo_server(&mut self, od: ConcreteOd);

    /// 在 process() 中自动处理 SDO 请求
    fn process_sdo_server(&mut self, frame: &CanOpenFrame) -> Option<CanOpenFrame>;
}
```

### 测试策略

| 测试 | 方式 |
|------|------|
| Expedited download | MockCanDriver 发 0x603，验证 OD 写入 + 响应 |
| Expedited upload | MockCanDriver 发 upload 请求，验证响应数据 |
| Segmented download | 多帧序列，验证 OD 最终值 |
| Segmented upload | 验证分段响应帧序列 |
| Block transfer | 批量帧序列验证 |
| Abort 场景 | 只读写、对象不存在、类型不匹配 |
| 并发 SDO | SDO Client + Server 同时操作 |

## 3. PDO 配置管理

### 目标

通过 SDO 读写 PDO 通信参数和映射参数，GUI 层提供映射验证。

### OD 索引布局

```
通信参数 (Communication Parameters):
  0x1400 - RPDO1  0x1401 - RPDO2  0x1402 - RPDO3  0x1403 - RPDO4
  0x1800 - TPDO1  0x1801 - TPDO2  0x1802 - TPDO3  0x1803 - TPDO4

  Sub-indices:
    0: COB-ID (u32, bit31=valid flag)
    1: Transmission Type (u8)
    2: Inhibit Time (u16, 100μs units)
    3: Reserved
    5: Event Timer (u16, ms)

映射参数 (Mapping Parameters):
  0x1600 - RPDO1 Mapping  0x1601 - RPDO2  0x1602 - RPDO3  0x1603 - RPDO4
  0x1A00 - TPDO1 Mapping  0x1A01 - TPDO2  0x1A02 - TPDO3  0x1A03 - TPDO4

  Sub-indices:
    0: Number of Mapped Objects (u8)
    1..N: Mapping Entry (u32: Index:16 + Subindex:8 + BitLength:8)
```

### PdoConfigManager

```rust
/// PDO configuration manager — reads/writes PDO config via SDO.
pub struct PdoConfigManager<'a, C: CanDriver> {
    stack: &'a mut CanopenStack<C>,
}

impl<'a, C: CanDriver> PdoConfigManager<'a, C> {
    /// Read PDO communication parameters for a node.
    pub async fn read_comm_params(
        &mut self, node_id: u8, pdo_number: u8, direction: PdoDirection,
    ) -> Result<PdoCommParams, CanOpenError>;

    /// Read PDO mapping for a node.
    pub async fn read_mapping(
        &mut self, node_id: u8, pdo_number: u8, direction: PdoDirection,
    ) -> Result<Vec<PdoMappingEntry>, CanOpenError>;

    /// Write PDO mapping (must disable PDO first).
    pub async fn write_mapping(
        &mut self, node_id: u8, pdo_number: u8, direction: PdoDirection,
        mappings: &[PdoMappingEntry],
    ) -> Result<(), CanOpenError>;
}

pub struct PdoCommParams {
    pub cob_id: u32,
    pub transmission_type: u8,
    pub inhibit_time: Option<u16>,
    pub event_timer: Option<u16>,
}

pub struct PdoMappingEntry {
    pub index: u16,
    pub subindex: u8,
    pub bit_length: u8,
}
```

### GUI 映射验证（GUI 层实现）

```rust
/// Validate PDO mapping before writing.
fn validate_mapping(mappings: &[PdoMappingEntry]) -> Result<(), String> {
    // 1. 总位宽 ≤ 64
    let total_bits: u16 = mappings.iter().map(|m| m.bit_length as u16).sum();
    if total_bits > 64 {
        return Err(format!("Total bit length {} exceeds 64", total_bits));
    }

    // 2. 每个映射条目的 bit_length 必须是 8 的倍数
    for m in mappings {
        if m.bit_length % 8 != 0 {
            return Err(format!("Bit length {} is not a multiple of 8", m.bit_length));
        }
    }

    // 3. 对象存在性检查（需要 OD 查询）
    Ok(())
}
```

### 测试策略

| 测试 | 方式 |
|------|------|
| 读通信参数 | SDO upload 0x1400:00-05，验证 COB-ID、传输类型 |
| 读映射表 | SDO upload 0x1600:00-N，验证映射条目 |
| 写映射表 | SDO download 序列（禁用→修改→启用），验证 OD 值 |
| 映射验证 | GUI 层单元测试：位宽超限、非 8 倍数、对象不存在 |

## 4. SYNC 消费 + 同步 PDO

### 目标

处理接收到的 SYNC 帧，触发同步 PDO 传输。

### 同步传输类型

| Transmission Type | 含义 |
|-------------------|------|
| 0 | 非周期性同步 |
| 1 | 每个 SYNC 触发一次 |
| 2-240 | 每 N 个 SYNC 触发一次 |
| 252 | RTR-only 同步 |
| 253 | RTR-only 异步 |
| 254 | 事件驱动（厂商） |
| 255 | 事件驱动（协议） |

### SyncConsumer

```rust
/// SYNC consumer — tracks SYNC events and triggers synchronous PDOs.
pub struct SyncConsumer {
    /// Counter for received SYNCs
    sync_count: u32,
    /// PDOs with synchronous transmission type
    /// Maps (pdo_number, direction) → transmission_type
    sync_pdos: HashMap<(u8, PdoDirection), u8>,
}

impl SyncConsumer {
    pub fn new() -> Self;

    /// Register a PDO for synchronous triggering.
    pub fn register_pdo(&mut self, pdo_number: u8, direction: PdoDirection, trans_type: u8);

    /// Process a received SYNC frame.
    /// Returns the list of PDOs that should be transmitted on this SYNC.
    pub fn on_sync(&mut self) -> Vec<(u8, PdoDirection)>;

    /// Reset the SYNC counter.
    pub fn reset(&mut self);
}
```

### Stack 集成

```rust
impl<C: CanDriver> CanopenStack<C> {
    /// 在 process() 中处理 SYNC 帧
    fn process_sync(&mut self, frame: &CanOpenFrame) {
        if frame.cob_id == 0x080 {
            let triggered = self.sync_consumer.on_sync();
            for (pdo_num, dir) in triggered {
                // 触发对应的 PDO 传输
                self.events.push(CanEvent::SyncTriggered {
                    pdo_number: pdo_num,
                    direction: dir,
                });
            }
        }
    }
}
```

### CanEvent 扩展

```rust
pub enum CanEvent {
    // ... 现有变体 ...
    /// SYNC received — triggers synchronous PDOs
    SyncReceived { counter: u32 },
    /// PDO should be transmitted (sync-triggered)
    SyncTriggered { pdo_number: u8, direction: PdoDirection },
}
```

### 测试策略

| 测试 | 方式 |
|------|------|
| SYNC 计数 | 发送 N 个 SYNC 帧，验证 counter |
| Type=1 触发 | 每个 SYNC 都应触发 PDO |
| Type=N 触发 | 每 N 个 SYNC 触发一次 |
| 混合模式 | 同步 + 异步 PDO 共存 |
| RESET | 发送 NMT Reset 后 counter 归零 |

## 5. DS402 完整操作模式

### 目标

实现 CiA 402 定义的全部 7 种操作模式。

### 模块结构

```
crates/canopen-ds402/src/
├── lib.rs
├── state_machine.rs     ← 现有，DS402 状态机
├── control.rs           ← 现有，基础控制
├── modes/
│   ├── mod.rs
│   ├── pp.rs            ← Profile Position (模式 1)
│   ├── pv.rs            ← Profile Velocity (模式 3)
│   ├── pt.rs            ← Profile Torque (模式 4)
│   ├── homing.rs        ← Homing (模式 6)
│   ├── csp.rs           ← Cyclic Sync Position (模式 8)
│   ├── csv.rs           ← Cyclic Sync Velocity (模式 9)
│   └── cst.rs           ← Cyclic Sync Torque (模式 10)
└── feature.rs           ← Feature flags
```

### 每个模式的统一接口

```rust
/// Trait for DS402 operation modes.
pub trait OperationModeHandler {
    /// The mode identifier.
    fn mode(&self) -> OperationMode;

    /// Configure the mode (write OD parameters).
    async fn configure(&self, sdo: &mut SdoClient<impl CanDriver>, node_id: u8) -> Result<(), CanOpenError>;

    /// Set target value.
    async fn set_target(&self, sdo: &mut SdoClient<impl CanDriver>, node_id: u8, target: ModeTarget) -> Result<(), CanOpenError>;

    /// Read actual value.
    async fn read_actual(&self, sdo: &mut SdoClient<impl CanDriver>, node_id: u8) -> Result<ModeActual, CanOpenError>;
}
```

### 各模式 OD 映射

| 模式 | Target OD | Actual OD | 特殊参数 |
|------|-----------|-----------|----------|
| **PP** | 0x607A (Target Position) | 0x6064 (Actual Position) | 0x6081 (Profile Velocity), 0x6083 (Profile Accel) |
| **PV** | 0x60FF (Target Velocity) | 0x606C (Actual Velocity) | 0x6083 (Profile Accel) |
| **PT** | 0x6071 (Target Torque) | 0x6077 (Actual Torque) | 0x6087 (Torque Slope) |
| **Homing** | — | 0x6064 (Actual Position) | 0x6098 (Homing Method), 0x6099 (Homing Speed) |
| **CSP** | 0x607A (Target Position) | 0x6064 (Actual Position) | 每个 SYNC 更新 |
| **CSV** | 0x60FF (Target Velocity) | 0x606C (Actual Velocity) | 每个 SYNC 更新 |
| **CST** | 0x6071 (Target Torque) | 0x6077 (Actual Torque) | 每个 SYNC 更新 |

### Feature Flags

```toml
[features]
default = ["ds402"]
ds402 = []
ds402-pp = ["ds402"]
ds402-pv = ["ds402"]
ds402-pt = ["ds402"]
ds402-homing = ["ds402"]
ds402-csp = ["ds402"]
ds402-csv = ["ds402"]
ds402-cst = ["ds402"]
ds402-all = ["ds402-pp", "ds402-pv", "ds402-pt", "ds402-homing", "ds402-csp", "ds402-csv", "ds402-cst"]
```

### 测试策略

| 模式 | 测试内容 |
|------|----------|
| PP | 设置目标位置 → 读取实际位置 → 验证到位 |
| PV | 设置目标速度 → 验证速度写入 |
| PT | 设置目标力矩 → 验证力矩写入 |
| Homing | 启动回原点 → 验证状态转换 → 验证原点偏移 |
| CSP/CSV/CST | 每个 SYNC 更新目标值，验证 SDO 序列 |
| 全部 | 与 DS402 状态机集成：Disable → Enable → SetMode → SetTarget |

## 6. 实现计划

| 子阶段 | 功能 | 预计代码量 |
|--------|------|-----------|
| **12a** | SDO Server (expedited + segmented + block) | ~500 行 |
| **12b** | PDO 配置管理 | ~300 行 |
| **12c** | SYNC 消费 + 同步 PDO | ~250 行 |
| **12d** | DS402 全部操作模式 | ~800 行 |

**总计:** ~1850 行新增代码

## 7. 测试目标

| 指标 | 当前 | 目标 |
|------|------|------|
| 测试数量 | 53 | 80+ |
| canopen-ds301 测试 | 17+8 | 30+ |
| canopen-ds402 测试 | 3 | 15+ |
| Clippy 警告 | 0 | 0 |
| Doc 警告 | 0 | 0 |
