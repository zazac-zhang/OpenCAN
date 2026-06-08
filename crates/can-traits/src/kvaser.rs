//! Kvaser CANlib 后端
//!
//! 使用 Kvaser CANlib SDK 实现 CAN 总线通信。
//! 需要安装 Kvaser CANlib SDK，并确保 `canlib32.dll` (Windows) 或 `libcanlib.so` (Linux) 在系统路径中。
//!
//! # 线程安全
//!
//! Kvaser CANlib 是**按 handle 线程安全**的：同一个 handle 的并发调用需要外部同步。
//! 本实现使用 `std::sync::Mutex` 保护同一个 handle 的 send/recv。
//!
//! # 初始化序列
//!
//! `canInitializeLibrary()` → `canOpenChannel()` → `canSetBusParams()` → `canBusOn()` → send/recv

use crate::{
    CanBitrate, CanBus, CanBusDyn, CanBusFactory, CanConfig, CanFrame, CanId, CanState,
    ClassicFrame, error::CanError,
};
use std::future::Future;
use std::sync::Mutex;
use std::sync::OnceLock;

// ========== 类型定义 ==========

/// Kvaser handle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CanHandle(i32);

/// Kvaser status
type CanStatus = i32;

const CAN_OK: CanStatus = 0;
const CAN_ERR_NOMSG: CanStatus = -2;
const CAN_ERR_TIMEOUT: CanStatus = -7;
const CAN_ERR_NOTINITIALIZED: CanStatus = -8;
const CAN_ERR_INVHANDLE: CanStatus = -10;
const CAN_ERR_BUSOFF: CanStatus = -12;

// Open channel flags
const CAN_OPEN_EXCLUSIVE: i32 = 0x0008;
const CAN_OPEN_ACCEPT_VIRTUAL: i32 = 0x0020;

// Message flags
const CAN_MSG_STD: u32 = 0x0002;
const CAN_MSG_EXT: u32 = 0x0004;

// Predefined bitrates
const CAN_BITRATE_1M: i32 = -1;
const CAN_BITRATE_500K: i32 = -2;
const CAN_BITRATE_250K: i32 = -3;
const CAN_BITRATE_125K: i32 = -4;
const CAN_BITRATE_100K: i32 = -5;
const CAN_BITRATE_50K: i32 = -7;

// Kvaser 状态标志 (canReadStatus 返回值)
const CANSTAT_ERROR_PASSIVE: u32 = 0x00000020;
const CANSTAT_BUS_OFF: u32 = 0x00000040;
const CANSTAT_ERROR_WARNING: u32 = 0x00000010;
const _CANSTAT_ERROR_ACTIVE: u32 = 0x00000008;
const _CANSTAT_TX_PENDING: u32 = 0x00000001;
const _CANSTAT_RX_PENDING: u32 = 0x00000002;

// Kvaser canRead 函数指针
#[allow(dead_code)]
struct KvaserFunctions {
    _lib: libloading::Library,
    fn_initialize_library: usize,
    fn_get_number_of_channels: usize,
    fn_get_channel_data: usize,
    fn_open_channel: usize,
    fn_close: usize,
    fn_bus_on: usize,
    fn_bus_off: usize,
    fn_set_bus_params: usize,
    fn_get_bus_params: usize,
    fn_write: usize,
    fn_write_wait: usize,
    fn_read: usize,
    fn_read_wait: usize,
    fn_read_status: usize,
    fn_get_error_text: usize,
}

// SAFETY: Kvaser CANlib 是按 handle 线程安全的
unsafe impl Send for KvaserFunctions {}
unsafe impl Sync for KvaserFunctions {}

impl KvaserFunctions {
    unsafe fn initialize_library(&self) {
        let func: unsafe extern "C" fn() =
            unsafe { std::mem::transmute(self.fn_initialize_library) };
        unsafe { func() };
    }

    unsafe fn open_channel(&self, channel: i32, flags: i32) -> CanHandle {
        let func: unsafe extern "C" fn(i32, i32) -> i32 =
            unsafe { std::mem::transmute(self.fn_open_channel) };
        CanHandle(unsafe { func(channel, flags) })
    }

    unsafe fn close(&self, handle: CanHandle) -> CanStatus {
        let func: unsafe extern "C" fn(i32) -> i32 = unsafe { std::mem::transmute(self.fn_close) };
        unsafe { func(handle.0) }
    }

