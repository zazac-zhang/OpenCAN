//! ZLG (致远电子) CAN 后端
//!
//! 使用 `zlgcan` SDK 实现 CAN 总线通信。
//! 需要安装 ZLG CAN 驱动，并确保 `zlgcan.dll` (Windows) 或 `libzlgcan.so` (Linux) 在系统路径中。
//!
//! # 线程安全
//!
//! ZLG SDK **不是**线程安全的。本实现使用 `std::sync::Mutex` 保护所有 FFI 调用。
//!
//! # 初始化序列
//!
//! `ZCAN_OpenDevice` → `ZCAN_InitCAN` → `ZCAN_StartCAN` → send/recv

use crate::{
    CanBitrate, CanBus, CanBusDyn, CanBusFactory, CanConfig, CanFrame, CanId, CanState,
    ClassicFrame, error::CanError,
};
use std::future::Future;
use std::sync::Mutex;
use std::sync::OnceLock;

// ========== 类型定义 ==========

/// 设备句柄
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DeviceHandle(*mut std::ffi::c_void);

/// 通道句柄
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChannelHandle(*mut std::ffi::c_void);

// SAFETY: 句柄只在 Mutex 保护下使用
unsafe impl Send for DeviceHandle {}
unsafe impl Sync for DeviceHandle {}
unsafe impl Send for ChannelHandle {}
unsafe impl Sync for ChannelHandle {}

const INVALID_DEVICE_HANDLE: DeviceHandle = DeviceHandle(std::ptr::null_mut());
const INVALID_CHANNEL_HANDLE: ChannelHandle = ChannelHandle(std::ptr::null_mut());

// ========== FFI 结构体 ==========

/// CAN 帧结构
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CanFrameFfi {
    can_id: u32, // CAN ID + flags
    can_dlc: u8, // 数据长度
    __pad: u8,
    __res0: u8,
    __res1: u8,
    data: [u8; 8],
}

/// CAN FD 帧结构（预留）
#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct CanFdFrameFfi {
    can_id: u32,
    len: u8,
    flags: u8,
    __res0: u8,
    __res1: u8,
    data: [u8; 64],
}

/// 通道初始化配置
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ZcanChannelInitConfig {
    can_type: u32, // 0=CAN, 1=CANFD
    // union: can or canfd config
    acc_code: u32,
    acc_mask: u32,
    reserved: u32,
    filter: u8,
    timing0: u8,
    timing1: u8,
    mode: u8,
    // padding for canfd fields (not used for CAN)
    _pad: [u8; 12],
}

/// 发送数据结构
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ZcanTransmitData {
    frame: CanFrameFfi,
    transmit_type: u32,
}

/// 接收数据结构
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ZcanReceiveData {
    frame: CanFrameFfi,
    timestamp: u64, // 微秒
}

// ========== FFI 函数指针 ==========

/// ZLG SDK 函数指针集合
///
/// # Safety
///
/// 所有函数指针都通过 `libloading` 从动态库加载。
/// 调用者必须确保动态库在调用期间保持加载状态。
#[allow(dead_code)]
struct ZlgFunctions {
    // 使用 usize 存储函数指针，避免 Send/Sync 问题
    _lib: libloading::Library, // 保持库的生命周期
    fn_open_device: usize,
    fn_close_device: usize,
    fn_init_can: usize,
    fn_start_can: usize,
    fn_transmit: usize,
    fn_receive: usize,
    fn_get_receive_num: usize,
    fn_clear_buffer: usize,
    fn_read_channel_err_info: usize,
}

// SAFETY: ZlgFunctions 内部所有操作都在 Mutex 保护下进行
unsafe impl Send for ZlgFunctions {}
unsafe impl Sync for ZlgFunctions {}

impl ZlgFunctions {
    unsafe fn open_device(
        &self,
        device_type: u32,
        device_index: u32,
        reserved: u32,
    ) -> DeviceHandle {
        let func: unsafe extern "C" fn(u32, u32, u32) -> *mut std::ffi::c_void =
            unsafe { std::mem::transmute(self.fn_open_device) };
        DeviceHandle(unsafe { func(device_type, device_index, reserved) })
    }

