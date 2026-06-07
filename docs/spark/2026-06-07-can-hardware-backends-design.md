# CAN 硬件后端集成设计

> 日期: 2026-06-07  
> 状态: 已批准  
> 范围: ZLG、Kvaser、PCAN 三家硬件后端的 FFI 实现

## 背景

OpenCAN 的 `can-traits` crate 定义了 `CanBus` trait 和 `CanBusFactory` trait，目前只有 SocketCAN 后端是完整实现的，ZLG、Kvaser、PCAN 三家都是 stub（返回 `Unsupported` 错误）。

目标是在 `can-traits` 内部直接实现这三家硬件后端的 FFI 绑定和 `CanBus` trait 实现，无需创建额外的 `-sys` crate。

## 硬件 SDK 概况

### ZLG (致远电子) — ControlCAN

- **Windows**: `ControlCAN.dll`
- **Linux**: `libcontrolcan.so`
- **头文件**: `ControlCAN.h`
- **API 模式**: 设备类型 + 设备索引 + 通道索引 三元组
- **关键函数**:
  - `VCI_OpenDevice(devtype, devindex, reserved)` → 打开设备
  - `VCI_CloseDevice(devtype, devindex)` → 关闭设备
  - `VCI_InitCAN(devtype, devindex, canindex, config)` → 初始化 CAN 通道
  - `VCI_StartCAN(devtype, devindex, canindex)` → 启动 CAN 通道
  - `VCI_Transmit(devtype, devindex, canindex, msg, count)` → 发送帧
  - `VCI_Receive(devtype, devindex, canindex, msg, count, timeout)` → 接收帧（阻塞）
  - `VCI_ReadBoardInfo(devtype, devindex, info)` → 读取设备信息

### Kvaser — CANlib

- **Windows**: `canlib32.dll`
- **Linux**: `libcanlib.so`
- **头文件**: `canlib.h`
- **API 模式**: handle 句柄
- **关键函数**:
  - `canInitializeLibrary()` → 初始化库
  - `canOpenChannel(channel, flags)` → 打开通道，返回 handle
  - `canSetBusParams(handle, bitrate, tseg1, tseg2, sjw, noSamp)` → 设置波特率
  - `canWrite(handle, id, data, dlc, flags)` → 发送帧
  - `canRead(handle, id, data, dlc, flags, time)` → 接收帧（阻塞）
  - `canClose(handle)` → 关闭通道
  - `canGetChannelData(channel, item, buf, bufsize)` → 获取通道信息

### Peak — PCAN-Basic

- **Windows**: `PCANBasic.dll`
- **Linux**: `libpcanbasic.so`
- **头文件**: `PCANBasic.h`
- **API 模式**: TPCANHandle 句柄
- **关键函数**:
  - `CAN_Initialize(channel, baudrate, hwtype, ioport, interrupt)` → 初始化
  - `CAN_Uninitialize(channel)` → 反初始化
  - `CAN_Read(channel, msg, timestamp)` → 读取帧（阻塞）
  - `CAN_Write(channel, msg)` → 写入帧
  - `CAN_GetValue(channel, parameter, buffer, buffer_length)` → 获取参数
  - `CAN_SetValue(channel, parameter, buffer, buffer_length)` → 设置参数

## 设计方案

### 目录结构

```
crates/can-traits/
├── build.rs              # bindgen 编译脚本，处理头文件
├── headers/              # 厂商头文件（用户自行放置或通过环境变量指定路径）
│   ├── ControlCAN.h      # ZLG
│   ├── canlib.h           # Kvaser
│   └── PCANBasic.h        # PCAN
├── src/
│   ├── lib.rs             # 模块导出
│   ├── error.rs           # 错误类型
│   ├── socketcan.rs       # 已有实现
│   ├── zlg.rs             # ZLG FFI + CanBus 实现
│   ├── kvaser.rs          # Kvaser FFI + CanBus 实现
│   └── pcan.rs            # PCAN FFI + CanBus 实现
```

