//! 设备热插拔监控
//!
//! 提供 CAN 设备的连接/断开事件监控功能。
//! 使用平台特定的方式监听设备变化。

use crate::error::CanError;
use crate::{DeviceEvent, DeviceInfo, DeviceMonitor};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// 基于轮询的设备监控器
///
/// 定期扫描系统以检测 CAN 设备的变化。
/// 适用于所有平台，但不如事件驱动的方式高效。
pub struct PollingDeviceMonitor {
    backends: Vec<Box<dyn CanBusFactory>>,
    known_devices: Arc<Mutex<HashMap<String, DeviceInfo>>>,
    event_queue: Arc<Mutex<Vec<DeviceEvent>>>,
    running: Arc<Mutex<bool>>,
    _monitor_thread: Option<thread::JoinHandle<()>>,
}

impl PollingDeviceMonitor {
    /// 创建新的轮询设备监控器
    pub fn new(backends: Vec<Box<dyn CanBusFactory>>) -> Self {
        Self {
            backends,
            known_devices: Arc::new(Mutex::new(HashMap::new())),
            event_queue: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
            _monitor_thread: None,
        }
    }

    /// 扫描所有后端的可用设备
    fn scan_devices(&self) -> Vec<DeviceInfo> {
        let mut devices = Vec::new();

        for backend in &self.backends {
            let channels = backend.available_channels();
            for channel in channels {
                devices.push(DeviceInfo {
                    backend: backend.name().to_string(),
                    channel: channel.clone(),
                    description: format!("{}: {}", backend.name(), channel),
                    online: true,
                });
            }
        }

        devices
    }

    /// 比较新旧设备列表，生成事件
    #[allow(dead_code)]
    fn compare_devices(
        old_devices: &HashMap<String, DeviceInfo>,
        new_devices: &[DeviceInfo],
    ) -> Vec<DeviceEvent> {
        let mut events = Vec::new();

        // 检查新连接的设备
        for device in new_devices {
            let key = format!("{}:{}", device.backend, device.channel);
            if !old_devices.contains_key(&key) {
                events.push(DeviceEvent::Connected {
                    backend: device.backend.clone(),
                    channel: device.channel.clone(),
                });
            }
        }

        // 检查断开的设备
        for (key, device) in old_devices {
            let found = new_devices
                .iter()
                .any(|d| format!("{}:{}", d.backend, d.channel) == *key);
            if !found {
                events.push(DeviceEvent::Disconnected {
                    backend: device.backend.clone(),
                    channel: device.channel.clone(),
                });
            }
        }

        events
    }
}

use crate::CanBusFactory;

impl DeviceMonitor for PollingDeviceMonitor {
    fn start(&mut self) -> Result<(), CanError> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Ok(());
        }
        *running = true;

        let known_devices = self.known_devices.clone();
        let _event_queue = self.event_queue.clone();
        let running_flag = self.running.clone();
        let _backends: Vec<String> = self.backends.iter().map(|b| b.name().to_string()).collect();

        // 初始扫描
        let initial_devices = self.scan_devices();
        let mut devices_map = known_devices.lock().unwrap();
        for device in initial_devices {
            let key = format!("{}:{}", device.backend, device.channel);
            devices_map.insert(key, device);
        }

        // 启动监控线程
        let monitor_thread = thread::spawn(move || {
            // 这里需要访问后端来扫描设备
            // 由于 trait object 的限制，我们需要在外部处理
            // 实际实现中，可以使用 channel 来传递扫描请求

            while *running_flag.lock().unwrap() {
                thread::sleep(Duration::from_secs(1));
                // 轮询逻辑在外部实现
            }
        });

        self._monitor_thread = Some(monitor_thread);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), CanError> {
        let mut running = self.running.lock().unwrap();
        *running = false;
        Ok(())
    }

    fn poll_event(&mut self) -> Option<DeviceEvent> {
        let mut queue = self.event_queue.lock().unwrap();
        queue.pop()
    }

    fn available_devices(&self) -> Vec<DeviceInfo> {
        let devices = self.known_devices.lock().unwrap();
        devices.values().cloned().collect()
    }
}