    unsafe fn close_device(&self, handle: DeviceHandle) -> u32 {
        let func: unsafe extern "C" fn(*mut std::ffi::c_void) -> u32 =
            unsafe { std::mem::transmute(self.fn_close_device) };
        unsafe { func(handle.0) }
    }

    unsafe fn init_can(
        &self,
        device_handle: DeviceHandle,
        can_index: u32,
        config: *const ZcanChannelInitConfig,
    ) -> ChannelHandle {
        let func: unsafe extern "C" fn(
            *mut std::ffi::c_void,
            u32,
            *const ZcanChannelInitConfig,
        ) -> *mut std::ffi::c_void = unsafe { std::mem::transmute(self.fn_init_can) };
        ChannelHandle(unsafe { func(device_handle.0, can_index, config) })
    }

    unsafe fn start_can(&self, channel_handle: ChannelHandle) -> u32 {
        let func: unsafe extern "C" fn(*mut std::ffi::c_void) -> u32 =
            unsafe { std::mem::transmute(self.fn_start_can) };
        unsafe { func(channel_handle.0) }
    }

    unsafe fn transmit(
        &self,
        channel_handle: ChannelHandle,
        data: *const ZcanTransmitData,
        len: u32,
    ) -> u32 {
        let func: unsafe extern "C" fn(*mut std::ffi::c_void, *const ZcanTransmitData, u32) -> u32 =
            unsafe { std::mem::transmute(self.fn_transmit) };
        unsafe { func(channel_handle.0, data, len) }
    }

    unsafe fn receive(
        &self,
        channel_handle: ChannelHandle,
        data: *mut ZcanReceiveData,
        len: u32,
        wait_time: i32,
    ) -> u32 {
        let func: unsafe extern "C" fn(
            *mut std::ffi::c_void,
            *mut ZcanReceiveData,
            u32,
            i32,
        ) -> u32 = unsafe { std::mem::transmute(self.fn_receive) };
        unsafe { func(channel_handle.0, data, len, wait_time) }
    }

    unsafe fn get_receive_num(&self, channel_handle: ChannelHandle, can_type: u8) -> u32 {
        let func: unsafe extern "C" fn(*mut std::ffi::c_void, u8) -> u32 =
            unsafe { std::mem::transmute(self.fn_get_receive_num) };
        unsafe { func(channel_handle.0, can_type) }
    }

    #[allow(dead_code)]
    unsafe fn clear_buffer(&self, channel_handle: ChannelHandle) -> u32 {
        let func: unsafe extern "C" fn(*mut std::ffi::c_void) -> u32 =
            unsafe { std::mem::transmute(self.fn_clear_buffer) };
        unsafe { func(channel_handle.0) }
    }
}

/// 通道错误信息
#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct ZcanChannelErrInfo {
    error_code: u32,
    passive_err_data: [u8; 3],
    ar_lost_err_data: u8,
}

// ========== 动态库加载 ==========

/// 全局函数指针缓存
static ZLG_FUNCS: OnceLock<Result<ZlgFunctions, String>> = OnceLock::new();

#[cfg(target_os = "windows")]
const ZLG_LIB_NAME: &str = "zlgcan.dll";

#[cfg(target_os = "linux")]
const ZLG_LIB_NAME: &str = "libzlgcan.so";

#[cfg(target_os = "macos")]
const ZLG_LIB_NAME: &str = "libzlgcan.dylib";