### 头文件管理策略

**推荐方式**: 使用 `libloading` 运行时动态加载，无需编译时头文件。

每个后端在运行时加载动态库并缓存函数指针：

```rust
use libloading::Library;

mod ffi {
    use libloading::Library;
    
    pub struct ZlgFunctions {
        pub VCI_OpenDevice: unsafe extern "C" fn(u32, u32, u32) -> i32,
        pub VCI_CloseDevice: unsafe extern "C" fn(u32, u32) -> i32,
        pub VCI_InitCAN: unsafe extern "C" fn(u32, u32, u32, *const VCI_INIT_CONFIG) -> i32,
        pub VCI_StartCAN: unsafe extern "C" fn(u32, u32, u32) -> i32,
        pub VCI_Transmit: unsafe extern "C" fn(u32, u32, u32, *const VCI_CAN_OBJ, u32) -> i32,
        pub VCI_Receive: unsafe extern "C" fn(u32, u32, u32, *mut VCI_CAN_OBJ, i32, i32) -> i32,
    }
    
    impl ZlgFunctions {
        pub unsafe fn load(lib: &Library) -> Result<Self, libloading::Error> {
            Ok(Self {
                VCI_OpenDevice: *lib.get(b"VCI_OpenDevice")?,
                VCI_CloseDevice: *lib.get(b"VCI_CloseDevice")?,
                VCI_InitCAN: *lib.get(b"VCI_InitCAN")?,
                VCI_StartCAN: *lib.get(b"VCI_StartCAN")?,
                VCI_Transmit: *lib.get(b"VCI_Transmit")?,
                VCI_Receive: *lib.get(b"VCI_Receive")?,
            })
        }
    }
}
```

这样用户只需安装 SDK 并确保动态库在系统路径中，无需配置头文件路径。

### 每个后端的实现模式

以 ZLG 为例：