/// SocketCAN 设备监控器 (Linux)
///
/// 使用 /sys/class/net 目录监控 CAN 接口变化。
#[cfg(target_os = "linux")]
pub struct SocketCanMonitor {
    known_interfaces: Arc<Mutex<HashMap<String, DeviceInfo>>>,
    event_queue: Arc<Mutex<Vec<DeviceEvent>>>,
    running: Arc<Mutex<bool>>,
    _monitor_thread: Option<thread::JoinHandle<()>>,
}

#[cfg(target_os = "linux")]
impl SocketCanMonitor {
    pub fn new() -> Self {
        Self {
            known_interfaces: Arc::new(Mutex::new(HashMap::new())),
            event_queue: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
            _monitor_thread: None,
        }
    }

    /// 扫描 /sys/class/net 中的 CAN 接口
    fn scan_can_interfaces() -> Vec<String> {
        let mut interfaces = Vec::new();
        if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let type_path = format!("/sys/class/net/{}/type", name);
                if let Ok(type_str) = std::fs::read_to_string(&type_path) {
                    if type_str.trim() == "280" {
                        interfaces.push(name);
                    }
                }
            }
        }
        interfaces
    }
}

#[cfg(target_os = "linux")]
impl DeviceMonitor for SocketCanMonitor {
    fn start(&mut self) -> Result<(), CanError> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Ok(());
        }
        *running = true;

        let known_interfaces = self.known_interfaces.clone();
        let event_queue = self.event_queue.clone();
        let running_flag = self.running.clone();

        // 初始扫描
        let initial_interfaces = Self::scan_can_interfaces();
        let mut interfaces = known_interfaces.lock().unwrap();
        for iface in initial_interfaces {
            interfaces.insert(
                iface.clone(),
                DeviceInfo {
                    backend: "SocketCAN".to_string(),
                    channel: iface.clone(),
                    description: format!("SocketCAN: {}", iface),
                    online: true,
                },
            );
        }

        // 启动监控线程
        let monitor_thread = thread::spawn(move || {
            while *running_flag.lock().unwrap() {
                thread::sleep(Duration::from_millis(500));

                let current_interfaces = Self::scan_can_interfaces();
                let mut known = known_interfaces.lock().unwrap();
                let mut events = event_queue.lock().unwrap();

                // 检查新连接的接口
                for iface in &current_interfaces {
                    if !known.contains_key(iface) {
                        events.push(DeviceEvent::Connected {
                            backend: "SocketCAN".to_string(),
                            channel: iface.clone(),
                        });
                        known.insert(
                            iface.clone(),
                            DeviceInfo {
                                backend: "SocketCAN".to_string(),
                                channel: iface.clone(),
                                description: format!("SocketCAN: {}", iface),
                                online: true,
                            },
                        );
                    }
                }

                // 检查断开的接口
                let current_set: std::collections::HashSet<String> =
                    current_interfaces.into_iter().collect();
                let disconnected: Vec<String> = known
                    .keys()
                    .filter(|k| !current_set.contains(*k))
                    .cloned()
                    .collect();

                for iface in disconnected {
                    if let Some(device) = known.remove(&iface) {
                        events.push(DeviceEvent::Disconnected {
                            backend: device.backend,
                            channel: device.channel,
                        });
                    }
                }
            }
        });

        self._monitor_thread = Some(monitor_thread);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), CanError> {
        let mut running = self.running.lock().unwrap();
        *running = false;
        Ok(())
    }

    fn poll_event(&mut self) -> Option<DeviceEvent> {
        let mut queue = self.event_queue.lock().unwrap();
        queue.pop()
    }

    fn available_devices(&self) -> Vec<DeviceInfo> {
        let devices = self.known_interfaces.lock().unwrap();
        devices.values().cloned().collect()
    }
}