    unsafe fn bus_on(&self, handle: CanHandle) -> CanStatus {
        let func: unsafe extern "C" fn(i32) -> i32 = unsafe { std::mem::transmute(self.fn_bus_on) };
        unsafe { func(handle.0) }
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn set_bus_params(
        &self,
        handle: CanHandle,
        freq: i32,
        tseg1: u32,
        tseg2: u32,
        sjw: u32,
        no_samp: u32,
    ) -> CanStatus {
        let func: unsafe extern "C" fn(i32, i32, u32, u32, u32, u32, u32) -> i32 =
            unsafe { std::mem::transmute(self.fn_set_bus_params) };
        unsafe { func(handle.0, freq, tseg1, tseg2, sjw, no_samp, 0) }
    }

    unsafe fn write(
        &self,
        handle: CanHandle,
        id: i32,
        msg: *const u8,
        dlc: u32,
        flags: u32,
    ) -> CanStatus {
        let func: unsafe extern "C" fn(i32, i32, *const u8, u32, u32) -> i32 =
            unsafe { std::mem::transmute(self.fn_write) };
        unsafe { func(handle.0, id, msg, dlc, flags) }
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn read_wait(
        &self,
        handle: CanHandle,
        id: *mut i32,
        msg: *mut u8,
        dlc: *mut u32,
        flags: *mut u32,
        time: *mut u32,
        timeout: u32,
    ) -> CanStatus {
        let func: unsafe extern "C" fn(
            i32,
            *mut i32,
            *mut u8,
            *mut u32,
            *mut u32,
            *mut u32,
            u32,
        ) -> i32 = unsafe { std::mem::transmute(self.fn_read_wait) };
        unsafe { func(handle.0, id, msg, dlc, flags, time, timeout) }
    }

    unsafe fn read_status(&self, handle: CanHandle, flags: *mut u32) -> CanStatus {
        let func: unsafe extern "C" fn(i32, *mut u32) -> i32 =
            unsafe { std::mem::transmute(self.fn_read_status) };
        unsafe { func(handle.0, flags) }
    }
}

// ========== 动态库加载 ==========

static KVASER_FUNCS: OnceLock<Result<KvaserFunctions, String>> = OnceLock::new();

#[cfg(target_os = "windows")]
const KVASER_LIB_NAME: &str = "canlib32.dll";

#[cfg(target_os = "linux")]
const KVASER_LIB_NAME: &str = "libcanlib.so";

#[cfg(target_os = "macos")]
const KVASER_LIB_NAME: &str = "libcanlib.dylib";

unsafe fn load_kvaser_functions() -> Result<KvaserFunctions, String> {
    let lib = unsafe { libloading::Library::new(KVASER_LIB_NAME) }
        .map_err(|e| format!("Failed to load {}: {}", KVASER_LIB_NAME, e))?;

    let fn_initialize_library = unsafe { load_sym(&lib, b"canInitializeLibrary")? };
    let fn_get_number_of_channels = unsafe { load_sym(&lib, b"canGetNumberOfChannels")? };
    let fn_get_channel_data = unsafe { load_sym(&lib, b"canGetChannelData")? };
    let fn_open_channel = unsafe { load_sym(&lib, b"canOpenChannel")? };
    let fn_close = unsafe { load_sym(&lib, b"canClose")? };
    let fn_bus_on = unsafe { load_sym(&lib, b"canBusOn")? };
    let fn_bus_off = unsafe { load_sym(&lib, b"canBusOff")? };
    let fn_set_bus_params = unsafe { load_sym(&lib, b"canSetBusParams")? };
    let fn_get_bus_params = unsafe { load_sym(&lib, b"canGetBusParams")? };
    let fn_write = unsafe { load_sym(&lib, b"canWrite")? };
    let fn_write_wait = unsafe { load_sym(&lib, b"canWriteWait")? };
    let fn_read = unsafe { load_sym(&lib, b"canRead")? };
    let fn_read_wait = unsafe { load_sym(&lib, b"canReadWait")? };
    let fn_read_status = unsafe { load_sym(&lib, b"canReadStatus")? };
    let fn_get_error_text = unsafe { load_sym(&lib, b"canGetErrorText")? };

    Ok(KvaserFunctions {
        _lib: lib,
        fn_initialize_library,
        fn_get_number_of_channels,
        fn_get_channel_data,
        fn_open_channel,
        fn_close,
        fn_bus_on,
        fn_bus_off,
        fn_set_bus_params,
        fn_get_bus_params,
        fn_write,
        fn_write_wait,
        fn_read,
        fn_read_wait,
        fn_read_status,
        fn_get_error_text,
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

fn get_kvaser_funcs() -> Result<&'static KvaserFunctions, CanError> {
    let result = KVASER_FUNCS.get_or_init(|| unsafe { load_kvaser_functions() });

    result
        .as_ref()
        .map_err(|e| CanError::Io(format!("Kvaser CANlib not available: {}", e)))
}

// ========== 错误处理 ==========

fn kvaser_status_to_error(status: CanStatus) -> CanError {
    match status {
        CAN_OK => CanError::Io("Kvaser operation succeeded but returned error".to_string()),
        CAN_ERR_NOMSG => CanError::Timeout,
        CAN_ERR_TIMEOUT => CanError::Timeout,
        CAN_ERR_NOTINITIALIZED => CanError::NotConnected,
        CAN_ERR_INVHANDLE => CanError::Io("Invalid Kvaser handle".to_string()),
        CAN_ERR_BUSOFF => CanError::BusOff,
        _ => CanError::BusError(format!("Kvaser error code: {}", status)),
    }
}

// ========== 波特率映射 ==========

fn bitrate_to_kvaser(bitrate: &CanBitrate) -> i32 {
    match bitrate.nominal {
        1_000_000 => CAN_BITRATE_1M,
        500_000 => CAN_BITRATE_500K,
        250_000 => CAN_BITRATE_250K,
        125_000 => CAN_BITRATE_125K,
        100_000 => CAN_BITRATE_100K,
        50_000 => CAN_BITRATE_50K,
        _ => CAN_BITRATE_500K, // 默认 500 kbps
    }
}

// ========== 库初始化 ==========

static KVASER_INITIALIZED: OnceLock<bool> = OnceLock::new();

fn ensure_initialized() -> Result<(), CanError> {
    KVASER_INITIALIZED.get_or_init(|| {
        if let Ok(funcs) = get_kvaser_funcs() {
            unsafe { funcs.initialize_library() };
            true
        } else {
            false
        }
    });

    if *KVASER_INITIALIZED.get().unwrap_or(&false) {
        Ok(())
    } else {
        Err(CanError::Io(
            "Kvaser CANlib initialization failed".to_string(),
        ))
    }
}

// ========== CanBus 实现 ==========

/// Kvaser CAN 总线实例
pub struct KvaserBus {
    handle: CanHandle,
    _mutex: Mutex<()>, // 保护同一个 handle 的并发调用
}

// SAFETY: Kvaser CANlib 是按 handle 线程安全的，我们使用 Mutex 保护
unsafe impl Send for KvaserBus {}
unsafe impl Sync for KvaserBus {}

impl KvaserBus {
    /// 打开 Kvaser CAN 通道
    pub fn open(channel: &str, config: &CanConfig) -> Result<Self, CanError> {
        ensure_initialized()?;
        let funcs = get_kvaser_funcs()?;

        // 解析通道号
        let channel_idx: i32 = channel
            .parse()
            .map_err(|_| CanError::InvalidConfig(format!("Invalid Kvaser channel: {}", channel)))?;

        // 打开通道（独占模式 + 接受虚拟通道用于测试）
        let flags = CAN_OPEN_EXCLUSIVE | CAN_OPEN_ACCEPT_VIRTUAL;
        let handle = unsafe { funcs.open_channel(channel_idx, flags) };

        if handle.0 < 0 {
            return Err(kvaser_status_to_error(handle.0));
        }

        // 设置波特率
        let bitrate = bitrate_to_kvaser(&config.bitrate);
        let status = unsafe { funcs.set_bus_params(handle, bitrate, 0, 0, 0, 1) };

        if status != CAN_OK {
            unsafe { funcs.close(handle) };
            return Err(kvaser_status_to_error(status));
        }

        // 上线
        let status = unsafe { funcs.bus_on(handle) };

        if status != CAN_OK {
            unsafe { funcs.close(handle) };
            return Err(kvaser_status_to_error(status));
        }

        Ok(Self {
            handle,
            _mutex: Mutex::new(()),
        })
    }
}

impl Drop for KvaserBus {
    fn drop(&mut self) {
        if let Ok(funcs) = get_kvaser_funcs() {
            unsafe {
                funcs.close(self.handle);
            }
        }
    }
}

impl CanBus for KvaserBus {
    fn send(&self, frame: &CanFrame) -> Result<(), CanError> {
        let funcs = get_kvaser_funcs()?;
        let _lock = self._mutex.lock().unwrap();

        let classic = match frame {
            CanFrame::Classic(f) => f,
            CanFrame::Fd(_) => {
                return Err(CanError::Unsupported(
                    "CAN FD not supported yet".to_string(),
                ));
            }
        };

        let (id, flags) = match classic.id {
            CanId::Standard(id) => (id as i32, CAN_MSG_STD),
            CanId::Extended(id) => (id as i32, CAN_MSG_EXT),
        };

        let mut data = [0u8; 8];
        let len = classic.len.min(8) as usize;
        data[..len].copy_from_slice(&classic.data[..len]);

        let status =
            unsafe { funcs.write(self.handle, id, data.as_ptr(), classic.len as u32, flags) };

        if status != CAN_OK {
            return Err(kvaser_status_to_error(status));
        }

        Ok(())
    }

    fn recv(&self) -> impl Future<Output = Result<CanFrame, CanError>> + Send {
        let handle = self.handle;

        async move {
            

            tokio::task::spawn_blocking(move || {
                let funcs = get_kvaser_funcs()?;

                let mut id: i32 = 0;
                let mut data = [0u8; 8];
                let mut dlc: u32 = 0;
                let mut flags: u32 = 0;
                let mut time: u32 = 0;

                // 使用 canReadWait 实现阻塞接收，超时 100ms
                let status = unsafe {
                    funcs.read_wait(
                        handle,
                        &mut id,
                        data.as_mut_ptr(),
                        &mut dlc,
                        &mut flags,
                        &mut time,
                        100,
                    )
                };

                if status != CAN_OK {
                    if status == CAN_ERR_NOMSG || status == CAN_ERR_TIMEOUT {
                        return Err(CanError::Timeout);
                    }
                    return Err(kvaser_status_to_error(status));
                }

                let can_id = if flags & CAN_MSG_EXT != 0 {
                    CanId::Extended((id as u32) & 0x1FFFFFFF)
                } else {
                    CanId::Standard((id as u32 & 0x7FF) as u16)
                };

                let mut frame_data = [0u8; 8];
                let len = dlc.min(8) as usize;
                frame_data[..len].copy_from_slice(&data[..len]);

                Ok(CanFrame::Classic(ClassicFrame {
                    id: can_id,
                    data: frame_data,
                    len: dlc as u8,
                    timestamp_us: Some(time as u64 * 1000), // 转换为微秒
                }))
            })
            .await
            .map_err(|e| CanError::Io(format!("Task join error: {}", e)))?
        }
    }

    fn state(&self) -> CanState {
        let funcs = match get_kvaser_funcs() {
            Ok(f) => f,
            Err(_) => return CanState::NotConnected,
        };

        let mut flags: u32 = 0;
        let status = unsafe { funcs.read_status(self.handle, &mut flags) };

        if status != CAN_OK {
            return CanState::NotConnected;
        }

        // 根据状态标志判断总线状态
        if flags & CANSTAT_BUS_OFF != 0 {
            CanState::BusOff
        } else if flags & CANSTAT_ERROR_PASSIVE != 0 {
            CanState::ErrorPassive
        } else if flags & CANSTAT_ERROR_WARNING != 0 {
            CanState::Warning
        } else {
            CanState::Active
        }
    }

    fn set_bitrate(&self, bitrate: CanBitrate) -> Result<(), CanError> {
        // Kvaser 需要下线、重新设置参数、再上线
        let _ = bitrate;
        Err(CanError::Unsupported(
            "Kvaser bitrate change requires channel reinitialization".to_string(),
        ))
    }
}

// ========== CanBusFactory 实现 ==========

/// Kvaser CAN 工厂
pub struct KvaserFactory;

impl CanBusFactory for KvaserFactory {
    fn open(&self, channel: &str, config: &CanConfig) -> Result<Box<dyn CanBusDyn>, CanError> {
        let bus = KvaserBus::open(channel, config)?;
        Ok(Box::new(bus))
    }

    fn name(&self) -> &str {
        "Kvaser"
    }

    fn available_channels(&self) -> Vec<String> {
        let mut channels = Vec::new();

        if ensure_initialized().is_err() {
            return channels;
        }

        if let Ok(funcs) = get_kvaser_funcs() {
            // Kvaser 支持最多 64 个通道
            // 使用 accept_virtual 标志来包含虚拟通道（用于测试）
            let max_channels = 64;

            for i in 0..max_channels {
                // 尝试打开通道（独占模式 + 接受虚拟通道）
                let handle =
                    unsafe { funcs.open_channel(i, CAN_OPEN_EXCLUSIVE | CAN_OPEN_ACCEPT_VIRTUAL) };

                if handle.0 >= 0 {
                    // 获取通道信息
                    // canGetChannelData 可以获取更多详细信息，这里简化处理
                    channels.push(format!("can{}", i));
                    unsafe { funcs.close(handle) };
                }
            }
        }

        channels
    }
}