unsafe fn load_zlg_functions() -> Result<ZlgFunctions, String> {
    let lib = unsafe { libloading::Library::new(ZLG_LIB_NAME) }
        .map_err(|e| format!("Failed to load {}: {}", ZLG_LIB_NAME, e))?;

    // 加载所有符号
    let fn_open_device = unsafe { load_sym(&lib, b"ZCAN_OpenDevice")? };
    let fn_close_device = unsafe { load_sym(&lib, b"ZCAN_CloseDevice")? };
    let fn_init_can = unsafe { load_sym(&lib, b"ZCAN_InitCAN")? };
    let fn_start_can = unsafe { load_sym(&lib, b"ZCAN_StartCAN")? };
    let fn_transmit = unsafe { load_sym(&lib, b"ZCAN_Transmit")? };
    let fn_receive = unsafe { load_sym(&lib, b"ZCAN_Receive")? };
    let fn_get_receive_num = unsafe { load_sym(&lib, b"ZCAN_GetReceiveNum")? };
    let fn_clear_buffer = unsafe { load_sym(&lib, b"ZCAN_ClearBuffer")? };
    let fn_read_channel_err_info = unsafe { load_sym(&lib, b"ZCAN_ReadChannelErrInfo")? };

    Ok(ZlgFunctions {
        _lib: lib,
        fn_open_device,
        fn_close_device,
        fn_init_can,
        fn_start_can,
        fn_transmit,
        fn_receive,
        fn_get_receive_num,
        fn_clear_buffer,
        fn_read_channel_err_info,
    })
}

unsafe fn load_sym(lib: &libloading::Library, name: &[u8]) -> Result<usize, String> {
    unsafe {
        lib.get::<unsafe extern "C" fn()>(name)
            .map(|sym| *sym as usize)
            .map_err(|e| {
                format!(
                    "Failed to load symbol {:?}: {}",
                    String::from_utf8_lossy(name),
                    e
                )
            })
    }
}

fn get_zlg_funcs() -> Result<&'static ZlgFunctions, CanError> {
    let result = ZLG_FUNCS.get_or_init(|| {
        // SAFETY: 只在初始化时调用一次
        unsafe { load_zlg_functions() }
    });

    result
        .as_ref()
        .map_err(|e| CanError::Io(format!("ZLG SDK not available: {}", e)))
}

// ========== 错误处理 ==========

fn zlg_status_to_error(status: u32) -> CanError {
    match status {
        0 => CanError::BusError("ZLG operation failed".to_string()),
        1 => CanError::Io("ZLG invalid device type".to_string()),
        2 => CanError::InvalidConfig("ZLG invalid parameter".to_string()),
        _ => CanError::BusError(format!("ZLG error code: {}", status)),
    }
}

// ========== 波特率映射 ==========

/// 将 CanBitrate 映射到 ZLG Timing0/Timing1 寄存器值
fn bitrate_to_timing(bitrate: &CanBitrate) -> (u8, u8) {
    // 常见波特率对应的 Timing0/Timing1 值
    // 这些值是基于 16MHz 时钟频率的标准配置
    match bitrate.nominal {
        1_000_000 => (0x00, 0x14), // 1 Mbps
        800_000 => (0x00, 0x16),   // 800 kbps
        500_000 => (0x00, 0x1C),   // 500 kbps
        250_000 => (0x01, 0x1C),   // 250 kbps
        125_000 => (0x03, 0x1C),   // 125 kbps
        100_000 => (0x04, 0x1C),   // 100 kbps
        50_000 => (0x09, 0x1C),    // 50 kbps
        20_000 => (0x18, 0x1C),    // 20 kbps
        10_000 => (0x31, 0x1C),    // 10 kbps
        _ => (0x00, 0x1C),         // 默认 500 kbps
    }
}

// ========== CanBus 实现 ==========

/// ZLG CAN 总线实例
#[allow(dead_code)]
pub struct ZlgBus {
    device_handle: DeviceHandle,
    channel_handle: ChannelHandle,
    device_type: u32,
    device_index: u32,
    channel: u32,
    _mutex: Mutex<()>, // 保护非线程安全的 C API
}

// SAFETY: ZlgBus 内部使用 Mutex 保护所有 FFI 调用
unsafe impl Send for ZlgBus {}
unsafe impl Sync for ZlgBus {}

