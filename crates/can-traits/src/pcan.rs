//! Peak PCAN-Basic 后端
//!
//! 使用 PCAN-Basic API 实现 CAN 总线通信。
//! 需要安装 PCAN 驱动，并确保 `PCANBasic.dll` (Windows) 或 `libpcanbasic.so` (Linux) 在系统路径中。
//!
//! # 线程安全
//!
//! PCAN-Basic API 是**线程安全**的，不需要额外的 Mutex 保护。
//!
//! # 初始化序列
//!
//! `CAN_Initialize` → send/recv（一步到位）

use crate::{CanBitrate, CanBus, CanBusDyn, CanBusFactory, CanConfig, CanFrame, CanId, CanState, ClassicFrame, error::CanError};
use std::future::Future;
use std::sync::OnceLock;

// ========== 类型定义 ==========

/// PCAN 句柄
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PcanHandle(u16);

/// PCAN 状态码
type PcanStatus = u32;

const PCAN_ERROR_OK: PcanStatus = 0x00000;
const PCAN_ERROR_QRCVEMPTY: PcanStatus = 0x00020;
const PCAN_ERROR_BUSOFF: PcanStatus = 0x00010;
const PCAN_ERROR_BUSHEAVY: PcanStatus = 0x00008;
const PCAN_ERROR_BUSLIGHT: PcanStatus = 0x00004;

/// 常用 PCAN 句柄
const PCAN_USBBUS1: PcanHandle = PcanHandle(0x51);
const PCAN_USBBUS2: PcanHandle = PcanHandle(0x52);
const PCAN_USBBUS3: PcanHandle = PcanHandle(0x53);
const PCAN_USBBUS4: PcanHandle = PcanHandle(0x54);
const PCAN_USBBUS5: PcanHandle = PcanHandle(0x55);
const PCAN_USBBUS6: PcanHandle = PcanHandle(0x56);
const PCAN_USBBUS7: PcanHandle = PcanHandle(0x57);
const PCAN_USBBUS8: PcanHandle = PcanHandle(0x58);

/// 常用波特率
const PCAN_BAUD_1M: u16 = 0x0014;
const PCAN_BAUD_500K: u16 = 0x001C;
const PCAN_BAUD_250K: u16 = 0x011C;
const PCAN_BAUD_125K: u16 = 0x031C;
const PCAN_BAUD_100K: u16 = 0x432F;
const PCAN_BAUD_50K: u16 = 0x472F;

/// 消息类型标志
const PCAN_MESSAGE_STANDARD: u8 = 0x00;
const PCAN_MESSAGE_EXTENDED: u8 = 0x02;

// ========== FFI 结构体 ==========

/// PCAN 消息结构
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct TPCANMsg {
    id: u32,
    msg_type: u8,
    len: u8,
    data: [u8; 8],
}

/// PCAN 时间戳结构
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct TPCANTimestamp {
    millis: u32,
    millis_overflow: u16,
    micros: u16,
}

// ========== FFI 函数指针 ==========

/// PCAN-Basic SDK 函数指针集合
#[allow(dead_code)]
struct PcanFunctions {
    _lib: libloading::Library,
    fn_initialize: usize,
    fn_uninitialize: usize,
    fn_read: usize,
    fn_write: usize,
    fn_get_value: usize,
    fn_set_value: usize,
    fn_get_error_text: usize,
}

// SAFETY: PCAN-Basic API 是线程安全的
unsafe impl Send for PcanFunctions {}
unsafe impl Sync for PcanFunctions {}

impl PcanFunctions {
    unsafe fn initialize(&self, channel: PcanHandle, baudrate: u16, hw_type: u8, io_port: u32, interrupt: u16) -> PcanStatus {
        let func: unsafe extern "C" fn(u16, u16, u8, u32, u16) -> PcanStatus =
            unsafe { std::mem::transmute(self.fn_initialize) };
        unsafe { func(channel.0, baudrate, hw_type, io_port, interrupt) }
    }

