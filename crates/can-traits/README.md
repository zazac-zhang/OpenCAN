# opencan-can-traits

CAN 总线 trait 抽象和硬件后端实现，为 OpenCAN 提供统一的 CAN 硬件访问接口。

## 特性

- **`CanBus` trait** — 统一的 CAN 总线 I/O 接口（trait-object 安全）
- **`CanBusFactory` trait** — 动态后端创建接口，GUI 用于运行时选择硬件
- **CAN 帧类型** — 支持 Classic CAN 和 CAN FD
- **硬件后端** — 多种 CAN 硬件支持（feature-gated）

## 后端支持

| Feature     | 后端         | 平台          | 状态 |
|-------------|-------------|---------------|------|
| `socketcan` | SocketCAN   | Linux         | ✅ 完整实现 |
| `zlg`       | ZLG (致远电子) | Windows/Linux | ✅ 完整实现 |
| `pcan`      | PCAN (Peak) | Windows/Linux | ✅ 完整实现 |
| `kvaser`    | Kvaser      | Windows/Linux | ✅ 完整实现 |

## 安装依赖

### ZLG (致远电子)

**Windows:**
1. 下载并安装 [ZLG CAN 驱动](http://www.zlg.cn/can/down/)
2. 确保 `zlgcan.dll` 在系统 PATH 中，或与可执行文件在同一目录

**Linux:**
1. 下载 ZLG Linux 驱动包
2. 安装 `libzlgcan.so` 到系统库路径（如 `/usr/local/lib`）
3. 运行 `sudo ldconfig` 更新库缓存

**macOS:**
- 当前不支持（ZLG 未提供 macOS 驱动）

### PCAN (Peak)

**Windows:**
1. 下载并安装 [PCAN-Basic API](https://www.peak-system.com/PCAN-Basic.239.0.html)
2. 确保 `PCANBasic.dll` 在系统 PATH 中

**Linux:**
```bash
# Ubuntu/Debian
sudo apt install libpcanbasic-dev

# 或从 Peak 官网下载 Linux 驱动包
```

**macOS:**
- 当前不支持（Peak 未提供 macOS 驱动）

### Kvaser

**Windows:**
1. 下载并安装 [Kvaser CANlib SDK](https://www.kvaser.com/download/)
2. 确保 `canlib32.dll` 在系统 PATH 中

**Linux:**
```bash
# Ubuntu/Debian
sudo apt install kvaser-canlib-dev

# 需要加载内核模块
sudo modprobe kvcommon
sudo modprobe kvpcidev  # 如果使用 PCI 设备
```

**macOS:**
- 当前不支持（Kvaser 未提供 macOS 驱动）

## 使用方法

### Feature Flags

在 `Cargo.toml` 中启用需要的后端：

```toml
[dependencies]
opencan-can-traits = { version = "0.1", features = ["zlg", "pcan"] }
```

### 基本用法

```rust
use opencan_can_traits::{
    CanBus, CanBusFactory, CanConfig, CanBitrate, CanFrame, CanId,
    zlg::ZlgFactory,
    pcan::PcanFactory,
    kvaser::KvaserFactory,
};

// 创建配置
let config = CanConfig {
    bitrate: CanBitrate::new(500_000),  // 500 kbps
    listen_only: false,
    fd: false,
};

// 使用 ZLG 设备
let zlg_factory = ZlgFactory;
// 格式: <device_type>:<device_index>:<channel>
// 常见设备类型: 4=USBCAN2, 21=USBCAN_2E_U, 41=USBCANFD_200U
let bus = zlg_factory.open("4:0:0", &config)?;

// 使用 PCAN 设备
let pcan_factory = PcanFactory;
// 格式: USBBUS1 到 USBBUS8
let bus = pcan_factory.open("USBBUS1", &config)?;

// 使用 Kvaser 设备
let kvaser_factory = KvaserFactory;
// 格式: 通道号 (0, 1, 2, ...)
let bus = kvaser_factory.open("0", &config)?;

// 发送帧
let frame = CanFrame::Classic(opencan_can_traits::ClassicFrame {
    id: CanId::Standard(0x123),
    data: [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
    len: 8,
    timestamp_us: None,
});
bus.send(&frame)?;

// 接收帧（异步）
let received = bus.recv().await?;
```

### 枚举可用设备

```rust
use opencan_can_traits::zlg::ZlgFactory;

let factory = ZlgFactory;
let channels = factory.available_channels();
println!("Available channels: {:?}", channels);
```

## 通道格式说明

### ZLG

格式: `<device_type>:<device_index>:<channel>`

- `device_type` — 设备类型编号（见 `zlg::device_types` 模块）
- `device_index` — 设备索引（同一类型的第几个设备，从 0 开始）
- `channel` — CAN 通道索引（从 0 开始）

常见设备类型:
- `4` — USBCAN2
- `21` — USBCAN_2E_U
- `41` — USBCANFD_200U
- `42` — USBCANFD_100U

示例: `"4:0:0"` 表示第一个 USBCAN2 设备的第 0 个通道

### PCAN

格式: `USBBUS<n>` 或 `<handle>`

- `USBBUS1` 到 `USBBUS16` — USB CAN 适配器
- 也可以直接使用数字句柄值

示例: `"USBBUS1"` 表示第一个 USB PCAN 设备

### Kvaser

格式: `<channel>`

- 通道号从 0 开始
- 可以使用 `available_channels()` 枚举可用通道

示例: `"0"` 表示第一个 Kvaser 设备

## 线程安全

| 后端 | 原生安全性 | 实现策略 |
|------|-----------|---------|
| SocketCAN | ✅ 安全 | `tokio::sync::Mutex` |
| ZLG | ❌ 不安全 | `std::sync::Mutex` 全保护 |
| PCAN | ✅ 安全 | 无额外锁定 |
| Kvaser | ✅ 按 handle 安全 | `std::sync::Mutex` 保护同一 handle |

## 错误处理

所有后端统一使用 `CanError` 错误类型:

```rust
pub enum CanError {
    BusOff,
    BusError(String),
    InterfaceNotFound(String),
    Timeout,
    Io(String),
    NotConnected,
    InvalidConfig(String),
    Unsupported(String),
}
```

## 技术细节

### 运行时动态链接

所有硬件后端使用 `libloading` crate 在运行时加载动态库，避免编译时依赖 SDK：

```rust
// 编译时不需要安装 SDK
// 运行时自动加载动态库
let lib = libloading::Library::new("libzlgcan.so")?;
```

### 异步接收

所有后端的 `recv()` 方法返回 `impl Future`，内部使用 `tokio::task::spawn_blocking` 包装阻塞的 C API 调用。

### 波特率映射

| 标称波特率 | ZLG Timing0/1 | PCAN 常量 | Kvaser 常量 |
|-----------|---------------|-----------|-------------|
| 1 Mbps    | 0x00, 0x14   | 0x0014    | -1          |
| 500 kbps  | 0x00, 0x1C   | 0x001C    | -2          |
| 250 kbps  | 0x01, 0x1C   | 0x011C    | -3          |
| 125 kbps  | 0x03, 0x1C   | 0x031C    | -4          |
| 100 kbps  | 0x04, 0x1C   | 0x432F    | -5          |
| 50 kbps   | 0x09, 0x1C   | 0x472F    | -7          |

## 已知限制

1. **CAN FD** — 当前仅支持 Classic CAN (8 字节)，CAN FD 支持待实现
2. **平台支持** — macOS 不支持（厂商未提供驱动）
3. **热插拔** — 不支持设备热插拔检测
4. **波特率修改** — 运行时修改波特率需要重新初始化通道

## 参考实现

- [noahridge/canlib-rs](https://github.com/noahridge/canlib-rs) — Kvaser Rust 绑定
- [timokroeger/pcan-basic-rs](https://github.com/timokroeger/pcan-basic-rs) — PCAN Rust 绑定
- [123zmz123/ZlgCanDriver](https://github.com/123zmz123/ZlgCanDriver) — ZLG 头文件

## License

MIT