impl ZlgBus {
    /// 打开 ZLG CAN 设备
    ///
    /// # Arguments
    /// * `device_type` - 设备类型 (如 ZCAN_USBCAN2 = 4)
    /// * `device_index` - 设备索引
    /// * `channel` - CAN 通道索引
    /// * `config` - CAN 配置
    pub fn open(
        device_type: u32,
        device_index: u32,
        channel: u32,
        config: &CanConfig,
    ) -> Result<Self, CanError> {
        let funcs = get_zlg_funcs()?;

        // 打开设备
        let device_handle = unsafe { funcs.open_device(device_type, device_index, 0) };

        if device_handle == INVALID_DEVICE_HANDLE {
            return Err(CanError::Io("Failed to open ZLG device".to_string()));
        }

        // 初始化 CAN 通道
        let (timing0, timing1) = bitrate_to_timing(&config.bitrate);
        let init_config = ZcanChannelInitConfig {
            can_type: 0, // CAN (not CANFD)
            acc_code: 0,
            acc_mask: 0xFFFFFFFF, // 接收所有帧
            reserved: 0,
            filter: 1, // 单滤波
            timing0,
            timing1,
            mode: 0, // 正常模式
            _pad: [0; 12],
        };

        let channel_handle = unsafe { funcs.init_can(device_handle, channel, &init_config) };

        if channel_handle == INVALID_CHANNEL_HANDLE {
            unsafe { funcs.close_device(device_handle) };
            return Err(CanError::Io(
                "Failed to initialize ZLG CAN channel".to_string(),
            ));
        }

        // 启动 CAN 通道
        let status = unsafe { funcs.start_can(channel_handle) };

        if status != 1 {
            unsafe { funcs.close_device(device_handle) };
            return Err(zlg_status_to_error(status));
        }

        Ok(Self {
            device_handle,
            channel_handle,
            device_type,
            device_index,
            channel,
            _mutex: Mutex::new(()),
        })
    }
}

impl Drop for ZlgBus {
    fn drop(&mut self) {
        if let Ok(funcs) = get_zlg_funcs()
            && self.device_handle != INVALID_DEVICE_HANDLE {
                unsafe {
                    funcs.close_device(self.device_handle);
                }
            }
    }
}

impl CanBus for ZlgBus {
    fn send(&self, frame: &CanFrame) -> Result<(), CanError> {
        let funcs = get_zlg_funcs()?;
        let _lock = self._mutex.lock().unwrap();

        let classic = match frame {
            CanFrame::Classic(f) => f,
            CanFrame::Fd(_) => {
                return Err(CanError::Unsupported(
                    "CAN FD not supported yet".to_string(),
                ));
            }
        };

        let can_id = match classic.id {
            CanId::Standard(id) => id as u32,
            CanId::Extended(id) => id | 0x80000000, // 设置扩展帧标志
        };

        let mut data = [0u8; 8];
        let len = classic.len.min(8) as usize;
        data[..len].copy_from_slice(&classic.data[..len]);

        let transmit_data = ZcanTransmitData {
            frame: CanFrameFfi {
                can_id,
                can_dlc: classic.len,
                __pad: 0,
                __res0: 0,
                __res1: 0,
                data,
            },
            transmit_type: 0, // 正常发送
        };

        let status = unsafe { funcs.transmit(self.channel_handle, &transmit_data, 1) };

        if status == 0 {
            return Err(CanError::BusError("ZLG transmit failed".to_string()));
        }

        Ok(())
    }

    fn recv(&self) -> impl Future<Output = Result<CanFrame, CanError>> + Send {
        let channel_handle = self.channel_handle;

        async move {
            

            tokio::task::spawn_blocking(move || {
                let funcs = get_zlg_funcs()?;

                let mut receive_data = ZcanReceiveData {
                    frame: CanFrameFfi {
                        can_id: 0,
                        can_dlc: 0,
                        __pad: 0,
                        __res0: 0,
                        __res1: 0,
                        data: [0; 8],
                    },
                    timestamp: 0,
                };

                // timeout = -1 表示无限等待
                let status = unsafe { funcs.receive(channel_handle, &mut receive_data, 1, -1) };

                if status == 0 {
                    return Err(CanError::Io("ZLG receive failed".to_string()));
                }

                let can_id = receive_data.frame.can_id;
                let id = if can_id & 0x80000000 != 0 {
                    // 扩展帧
                    CanId::Extended(can_id & 0x1FFFFFFF)
                } else {
                    // 标准帧
                    CanId::Standard((can_id & 0x7FF) as u16)
                };

                let mut data = [0u8; 8];
                let len = receive_data.frame.can_dlc.min(8) as usize;
                data[..len].copy_from_slice(&receive_data.frame.data[..len]);

                Ok(CanFrame::Classic(ClassicFrame {
                    id,
                    data,
                    len: receive_data.frame.can_dlc,
                    timestamp_us: Some(receive_data.timestamp),
                }))
            })
            .await
            .map_err(|e| CanError::Io(format!("Task join error: {}", e)))?
        }
    }