```rust
// crates/can-traits/src/zlg.rs

//! ZLG (致远电子) CAN 后端
//! 
//! 需要安装 ZLG CAN 驱动，并设置 ZLG_SDK_DIR 环境变量指向 SDK 目录。

#[cfg(feature = "zlg")]
mod zlg_impl {
    use crate::{CanBus, CanBusDyn, CanBusFactory, CanConfig, CanFrame, CanId, CanState, CanBitrate, ClassicFrame, error::CanError};
    use std::future::Future;
    use std::sync::Mutex;

    // ========== FFI 绑定层 ==========
    
    mod ffi {
        use std::os::raw::{c_char, c_int, c_uint, c_ushort, c_ulong};
        
        // 设备类型常量
        pub const VCI_USBCAN2: c_uint = 4;
        pub const VCI_USBCAN_2E_U: c_uint = 21;
        
        // 数据结构
        #[repr(C)]
        pub struct VCI_INIT_CONFIG {
            pub acc_code: c_ulong,
            pub acc_mask: c_ulong,
            pub mode: c_uchar,
            pub timing0: c_uchar,
            pub timing1: c_uchar,
        }
        
        #[repr(C)]
        pub struct VCI_CAN_OBJ {
            pub id: c_uint,
            pub time_stamp: c_uint,
            pub time_flag: c_uchar,
            pub send_type: c_uchar,
            pub remote_flag: c_uchar,
            pub extern_flag: c_uchar,
            pub data_len: c_uchar,
            pub data: [c_uchar; 8],
            pub reserved: [c_uchar; 3],
        }
        
        extern "C" {
            pub fn VCI_OpenDevice(devtype: c_uint, devindex: c_uint, reserved: c_uint) -> c_int;
            pub fn VCI_CloseDevice(devtype: c_uint, devindex: c_uint) -> c_int;
            pub fn VCI_InitCAN(devtype: c_uint, devindex: c_uint, canindex: c_uint, config: *const VCI_INIT_CONFIG) -> c_int;
            pub fn VCI_StartCAN(devtype: c_uint, devindex: c_uint, canindex: c_uint) -> c_int;
            pub fn VCI_Transmit(devtype: c_uint, devindex: c_uint, canindex: c_uint, msg: *const VCI_CAN_OBJ, count: c_uint) -> c_int;
            pub fn VCI_Receive(devtype: c_uint, devindex: c_uint, canindex: c_uint, msg: *mut VCI_CAN_OBJ, count: c_int, timeout: c_int) -> c_int;
            pub fn VCI_ReadBoardInfo(devtype: c_uint, devindex: c_uint, info: *mut VCI_BOARD_INFO) -> c_int;
            pub fn VCI_GetReceiveNum(devtype: c_uint, devindex: c_uint, canindex: c_uint) -> c_uint;
            pub fn VCI_ClearBuffer(devtype: c_uint, devindex: c_uint, canindex: c_uint) -> c_int;
        }
    }

    // ========== 错误处理 ==========
    
    fn zlg_error(code: i32) -> CanError {
        match code {
            0 => CanError::BusError("Operation failed".to_string()),
            1 => CanError::Io("Invalid device type".to_string()),
            2 => CanError::InvalidConfig("Invalid parameter".to_string()),
            // ... 更多错误码映射
            _ => CanError::BusError(format!("ZLG error code: {}", code)),
        }
    }

    // ========== CanBus 实现 ==========
    
    /// ZLG CAN 总线实例
    pub struct ZlgBus {
        device_type: u32,
        device_index: u32,
        channel: u32,
        config: Mutex<CanConfig>,
    }
    
    impl ZlgBus {
        pub fn open(device_type: u32, device_index: u32, channel: u32, config: &CanConfig) -> Result<Self, CanError> {
            // 打开设备
            let ret = unsafe { ffi::VCI_OpenDevice(device_type, device_index, 0) };
            if ret != 1 {
                return Err(zlg_error(ret));
            }
            
            // 初始化 CAN 通道
            let init_config = ffi::VCI_INIT_CONFIG {
                acc_code: 0,
                acc_mask: 0xFFFFFFFF,
                mode: 0,
                timing0: config.timing0,
                timing1: config.timing1,
            };
            
            let ret = unsafe { ffi::VCI_InitCAN(device_type, device_index, channel, &init_config) };
            if ret != 1 {
                unsafe { ffi::VCI_CloseDevice(device_type, device_index) };
                return Err(zlg_error(ret));
            }
            
            // 启动 CAN 通道
            let ret = unsafe { ffi::VCI_StartCAN(device_type, device_index, channel) };
            if ret != 1 {
                unsafe { ffi::VCI_CloseDevice(device_type, device_index) };
                return Err(zlg_error(ret));
            }
            
            Ok(Self {
                device_type,
                device_index,
                channel,
                config: Mutex::new(config.clone()),
            })
        }
    }
    
    impl Drop for ZlgBus {
        fn drop(&mut self) {
            unsafe {
                ffi::VCI_CloseDevice(self.device_type, self.device_index);
            }
        }
    }
    
    impl CanBus for ZlgBus {
        fn send(&self, frame: &CanFrame) -> Result<(), CanError> {
            let classic = match frame {
                CanFrame::Classic(f) => f,
                CanFrame::Fd(_) => return Err(CanError::Unsupported("CAN FD not supported by ZLG".to_string())),
            };
            
            let msg = ffi::VCI_CAN_OBJ {
                id: match classic.id {
                    CanId::Standard(id) => id as u32,
                    CanId::Extended(id) => id | 0x80000000,  // 扩展帧标志
                },
                time_stamp: 0,
                time_flag: 0,
                send_type: 1,  // 正常发送
                remote_flag: 0,
                extern_flag: if matches!(classic.id, CanId::Extended(_)) { 1 } else { 0 },
                data_len: classic.len,
                data: classic.data,
                reserved: [0; 3],
            };
            
            let ret = unsafe {
                ffi::VCI_Transmit(self.device_type, self.device_index, self.channel, &msg, 1)
            };
            
            if ret != 1 {
                return Err(zlg_error(ret));
            }
            Ok(())
        }
        
        fn recv(&self) -> impl Future<Output = Result<CanFrame, CanError>> + Send {
            let device_type = self.device_type;
            let device_index = self.device_index;
            let channel = self.channel;
            
            async move {
                // 使用 spawn_blocking 包装阻塞的 recv
                let frame = tokio::task::spawn_blocking(move || {
                    let mut msg = ffi::VCI_CAN_OBJ {
                        id: 0, time_stamp: 0, time_flag: 0, send_type: 0,
                        remote_flag: 0, extern_flag: 0, data_len: 0,
                        data: [0; 8], reserved: [0; 3],
                    };
                    
                    // timeout = -1 表示无限等待
                    let ret = unsafe {
                        ffi::VCI_Receive(device_type, device_index, channel, &mut msg, 1, -1)
                    };
                    
                    if ret <= 0 {
                        return Err(CanError::Io("ZLG receive failed".to_string()));
                    }
                    
                    let id = if msg.extern_flag == 1 {
                        CanId::Extended(msg.id & 0x1FFFFFFF)
                    } else {
                        CanId::Standard((msg.id & 0x7FF) as u16)
                    };
                    
                    Ok(CanFrame::Classic(ClassicFrame {
                        id,
                        data: msg.data,
                        len: msg.data_len,
                        timestamp_us: Some(msg.time_stamp as u64 * 1000), // 转换为微秒
                    }))
                }).await.map_err(|e| CanError::Io(format!("Task join error: {}", e)))?;
                
                frame
            }
        }
        
        fn state(&self) -> CanState {
            // ZLG SDK 没有直接的状态查询接口
            // 可以尝试读取缓冲区数量来判断是否正常
            CanState::Active
        }
        
        fn set_bitrate(&self, bitrate: CanBitrate) -> Result<(), CanError> {
            // 需要重新初始化 CAN 通道
            let (timing0, timing1) = match bitrate {
                CanBitrate::Rate125K => (0x03, 0x1C),
                CanBitrate::Rate250K => (0x01, 0x1C),
                CanBitrate::Rate500K => (0x00, 0x1C),
                CanBitrate::Rate1000K => (0x00, 0x14),
                _ => return Err(CanError::Unsupported("Unsupported bitrate".to_string())),
            };
            
            // 停止 -> 重新初始化 -> 启动
            // ... 实现细节
            Ok(())
        }
    }

    // ========== CanBusFactory 实现 ==========
    
    pub struct ZlgFactory;
    
    impl CanBusFactory for ZlgFactory {
        fn open(&self, channel: &str, config: &CanConfig) -> Result<Box<dyn CanBusDyn>, CanError> {
            // 解析 channel 字符串，格式: "USBCAN2:0:0" (设备类型:设备索引:通道)
            let parts: Vec<&str> = channel.split(':').collect();
            if parts.len() != 3 {
                return Err(CanError::InvalidConfig(
                    "ZLG channel format: <device_type>:<device_index>:<channel>".to_string()
                ));
            }
            
            let device_type: u32 = parts[0].parse().map_err(|_| CanError::InvalidConfig("Invalid device type".to_string()))?;
            let device_index: u32 = parts[1].parse().map_err(|_| CanError::InvalidConfig("Invalid device index".to_string()))?;
            let channel_idx: u32 = parts[2].parse().map_err(|_| CanError::InvalidConfig("Invalid channel".to_string()))?;
            
            let bus = ZlgBus::open(device_type, device_index, channel_idx, config)?;
            Ok(Box::new(bus))
        }
        
        fn name(&self) -> &str {
            "ZLG"
        }
        
        fn available_channels(&self) -> Vec<String> {
            // ZLG SDK 没有标准的设备枚举接口
            // 可以尝试打开常见设备类型来检测
            Vec::new()
        }
    }
}

#[cfg(feature = "zlg")]
pub use zlg_impl::{ZlgBus, ZlgFactory};
```