    unsafe fn uninitialize(&self, channel: PcanHandle) -> PcanStatus {
        let func: unsafe extern "C" fn(u16) -> PcanStatus =
            unsafe { std::mem::transmute(self.fn_uninitialize) };
        unsafe { func(channel.0) }
    }

    unsafe fn read(&self, channel: PcanHandle, msg: *mut TPCANMsg, timestamp: *mut TPCANTimestamp) -> PcanStatus {
        let func: unsafe extern "C" fn(u16, *mut TPCANMsg, *mut TPCANTimestamp) -> PcanStatus =
            unsafe { std::mem::transmute(self.fn_read) };
        unsafe { func(channel.0, msg, timestamp) }
    }

    unsafe fn write(&self, channel: PcanHandle, msg: *const TPCANMsg) -> PcanStatus {
        let func: unsafe extern "C" fn(u16, *const TPCANMsg) -> PcanStatus =
            unsafe { std::mem::transmute(self.fn_write) };
        unsafe { func(channel.0, msg) }
    }

    #[allow(dead_code)]
    unsafe fn get_error_text(&self, error: PcanStatus, language: u16, buffer: *mut u8, buffer_len: u32) -> PcanStatus {
        let func: unsafe extern "C" fn(PcanStatus, u16, *mut u8, u32) -> PcanStatus =
            unsafe { std::mem::transmute(self.fn_get_error_text) };
        unsafe { func(error, language, buffer, buffer_len) }
    }
}

// ========== 动态库加载 ==========

static PCAN_FUNCS: OnceLock<Result<PcanFunctions, String>> = OnceLock::new();

#[cfg(target_os = "windows")]
const PCAN_LIB_NAME: &str = "PCANBasic.dll";

#[cfg(target_os = "linux")]
const PCAN_LIB_NAME: &str = "libpcanbasic.so";

#[cfg(target_os = "macos")]
const PCAN_LIB_NAME: &str = "libpcanbasic.dylib";

unsafe fn load_pcan_functions() -> Result<PcanFunctions, String> {
    let lib = unsafe { libloading::Library::new(PCAN_LIB_NAME) }
        .map_err(|e| format!("Failed to load {}: {}", PCAN_LIB_NAME, e))?;

    let fn_initialize = unsafe { load_sym(&lib, b"CAN_Initialize")? };
    let fn_uninitialize = unsafe { load_sym(&lib, b"CAN_Uninitialize")? };
    let fn_read = unsafe { load_sym(&lib, b"CAN_Read")? };
    let fn_write = unsafe { load_sym(&lib, b"CAN_Write")? };
    let fn_get_value = unsafe { load_sym(&lib, b"CAN_GetValue")? };
    let fn_set_value = unsafe { load_sym(&lib, b"CAN_SetValue")? };
    let fn_get_error_text = unsafe { load_sym(&lib, b"CAN_GetErrorText")? };

    Ok(PcanFunctions {
        _lib: lib,
        fn_initialize,
        fn_uninitialize,
        fn_read,
        fn_write,
        fn_get_value,
        fn_set_value,
        fn_get_error_text,
    })
}

unsafe fn load_sym(lib: &libloading::Library, name: &[u8]) -> Result<usize, String> {
    unsafe {
        lib.get::<unsafe extern "C" fn()>(name)
            .map(|sym| *sym as usize)
            .map_err(|e| format!("Failed to load symbol {:?}: {}", String::from_utf8_lossy(name), e))
    }
}

fn get_pcan_funcs() -> Result<&'static PcanFunctions, CanError> {
    let result = PCAN_FUNCS.get_or_init(|| {
        unsafe { load_pcan_functions() }
    });

    result.as_ref().map_err(|e| CanError::Io(format!("PCAN-Basic not available: {}", e)))
}

// ========== 错误处理 ==========