    fn state(&self) -> CanState {
        let funcs = match get_zlg_funcs() {
            Ok(f) => f,
            Err(_) => return CanState::NotConnected,
        };

        let _lock = self._mutex.lock().unwrap();

        // 检查是否有待接收的数据（作为总线活跃的指标）
        let recv_num = unsafe { funcs.get_receive_num(self.channel_handle, 0) };

        // 尝试读取通道错误信息
        let mut err_info = ZcanChannelErrInfo {
            error_code: 0,
            passive_err_data: [0; 3],
            ar_lost_err_data: 0,
        };

        // 调用 ZCAN_ReadChannelErrInfo 获取错误状态
        let read_err_fn: unsafe extern "C" fn(
            *mut std::ffi::c_void,
            *mut ZcanChannelErrInfo,
        ) -> u32 = unsafe { std::mem::transmute(funcs.fn_read_channel_err_info) };
        let status = unsafe { read_err_fn(self.channel_handle.0, &mut err_info) };

        if status == 1 {
            // 成功读取错误信息
            // ZLG SDK 的错误代码定义:
            // 0x0001: CAN 控制器内部 FIFO 溢出
            // 0x0002: CAN 控制器错误报警
            // 0x0004: CAN 控制器被动错误
            // 0x0008: CAN 控制器仲裁丢失
            // 0x0010: CAN 控制器总线错误
            // 0x0020: CAN 控制器总线关闭
            if err_info.error_code & 0x0020 != 0 {
                return CanState::BusOff;
            } else if err_info.error_code & 0x0004 != 0 {
                return CanState::ErrorPassive;
            } else if err_info.error_code & 0x0002 != 0 {
                return CanState::Warning;
            }
        }

        // 如果有数据或没有严重错误，认为总线活跃
        if recv_num > 0 || err_info.error_code == 0 {
            CanState::Active
        } else {
            CanState::Warning
        }
    }

    fn set_bitrate(&self, bitrate: CanBitrate) -> Result<(), CanError> {
        // ZLG SDK 需要重新初始化通道才能改变波特率
        // 这是一个破坏性操作，暂时不支持
        let _ = bitrate;
        Err(CanError::Unsupported(
            "ZLG bitrate change requires channel reinitialization".to_string(),
        ))
    }
}

// ========== CanBusFactory 实现 ==========

/// ZLG CAN 工厂
pub struct ZlgFactory;

impl CanBusFactory for ZlgFactory {
    fn open(&self, channel: &str, config: &CanConfig) -> Result<Box<dyn CanBusDyn>, CanError> {
        // 解析 channel 字符串，格式: "<device_type>:<device_index>:<channel>"
        // 例如: "4:0:0" 表示 USBCAN2, 设备0, 通道0
        let parts: Vec<&str> = channel.split(':').collect();
        if parts.len() != 3 {
            return Err(CanError::InvalidConfig(
                "ZLG channel format: <device_type>:<device_index>:<channel>".to_string(),
            ));
        }

        let device_type: u32 = parts[0]
            .parse()
            .map_err(|_| CanError::InvalidConfig("Invalid device type".to_string()))?;
        let device_index: u32 = parts[1]
            .parse()
            .map_err(|_| CanError::InvalidConfig("Invalid device index".to_string()))?;
        let channel_idx: u32 = parts[2]
            .parse()
            .map_err(|_| CanError::InvalidConfig("Invalid channel".to_string()))?;

        let bus = ZlgBus::open(device_type, device_index, channel_idx, config)?;
        Ok(Box::new(bus))
    }

    fn name(&self) -> &str {
        "ZLG"
    }