### Kvaser 实现要点

```rust
// kvaser.rs
mod ffi {
    extern "C" {
        pub fn canInitializeLibrary() -> c_int;
        pub fn canOpenChannel(channel: c_int, flags: c_int) -> c_int;  // 返回 handle
        pub fn canSetBusParams(handle: c_int, freq: c_long, tseg1: c_int, tseg2: c_int, sjw: c_int, noSamp: c_int) -> c_int;
        pub fn canWrite(handle: c_int, id: c_ulong, msg: *const c_void, dlc: c_uint, flags: c_uint) -> c_int;
        pub fn canRead(handle: c_int, id: *mut c_ulong, msg: *mut c_void, dlc: *mut c_uint, flags: *mut c_uint, time: *mut c_ulong) -> c_int;
        pub fn canReadWait(handle: c_int, id: *mut c_ulong, msg: *mut c_void, dlc: *mut c_uint, flags: *mut c_uint, time: *mut c_ulong, timeout: c_long) -> c_int;
        pub fn canClose(handle: c_int) -> c_int;
    }
}

// Kvaser 使用 handle 模式
pub struct KvaserBus {
    handle: Mutex<c_int>,
}

// recv 使用 canReadWait (带超时的阻塞读取)
```

### PCAN 实现要点

```rust
// pcan.rs
mod ffi {
    pub type TPCANHandle = u16;
    pub type TPCANBaudrate = u16;
    
    pub const PCAN_USBBUS1: TPCANHandle = 0x51;
    pub const PCAN_USBBUS2: TPCANHandle = 0x52;
    pub const PCAN_BAUD_500K: TPCANBaudrate = 0x001C;
    
    #[repr(C)]
    pub struct TPCANMsg {
        pub id: u32,
        pub msgtype: u8,  // 标准/扩展/远程帧
        pub len: u8,
        pub data: [u8; 8],
    }
    
    extern "C" {
        pub fn CAN_Initialize(channel: TPCANHandle, baudrate: TPCANBaudrate, hwtype: u8, ioport: u32, interrupt: u16) -> u32;
        pub fn CAN_Uninitialize(channel: TPCANHandle) -> u32;
        pub fn CAN_Read(channel: TPCANHandle, msg: *mut TPCANMsg, timestamp: *mut TPCANTimestamp) -> u32;
        pub fn CAN_Write(channel: TPCANHandle, msg: *const TPCANMsg) -> u32;
    }
}

pub struct PcanBus {
    handle: TPCANHandle,
}
```