fn pcan_status_to_error(status: PcanStatus) -> CanError {
    match status {
        PCAN_ERROR_OK => CanError::Io("PCAN operation succeeded but returned error".to_string()),
        PCAN_ERROR_QRCVEMPTY => CanError::Timeout,
        PCAN_ERROR_BUSOFF => CanError::BusOff,
        PCAN_ERROR_BUSHEAVY => CanError::BusError("PCAN bus heavy error".to_string()),
        PCAN_ERROR_BUSLIGHT => CanError::BusError("PCAN bus light error".to_string()),
        _ => CanError::BusError(format!("PCAN error code: 0x{:08X}", status)),
    }
}

// ========== 波特率映射 ==========

fn bitrate_to_pcan(bitrate: &CanBitrate) -> u16 {
    match bitrate.nominal {
        1_000_000 => PCAN_BAUD_1M,
        500_000 => PCAN_BAUD_500K,
        250_000 => PCAN_BAUD_250K,
        125_000 => PCAN_BAUD_125K,
        100_000 => PCAN_BAUD_100K,
        50_000 => PCAN_BAUD_50K,
        _ => PCAN_BAUD_500K,  // 默认 500 kbps
    }
}

// ========== 通道解析 ==========

fn parse_pcan_handle(channel: &str) -> Result<PcanHandle, CanError> {
    // 支持两种格式:
    // 1. "USBBUS1" 到 "USBBUS16"
    // 2. 数字形式的句柄值
    match channel {
        "USBBUS1" | "usb1" => Ok(PCAN_USBBUS1),
        "USBBUS2" | "usb2" => Ok(PCAN_USBBUS2),
        "USBBUS3" | "usb3" => Ok(PCAN_USBBUS3),
        "USBBUS4" | "usb4" => Ok(PCAN_USBBUS4),
        "USBBUS5" | "usb5" => Ok(PCAN_USBBUS5),
        "USBBUS6" | "usb6" => Ok(PCAN_USBBUS6),
        "USBBUS7" | "usb7" => Ok(PCAN_USBBUS7),
        "USBBUS8" | "usb8" => Ok(PCAN_USBBUS8),
        _ => {
            // 尝试解析为数字
            let handle: u16 = channel.parse()
                .map_err(|_| CanError::InvalidConfig(format!("Invalid PCAN channel: {}", channel)))?;
            Ok(PcanHandle(handle))
        }
    }
}

// ========== CanBus 实现 ==========

/// PCAN CAN 总线实例
pub struct PcanBus {
    handle: PcanHandle,
}

// SAFETY: PCAN-Basic API 是线程安全的
unsafe impl Send for PcanBus {}
unsafe impl Sync for PcanBus {}

impl PcanBus {
    /// 打开 PCAN 设备
    pub fn open(channel: &str, config: &CanConfig) -> Result<Self, CanError> {
        let funcs = get_pcan_funcs()?;
        let handle = parse_pcan_handle(channel)?;
        let baudrate = bitrate_to_pcan(&config.bitrate);

        // 初始化 PCAN 通道
        // USB 设备: hw_type=0, io_port=0, interrupt=0
        let status = unsafe {
            funcs.initialize(handle, baudrate, 0, 0, 0)
        };

        if status != PCAN_ERROR_OK {
            return Err(pcan_status_to_error(status));
        }

        Ok(Self { handle })
    }
}

impl Drop for PcanBus {
    fn drop(&mut self) {
        if let Ok(funcs) = get_pcan_funcs() {
            unsafe {
                funcs.uninitialize(self.handle);
            }
        }
    }
}

impl CanBus for PcanBus {
    fn send(&self, frame: &CanFrame) -> Result<(), CanError> {
        let funcs = get_pcan_funcs()?;

        let classic = match frame {
            CanFrame::Classic(f) => f,
            CanFrame::Fd(_) => return Err(CanError::Unsupported("CAN FD not supported yet".to_string())),
        };

        let (id, msg_type) = match classic.id {
            CanId::Standard(id) => (id as u32, PCAN_MESSAGE_STANDARD),
            CanId::Extended(id) => (id, PCAN_MESSAGE_EXTENDED),
        };

        let mut data = [0u8; 8];
        let len = classic.len.min(8) as usize;
        data[..len].copy_from_slice(&classic.data[..len]);

        let msg = TPCANMsg {
            id,
            msg_type,
            len: classic.len,
            data,
        };

        let status = unsafe { funcs.write(self.handle, &msg) };

        if status != PCAN_ERROR_OK {
            return Err(pcan_status_to_error(status));
        }

        Ok(())
    }