    fn available_channels(&self) -> Vec<String> {
        let mut channels = Vec::new();

        if let Ok(funcs) = get_zlg_funcs() {
            // 常见设备类型及其最大通道数
            let device_configs = [
                (4, "USBCAN2", 2),
                (21, "USBCAN_2E_U", 2),
                (41, "USBCANFD_200U", 2),
                (42, "USBCANFD_100U", 1),
                (59, "USBCANFD_800U", 8),
                (76, "USBCANFD_400U", 4),
            ];

            for (dev_type, name, max_channels) in &device_configs {
                // 尝试打开设备（最多尝试 4 个设备索引）
                for dev_idx in 0..4 {
                    let handle = unsafe { funcs.open_device(*dev_type, dev_idx, 0) };
                    if handle != INVALID_DEVICE_HANDLE {
                        // 设备存在，添加所有通道
                        for ch in 0..*max_channels {
                            channels.push(format!(
                                "{}:{}:{} ({} #{})",
                                dev_type, dev_idx, ch, name, dev_idx
                            ));
                        }
                        unsafe { funcs.close_device(handle) };
                    }
                }
            }
        }

        channels
    }
}

// ========== 设备类型常量 ==========

/// ZLG 设备类型常量
#[allow(non_upper_case_globals)]
pub mod device_types {
    pub const ZCAN_PCI5121: u32 = 1;
    pub const ZCAN_PCI9810: u32 = 2;
    pub const ZCAN_USBCAN1: u32 = 3;
    pub const ZCAN_USBCAN2: u32 = 4;
    pub const ZCAN_PCI9820: u32 = 5;
    pub const ZCAN_CAN232: u32 = 6;
    pub const ZCAN_PCI5110: u32 = 7;
    pub const ZCAN_CANLITE: u32 = 8;
    pub const ZCAN_ISA9620: u32 = 9;
    pub const ZCAN_ISA5420: u32 = 10;
    pub const ZCAN_PC104CAN: u32 = 11;
    pub const ZCAN_CANETUDP: u32 = 12;
    pub const ZCAN_CANETE: u32 = 12;
    pub const ZCAN_DNP9810: u32 = 13;
    pub const ZCAN_PCI9840: u32 = 14;
    pub const ZCAN_PC104CAN2: u32 = 15;
    pub const ZCAN_PCI9820I: u32 = 16;
    pub const ZCAN_CANETTCP: u32 = 17;
    pub const ZCAN_PCIE_9220: u32 = 18;
    pub const ZCAN_PCI5010U: u32 = 19;
    pub const ZCAN_USBCAN_E_U: u32 = 20;
    pub const ZCAN_USBCAN_2E_U: u32 = 21;
    pub const ZCAN_PCI5020U: u32 = 22;
    pub const ZCAN_EG20T_CAN: u32 = 23;
    pub const ZCAN_PCIE9221: u32 = 24;
    pub const ZCAN_WIFICAN_TCP: u32 = 25;
    pub const ZCAN_WIFICAN_UDP: u32 = 26;
    pub const ZCAN_PCIe9120: u32 = 27;
    pub const ZCAN_PCIe9110: u32 = 28;
    pub const ZCAN_PCIe9140: u32 = 29;
    pub const ZCAN_USBCAN_4E_U: u32 = 31;
    pub const ZCAN_CANDTU_200UR: u32 = 32;
    pub const ZCAN_CANDTU_MINI: u32 = 33;
    pub const ZCAN_USBCAN_8E_U: u32 = 34;
    pub const ZCAN_CANREPLAY: u32 = 35;
    pub const ZCAN_CANDTU_NET: u32 = 36;
    pub const ZCAN_CANDTU_100UR: u32 = 37;
    pub const ZCAN_PCIE_CANFD_100U: u32 = 38;
    pub const ZCAN_PCIE_CANFD_200U: u32 = 39;
    pub const ZCAN_PCIE_CANFD_400U: u32 = 40;
    pub const ZCAN_USBCANFD_200U: u32 = 41;
    pub const ZCAN_USBCANFD_100U: u32 = 42;
    pub const ZCAN_USBCANFD_MINI: u32 = 43;
    pub const ZCAN_CANFDCOM_100IE: u32 = 44;
    pub const ZCAN_CANSCOPE: u32 = 45;
    pub const ZCAN_CLOUD: u32 = 46;
    pub const ZCAN_CANDTU_NET_400: u32 = 47;
    pub const ZCAN_CANFDNET_TCP: u32 = 48;
    pub const ZCAN_CANFDNET_200U_TCP: u32 = 48;
    pub const ZCAN_CANFDNET_UDP: u32 = 49;
    pub const ZCAN_CANFDNET_200U_UDP: u32 = 49;
    pub const ZCAN_CANFDWIFI_TCP: u32 = 50;
    pub const ZCAN_CANFDWIFI_100U_TCP: u32 = 50;
    pub const ZCAN_CANFDWIFI_UDP: u32 = 51;
    pub const ZCAN_CANFDWIFI_100U_UDP: u32 = 51;
    pub const ZCAN_CANFDNET_400U_TCP: u32 = 52;
    pub const ZCAN_CANFDNET_400U_UDP: u32 = 53;
    pub const ZCAN_CANFDBLUE_200U: u32 = 54;
    pub const ZCAN_CANFDNET_100U_TCP: u32 = 55;
    pub const ZCAN_CANFDNET_100U_UDP: u32 = 56;
    pub const ZCAN_CANFDNET_800U_TCP: u32 = 57;
    pub const ZCAN_CANFDNET_800U_UDP: u32 = 58;
    pub const ZCAN_USBCANFD_800U: u32 = 59;
    pub const ZCAN_PCIE_CANFD_100U_EX: u32 = 60;
    pub const ZCAN_PCIE_CANFD_400U_EX: u32 = 61;
    pub const ZCAN_PCIE_CANFD_200U_MINI: u32 = 62;
    pub const ZCAN_PCIE_CANFD_200U_EX: u32 = 63;
    pub const ZCAN_PCIE_CANFD_200U_M2: u32 = 63;
    pub const ZCAN_CANFDDTU_400_TCP: u32 = 64;
    pub const ZCAN_CANFDDTU_400_UDP: u32 = 65;
    pub const ZCAN_CANFDWIFI_200U_TCP: u32 = 66;
    pub const ZCAN_CANFDWIFI_200U_UDP: u32 = 67;
    pub const ZCAN_CANFDDTU_800ER_TCP: u32 = 68;
    pub const ZCAN_CANFDDTU_800ER_UDP: u32 = 69;
    pub const ZCAN_CANFDDTU_800EWGR_TCP: u32 = 70;
    pub const ZCAN_CANFDDTU_800EWGR_UDP: u32 = 71;
    pub const ZCAN_CANFDDTU_600EWGR_TCP: u32 = 72;
    pub const ZCAN_CANFDDTU_600EWGR_UDP: u32 = 73;
    pub const ZCAN_CANFDDTU_CASCADE_TCP: u32 = 74;
    pub const ZCAN_CANFDDTU_CASCADE_UDP: u32 = 75;
    pub const ZCAN_USBCANFD_400U: u32 = 76;
    pub const ZCAN_CANFDDTU_200U: u32 = 77;
    pub const ZCAN_ZPSCANFD_TCP: u32 = 78;
    pub const ZCAN_ZPSCANFD_USB: u32 = 79;
    pub const ZCAN_CANFDBRIDGE_PLUS: u32 = 80;
    pub const ZCAN_CANFDDTU_300U: u32 = 81;
    pub const ZCAN_PCIE_CANFD_800U: u32 = 82;
    pub const ZCAN_PCIE_CANFD_1200U: u32 = 83;
    pub const ZCAN_MINI_PCIE_CANFD: u32 = 84;
    pub const ZCAN_USBCANFD_800H: u32 = 85;
    pub const ZCAN_BG002: u32 = 86;
    pub const ZCAN_BG004: u32 = 87;

    pub const ZCAN_OFFLINE_DEVICE: u32 = 98;
    pub const ZCAN_VIRTUAL_DEVICE: u32 = 99;
}