### Cargo.toml 配置

```toml
[package]
name = "opencan-can-traits"

[dependencies]
thiserror = { workspace = true }
tokio = { workspace = true }
socketcan = { version = "3.5", features = ["tokio"], optional = true }
libloading = { version = "0.8", optional = true }

[features]
socketcan = ["dep:socketcan"]
kvaser = ["dep:libloading"]
pcan = ["dep:libloading"]
zlg = ["dep:libloading"]
```

### 平台差异处理

```rust
// zlg.rs
#[cfg(target_os = "windows")]
const LIB_NAME: &str = "ControlCAN";

#[cfg(target_os = "linux")]  
const LIB_NAME: &str = "controlcan";

// build.rs 中动态链接
#[cfg(feature = "zlg")]
{
    println!("cargo:rustc-link-lib=dylib={}", LIB_NAME);
}
```

### 测试策略

1. **单元测试**: 使用 mock 测试 CanFrame 转换逻辑
2. **集成测试**: 需要真实硬件，标记为 `#[ignore]`
3. **CI**: 仅编译检查，不运行需要硬件的测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frame_conversion() {
        // 测试 CanFrame <-> VCI_CAN_OBJ 转换
    }
    
    #[test]
    #[ignore] // 需要真实硬件
    fn test_zlg_send_recv() {
        // 测试实际发送接收
    }
}
```

## 实施计划

### Phase 1: ZLG 后端（优先）
1. 下载 ZLG SDK，获取 `ControlCAN.h`
2. 在 `zlg.rs` 中实现 FFI 绑定
3. 实现 `CanBus` trait
4. 实现 `CanBusFactory` trait
5. 编写测试

### Phase 2: PCAN 后端
1. 下载 PCAN-Basic API，获取 `PCANBasic.h`
2. 实现 FFI 绑定和 `CanBus` trait

### Phase 3: Kvaser 后端
1. 下载 Kvaser CANlib SDK，获取 `canlib.h`
2. 实现 FFI 绑定和 `CanBus` trait

## 风险和注意事项

1. **线程安全**: C SDK 通常不是线程安全的，必须用 Mutex 保护
2. **阻塞 recv**: 需要用 `spawn_blocking` 包装，避免阻塞 tokio 运行时
3. **动态库路径**: 用户需要正确安装 SDK 并配置库路径
4. **错误码**: 每家 SDK 错误码不同，需要仔细映射
5. **CAN FD**: 当前三家 SDK 主要支持 Classic CAN，CAN FD 支持需要额外 API

## 关键发现（来自调研）

### 线程安全差异

| 后端 | 原生安全性 | OpenCAN 策略 |
|------|-----------|-------------|
| ZLG | ❌ 不安全 | `std::sync::Mutex<Handle>` 全保护 |
| Kvaser | ✅ 按 handle 安全 | `std::sync::Mutex` 保护 send+recv |
| PCAN | ✅ 完全安全 | 最小化锁定 |

### 阻塞接收模式

| 后端 | 阻塞方式 | 备注 |
|------|---------|------|
| ZLG | `VCI_Receive(WaitTime=-1)` | 无限等待 |
| Kvaser | `canReadWait(timeout)` | 带超时 |
| PCAN | `CAN_Read` 始终非阻塞 | Windows: `PCAN_RECEIVE_EVENT` + `WaitForSingleObject`; Linux: 需轮询 |

### 初始化复杂度

| 后端 | 初始化步骤 |
|------|----------|
| ZLG | `OpenDevice` → `InitCAN` → `StartCAN`（3步）|
| Kvaser | `canInitializeLibrary()` → `canOpenChannel` → `canSetBusParams` → `canBusOn` |
| PCAN | `CAN_Initialize`（1步，最简单）|

### 重要注意事项

1. **ZLG**: 波特率使用私有的 Timing0/Timing1 寄存器值，需要查表
2. **ZLG**: 错误约定是 0=失败, 1=成功（非标准错误码）
3. **Kvaser**: 必须先调用 `canInitializeLibrary()` 才能使用其他函数
4. **PCAN**: `CAN_Read` 始终非阻塞，Linux 下需要轮询实现阻塞
5. **PCAN**: USB 设备自动枚举为 USBBUS1..16，无需手动枚举

### 推荐：运行时动态链接（libloading）

使用 `libloading` crate 在运行时加载动态库，避免编译时依赖 SDK：

```rust
use libloading::Library;

struct ZlgBus {
    lib: Library,
    // 缓存的函数指针
}
```

优势：
- 编译时不需要安装 SDK
- 用户只需在运行时提供动态库
- 更灵活的错误处理

## 参考资料

- ZLG ControlCAN SDK 文档
- Kvaser CANlib SDK 文档  
- PCAN-Basic API 文档
- 现有 `socketcan.rs` 实现作为模式参考
- `noahridge/canlib-rs` — Kvaser Rust 绑定参考
- `timokroeger/pcan-basic-rs` — PCAN Rust 绑定参考
- `123zmz123/ZlgCanDriver` — ZLG 头文件来源