    fn recv(&self) -> impl Future<Output = Result<CanFrame, CanError>> + Send {
        let handle = self.handle;

        async move {
            // PCAN_Read 是非阻塞的，需要轮询实现阻塞接收
            let frame = tokio::task::spawn_blocking(move || {
                let funcs = get_pcan_funcs()?;

                loop {
                    let mut msg = TPCANMsg {
                        id: 0,
                        msg_type: 0,
                        len: 0,
                        data: [0; 8],
                    };
                    let mut timestamp = TPCANTimestamp {
                        millis: 0,
                        millis_overflow: 0,
                        micros: 0,
                    };

                    let status = unsafe {
                        funcs.read(handle, &mut msg, &mut timestamp)
                    };

                    if status == PCAN_ERROR_OK {
                        // 成功接收到帧
                        let id = if msg.msg_type & PCAN_MESSAGE_EXTENDED != 0 {
                            CanId::Extended(msg.id & 0x1FFFFFFF)
                        } else {
                            CanId::Standard((msg.id & 0x7FF) as u16)
                        };

                        let mut data = [0u8; 8];
                        let len = msg.len.min(8) as usize;
                        data[..len].copy_from_slice(&msg.data[..len]);

                        let timestamp_us = (timestamp.millis as u64) * 1000
                            + (timestamp.millis_overflow as u64) * 0x100000000 * 1000
                            + timestamp.micros as u64;

                        return Ok(CanFrame::Classic(ClassicFrame {
                            id,
                            data,
                            len: msg.len,
                            timestamp_us: Some(timestamp_us),
                        }));
                    } else if status == PCAN_ERROR_QRCVEMPTY {
                        // 接收队列为空，继续轮询
                        // 使用 sleep 避免 CPU 占用过高
                        std::thread::sleep(std::time::Duration::from_millis(1));
                        continue;
                    } else {
                        // 其他错误
                        return Err(pcan_status_to_error(status));
                    }
                }
            })
            .await
            .map_err(|e| CanError::Io(format!("Task join error: {}", e)))?;

            frame
        }
    }

    fn state(&self) -> CanState {
        // PCAN 没有简单的状态查询接口
        // 可以通过 CAN_GetValue 查询，但这里简化处理
        CanState::Active
    }

    fn set_bitrate(&self, bitrate: CanBitrate) -> Result<(), CanError> {
        // PCAN 需要重新初始化才能改变波特率
        let _ = bitrate;
        Err(CanError::Unsupported(
            "PCAN bitrate change requires channel reinitialization".to_string()
        ))
    }
}

// ========== CanBusFactory 实现 ==========

/// PCAN CAN 工厂
pub struct PcanFactory;

impl CanBusFactory for PcanFactory {
    fn open(&self, channel: &str, config: &CanConfig) -> Result<Box<dyn CanBusDyn>, CanError> {
        let bus = PcanBus::open(channel, config)?;
        Ok(Box::new(bus))
    }

    fn name(&self) -> &str {
        "PCAN"
    }

    fn available_channels(&self) -> Vec<String> {
        // PCAN USB 设备自动枚举为 USBBUS1..16
        // 返回可能的通道列表
        let mut channels = Vec::new();

        // 尝试初始化每个通道来检测是否存在
        if let Ok(funcs) = get_pcan_funcs() {
            for i in 1..=8 {
                let handle = PcanHandle(0x50 + i);
                let status = unsafe { funcs.initialize(handle, PCAN_BAUD_500K, 0, 0, 0) };
                if status == PCAN_ERROR_OK {
                    channels.push(format!("USBBUS{}", i));
                    unsafe { funcs.uninitialize(handle) };
                }
            }
        }

        channels
    }
}
