# OpenCAN 开发者指南

本指南为 OpenCAN 项目贡献者和开发者提供详细的架构说明和开发指南。

---

## 📋 目录

- [项目架构](#项目架构)
- [Crate 结构](#crate-结构)
- [两层 Trait 系统](#两层-trait-系统)
- [前端架构](#前端架构)
- [添加硬件后端](#添加硬件后端)
- [添加协议功能](#添加协议功能)
- [添加前端功能](#添加前端功能)
- [测试策略](#测试策略)
- [性能优化](#性能优化)
- [调试技巧](#调试技巧)

---

## 🏗️ 项目架构

```
┌─────────────────────────────────────────────────────┐
│                    Tauri GUI                         │
│  ┌───────────────────────────────────────────────┐  │
│  │              Frontend (React)                 │  │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐        │  │
│  │  │  Pages  │ │Components│ │  Hooks  │        │  │
│  │  └────┬────┘ └────┬────┘ └────┬────┘        │  │
│  │       └───────────┼───────────┘              │  │
│  │                   │                          │  │
│  │              ┌────▼────┐                     │  │
│  │              │  Store  │ (Zustand)           │  │
│  │              └────┬────┘                     │  │
│  └───────────────────┼─────────────────────────┘  │
│                      │ Tauri IPC                   │
│  ┌───────────────────▼─────────────────────────┐  │
│  │            Tauri Backend (Rust)             │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐   │  │
│  │  │ Commands │ │  State   │ │ Channels │   │  │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘   │  │
│  │       └────────────┼────────────┘          │  │
│  │                    │                        │  │
│  │       ┌────────────▼────────────┐          │  │
│  │       │    CanopenStack         │          │  │
│  │       │    (canopen-core)       │          │  │
│  │       └────────────┬────────────┘          │  │
│  │                    │                        │  │
│  │       ┌────────────▼────────────┐          │  │
│  │       │    CanBus (can-traits)  │          │  │
│  │       └─────────────────────────┘          │  │
│  └─────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

---

## 📦 Crate 结构

### can-traits — CAN 硬件抽象层

**职责**：定义硬件无关的 CAN 接口

```rust
// 核心 Trait
pub trait CanBus: Send + Sync {
    async fn send(&self, frame: &CanFrame) -> Result<(), CanError>;
    async fn recv(&self) -> Result<CanFrame, CanError>;
}

pub trait CanBusFactory: Send + Sync {
    fn name(&self) -> &str;
    fn available_devices(&self) -> Vec<CanDevice>;
    fn open(&self, config: &CanConfig) -> Result<Box<dyn CanBusDyn>, CanError>;
}
```

**特性**：
- `socketcan` — Linux SocketCAN 支持
- `zlg` — ZLG 致远电子支持
- `kvaser` — Kvaser 支持
- `pcan` — Peak PCAN 支持

### canopen-core — DS301 标准协议

**职责**：实现 CANopen 基础协议

```
canopen-core/src/
├── frame.rs           # 帧编解码
├── od.rs              # ObjectDictionary trait
├── concrete_od.rs     # BTreeMap 实现
├── node_id.rs         # NodeId 类型
├── stack.rs           # CanopenStack 主循环
├── testing.rs         # MockCanDriver
├── eds/               # EDS 解析器
│   ├── parser.rs
│   ├── model.rs
│   └── builder.rs
└── protocol/
    ├── sdo/           # SDO 协议
    │   ├── client.rs
    │   ├── server.rs
    │   ├── enhanced_server.rs
    │   ├── recovery.rs
    │   └── abort.rs
    ├── pdo/           # PDO 协议
    │   ├── types.rs
    │   ├── config.rs
    │   ├── dynamic.rs
    │   ├── event.rs
    │   └── sync.rs
    ├── nmt/           # NMT 协议
    ├── heartbeat/     # Heartbeat/SYNC
    └── emcy/          # Emergency
```

**设计约束**：
- 独立可发布到 crates.io
- 主从站通用
- 嵌入式友好（no_std 兼容）
- 仅包含 DS301 标准功能

### canopen-master — 主站增强功能

**职责**：主站专用功能

```
canopen-master/src/
├── adapter.rs         # CanBus → CanDriver 桥接
├── node_manager.rs    # 节点管理
├── heartbeat_monitor.rs
├── nmt_state_machine.rs
├── emergency_handler.rs
└── sdo_multi_client.rs
```

### canopen-ds402 — DS402 运动控制

**职责**：DS402 设备配置文件

```
canopen-ds402/src/ds402/
├── state_machine.rs   # 状态机
├── control.rs         # Ds402Device API
├── error.rs           # 错误处理
├── mode_validator.rs  # 模式验证
├── pdo_templates.rs   # PDO 模板
└── modes/             # 操作模式
    ├── csp.rs
    ├── csv.rs
    ├── cst.rs
    ├── pp.rs
    ├── pv.rs
    ├── pt.rs
    └── homing.rs
```

---

## 🔗 两层 Trait 系统

```
┌─ canopen-core: CanDriver ──────────────────────┐
│  协议栈内部使用。操作 CanOpenFrame (COB-ID + 8B) │
├─ canopen-master: CanDriverAdapter<B: CanBus> ──┤
│  桥接层。CanOpenFrame ↔ CanFrame 编解码          │
├─ can-traits: CanBus + CanBusFactory ────────────┤
│  硬件后端实现。操作 CanFrame (Classic/FD)         │
└─────────────────────────────────────────────────┘
```

### CanDriver（协议栈内部）

```rust
// canopen-core 定义
pub trait CanDriver: Send + Sync {
    async fn send(&self, frame: &CanOpenFrame) -> Result<(), CanError>;
    async fn recv(&self) -> Result<CanOpenFrame, CanError>;
}
```

### CanBus（硬件接口）

```rust
// can-traits 定义
pub trait CanBus: Send + Sync {
    async fn send(&self, frame: &CanFrame) -> Result<(), CanError>;
    async fn recv(&self) -> Result<CanFrame, CanError>;
}
```

### CanDriverAdapter（桥接层）

```rust
// canopen-master 定义
pub struct CanDriverAdapter<B: CanBus> {
    bus: B,
}

impl<B: CanBus> CanDriver for CanDriverAdapter<B> {
    async fn send(&self, frame: &CanOpenFrame) -> Result<(), CanError> {
        let can_frame = frame.to_can_frame();
        self.bus.send(&can_frame).await
    }
    
    async fn recv(&self) -> Result<CanOpenFrame, CanError> {
        let can_frame = self.bus.recv().await?;
        CanOpenFrame::from_can_frame(&can_frame)
    }
}
```

---

## 🖥️ 前端架构

### 目录结构

```
frontend/src/
├── pages/              # 页面组件（按功能分组）
│   ├── CAN/            # CAN 相关
│   │   ├── FrameMonitor.tsx
│   │   └── FrameTransmit.tsx
│   ├── CANOpen/        # CANopen 相关
│   │   ├── Ds402Control.tsx
│   │   ├── SdoExplorer.tsx
│   │   ├── NetworkTopology.tsx
│   │   └── ...
│   ├── Recording/      # 录制回放
│   └── Settings/       # 设置
├── components/         # 通用组件
│   ├── ds402/          # DS402 组件
│   └── common/         # 通用 UI
├── hooks/              # 自定义 Hooks
│   ├── useCommands.ts  # Tauri IPC 命令
│   └── useKeyboardShortcuts.ts
├── lib/                # 工具库
│   ├── store.ts        # Zustand 状态管理
│   ├── tauri.ts        # Tauri IPC 封装
│   └── utils.ts        # 工具函数
└── types/              # TypeScript 类型
```

### 状态管理（Zustand）

```typescript
// store.ts
interface AppState {
  can: CanState;         // CAN 连接状态
  ds402: Ds402State;     // DS402 设备状态
  heartbeat: HeartbeatState;
  ui: UiState;           // UI 状态
  recording: RecordingState;
}
```

### Tauri IPC 命令

```typescript
// tauri.ts
export const connectCan = invoke('connect_can', ...);
export const disconnectCan = invoke('disconnect_can', ...);
export const sdoUpload = invoke('sdo_upload', ...);
export const sdoDownload = invoke('sdo_download', ...);
export const ds402Enable = invoke('ds402_enable', ...);
export const scanNodes = invoke('scan_nodes', ...);
export const nmtCommand = invoke('nmt_command', ...);
```

---

## ➕ 添加硬件后端

### 1. 创建后端文件

```rust
// can-traits/src/mydevice.rs
use crate::{CanBus, CanBusDyn, CanBusFactory, CanConfig, CanDevice, CanError, CanFrame};
use async_trait::async_trait;

pub struct MyDeviceBus {
    // 硬件句柄
}

#[async_trait]
impl CanBus for MyDeviceBus {
    async fn send(&self, frame: &CanFrame) -> Result<(), CanError> {
        // 实现发送
    }
    
    async fn recv(&self) -> Result<CanFrame, CanError> {
        // 实现接收
    }
}

pub struct MyDeviceFactory;

impl CanBusFactory for MyDeviceFactory {
    fn name(&self) -> &str {
        "MyDevice"
    }
    
    fn available_devices(&self) -> Vec<CanDevice> {
        // 枚举可用设备
    }
    
    fn open(&self, config: &CanConfig) -> Result<Box<dyn CanBusDyn>, CanError> {
        // 打开设备连接
    }
}
```

### 2. 注册后端

```rust
// can-traits/src/lib.rs
#[cfg(feature = "mydevice")]
pub mod mydevice;
```

```toml
# can-traits/Cargo.toml
[features]
mydevice = ["dep:mydevice-sys"]

[dependencies]
mydevice-sys = { version = "0.1", optional = true }
```

### 3. 更新 CI

```yaml
# .github/workflows/ci.yml
- name: Build features
  matrix:
    feature: [socketcan, zlg, kvaser, pcan, mydevice]
```

---

## ➕ 添加协议功能

### 新增 SDO 功能

1. 在 `canopen-core/src/protocol/sdo/` 下修改对应文件
2. 添加新的方法到 `SdoClient` 或 `SdoServer`
3. 使用 `MockCanDriver` 编写单元测试

```rust
// 示例：添加 SDO block upload
impl SdoClient {
    pub async fn block_upload(
        &mut self,
        index: u16,
        subindex: u8,
    ) -> Result<Vec<u8>, SdoError> {
        // 实现 block upload 协议
    }
}
```

### 新增操作模式

1. 在 `canopen-ds402/src/ds402/modes/` 下创建新文件
2. 实现操作模式 trait
3. 在 `Ds402Device` 中注册

```rust
// canopen-ds402/src/ds402/modes/my_mode.rs
pub struct MyMode;

impl OperationMode for MyMode {
    fn mode_code(&self) -> i8 {
        0x01
    }
    
    fn setup(&self, od: &mut impl ObjectDictionary) -> Result<(), Ds402Error> {
        // 配置模式参数
    }
    
    fn process(&self, status_word: u16) -> Option<u16> {
        // 处理状态字，返回控制字
    }
}
```

---

## ➕ 添加前端功能

### 添加新页面

1. 创建页面组件

```tsx
// frontend/src/pages/CANOpen/MyFeature.tsx
export function MyFeature() {
  return (
    <div>
      <h1>My Feature</h1>
      {/* 实现 */}
    </div>
  );
}
```

2. 注册到 App.tsx

```tsx
// frontend/src/App.tsx
const MyFeature = lazy(() => 
  import('@/pages/CANOpen/MyFeature').then(m => ({ default: m.MyFeature }))
);

const TAB_COMPONENTS: Record<string, Record<string, React.ComponentType>> = {
  canopen: {
    // ...existing tabs
    MyFeature: MyFeature,
  },
};
```

3. 添加到 store

```tsx
// frontend/src/lib/store.ts
const GROUP_TABS = {
  canopen: [
    // ...existing tabs
    { key: 'MyFeature', label: 'My Feature' },
  ],
};
```

### 添加 Tauri 命令

1. 在后端添加命令

```rust
// opencan-gui/src-tauri/src/commands/my_module.rs
#[tauri::command]
pub async fn my_command(param: String) -> Result<String, String> {
    // 实现
}
```

2. 注册命令

```rust
// opencan-gui/src-tauri/src/commands/mod.rs
pub mod my_module;
pub use my_module::*;
```

3. 前端调用

```typescript
// frontend/src/lib/tauri.ts
export const myCommand = (param: string) => 
  invoke<string>('my_command', { param });
```

---

## 🧪 测试策略

### Rust 测试

```bash
# 运行所有测试
cargo test --workspace

# 运行特定 crate 测试
cargo test -p canopen-core

# 运行特定测试
cargo test test_sdo_upload

# 包含 socketcan 测试（需要 vcan0）
sudo modprobe vcan
sudo ip link add dev vcan0 type vcan
sudo ip link set up vcan0
OPENCAN_VCAN_TEST=1 cargo test --workspace --features socketcan -- --ignored
```

### MockCanDriver

```rust
use canopen_core::testing::MockCanDriver;

#[tokio::test]
async fn test_sdo_upload() {
    let mut driver = MockCanDriver::new();
    driver.expect_recv().returning(|| {
        Ok(CanOpenFrame::new_sdo_response(0x581, ...))
    });
    
    let mut client = SdoClient::new(NodeId::new(1).unwrap());
    let result = client.upload(&mut driver, 0x1000, 0).await;
    assert_eq!(result.unwrap(), vec![0x00, 0x00, 0x00, 0x00]);
}
```

### 前端测试

```bash
cd frontend

# 运行测试
npm test

# 监听模式
npm test -- --watch

# 覆盖率
npm test -- --coverage
```

---

## ⚡ 性能优化

### 代码分割

```tsx
// 使用 React.lazy 进行代码分割
const Ds402Control = lazy(() => 
  import('@/pages/CANOpen/Ds402Control')
);

// 使用 Suspense 包裹
<Suspense fallback={<Loading />}>
  <Ds402Control />
</Suspense>
```

### 状态优化

```tsx
// 使用 Zustand 选择器避免不必要的重渲染
const selectedNode = useAppStore((s) => s.can.selectedNode);

// 使用 React.memo 包裹纯组件
const NodeCard = React.memo(({ node }: NodeCardProps) => {
  // 实现
});
```

### 数据处理

```typescript
// 使用 Web Workers 处理大量数据
const worker = new Worker('data-processor.js');
worker.postMessage({ frames: largeFrameArray });
worker.onmessage = (e) => {
  // 处理结果
};
```

---

## 🐛 调试技巧

### Rust 调试

```bash
# 启用日志
RUST_LOG=debug cargo tauri dev

# 启用特定模块日志
RUST_LOG=opencan_canopen_core=debug cargo tauri dev

# 使用 tracing
tracing::debug!("SDO request: {:?}", request);
tracing::info!("Node {} connected", node_id);
tracing::error!("SDO timeout: {}", error);
```

### 前端调试

```tsx
// React DevTools
// 安装浏览器扩展

// Zustand DevTools
import { devtools } from 'zustand/middleware';
const useStore = create(devtools((set) => ({
  // ...
})));

// 控制台日志
console.log('State:', useAppStore.getState());
```

### Tauri 调试

```rust
// 后端日志
println!("Tauri command called: {:?}", args);

// 错误处理
#[tauri::command]
async fn my_command() -> Result<String, String> {
    // 错误会自动转换为 JS 异常
    do_something().map_err(|e| e.to_string())?
}
```

---

## 📚 参考资料

- [CANopen 规范](https://www.can-cia.org/)
- [CiA 301 基础协议](https://www.can-cia.org/canopen/specification/)
- [CiA 402 运动控制](https://www.can-cia.org/canopen/specification/)
- [Tauri 文档](https://tauri.app/)
- [React 文档](https://react.dev/)
- [Zustand 文档](https://github.com/pmndrs/zustand)
- [Tailwind CSS](https://tailwindcss.com/)
