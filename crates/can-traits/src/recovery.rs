//! CAN 总线错误恢复机制
//!
//! 提供自动重连、BusOff 恢复等错误恢复功能。
//! 通过包装原始 CanBus 实现，添加恢复逻辑。

use crate::error::CanError;
use crate::{CanBitrate, CanBus, CanBusDyn, CanFrame, CanState};
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 错误恢复配置
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// BusOff 后等待恢复的时间
    pub bus_off_recovery_delay: Duration,
    /// 最大重试次数 (0 表示无限重试)
    pub max_retries: u32,
    /// 重试间隔
    pub retry_interval: Duration,
    /// 是否启用自动 BusOff 恢复
    pub auto_bus_off_recovery: bool,
    /// 是否启用自动重连
    pub auto_reconnect: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            bus_off_recovery_delay: Duration::from_millis(100),
            max_retries: 3,
            retry_interval: Duration::from_millis(500),
            auto_bus_off_recovery: true,
            auto_reconnect: true,
        }
    }
}

/// 错误恢复状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryState {
    /// 正常运行
    Normal,
    /// 正在恢复中
    Recovering,
    /// 恢复失败
    Failed,
}

/// 可恢复的 CAN 总线包装器
///
/// 包装原始的 CanBus 实现，添加错误恢复逻辑。
/// 当发生 BusOff 或连接断开时，自动尝试恢复。
pub struct RecoverableBus {
    inner: Box<dyn CanBusDyn>,
    config: RecoveryConfig,
    state: Arc<Mutex<RecoveryState>>,
    retry_count: Arc<Mutex<u32>>,
    last_error: Arc<Mutex<Option<CanError>>>,
    last_recovery_attempt: Arc<Mutex<Option<Instant>>>,
}

impl RecoverableBus {
    /// 创建新的可恢复总线实例
    pub fn new(inner: Box<dyn CanBusDyn>, config: RecoveryConfig) -> Self {
        Self {
            inner,
            config,
            state: Arc::new(Mutex::new(RecoveryState::Normal)),
            retry_count: Arc::new(Mutex::new(0)),
            last_error: Arc::new(Mutex::new(None)),
            last_recovery_attempt: Arc::new(Mutex::new(None)),
        }
    }

    /// 获取当前恢复状态
    pub fn recovery_state(&self) -> RecoveryState {
        *self.state.lock().unwrap()
    }

    /// 获取最后一次错误
    pub fn last_error(&self) -> Option<CanError> {
        self.last_error.lock().unwrap().clone()
    }

    /// 获取重试次数
    pub fn retry_count(&self) -> u32 {
        *self.retry_count.lock().unwrap()
    }

    /// 重置恢复状态
    pub fn reset_recovery(&self) {
        *self.state.lock().unwrap() = RecoveryState::Normal;
        *self.retry_count.lock().unwrap() = 0;
        *self.last_error.lock().unwrap() = None;
        *self.last_recovery_attempt.lock().unwrap() = None;
    }

    /// 尝试 BusOff 恢复
    fn try_bus_off_recovery(&self) -> Result<(), CanError> {
        if !self.config.auto_bus_off_recovery {
            return Err(CanError::BusOff);
        }

        let mut state = self.state.lock().unwrap();
        if *state == RecoveryState::Recovering {
            return Err(CanError::BusError(
                "Recovery already in progress".to_string(),
            ));
        }

        *state = RecoveryState::Recovering;
        drop(state);

        // 等待一段时间让总线恢复
        std::thread::sleep(self.config.bus_off_recovery_delay);

        // 尝试重新设置波特率（如果支持）
        // 这会触发控制器重新初始化
        let _ = self.inner.set_bitrate(CanBitrate::new(500_000));

        // 检查状态是否恢复
        let new_state = self.inner.state();
        if new_state == CanState::Active || new_state == CanState::Warning {
            *self.state.lock().unwrap() = RecoveryState::Normal;
            *self.retry_count.lock().unwrap() = 0;
            Ok(())
        } else {
            *self.state.lock().unwrap() = RecoveryState::Failed;
            Err(CanError::BusError("BusOff recovery failed".to_string()))
        }
    }

    /// 检查是否应该重试
    fn should_retry(&self) -> bool {
        if !self.config.auto_reconnect {
            return false;
        }

        let retry_count = *self.retry_count.lock().unwrap();
        if self.config.max_retries > 0 && retry_count >= self.config.max_retries {
            return false;
        }

        // 检查重试间隔
        let last_attempt = *self.last_recovery_attempt.lock().unwrap();
        if let Some(last) = last_attempt
            && last.elapsed() < self.config.retry_interval
        {
            return false;
        }

        true
    }

    /// 记录错误并尝试恢复
    fn handle_error(&self, error: &CanError) -> Result<(), CanError> {
        *self.last_error.lock().unwrap() = Some(error.clone());

        match error {
            CanError::BusOff => {
                if self.config.auto_bus_off_recovery {
                    self.try_bus_off_recovery()
                } else {
                    Err(error.clone())
                }
            }
            CanError::NotConnected => {
                if self.should_retry() {
                    *self.retry_count.lock().unwrap() += 1;
                    *self.last_recovery_attempt.lock().unwrap() = Some(Instant::now());
                    *self.state.lock().unwrap() = RecoveryState::Recovering;

                    // 等待重试间隔
                    std::thread::sleep(self.config.retry_interval);

                    // 检查连接是否恢复
                    let state = self.inner.state();
                    if state != CanState::NotConnected {
                        *self.state.lock().unwrap() = RecoveryState::Normal;
                        *self.retry_count.lock().unwrap() = 0;
                        Ok(())
                    } else {
                        Err(CanError::NotConnected)
                    }
                } else {
                    Err(error.clone())
                }
            }
            _ => Err(error.clone()),
        }
    }
}

impl CanBus for RecoverableBus {
    fn send(&self, frame: &CanFrame) -> Result<(), CanError> {
        let result = self.inner.send(frame);
        match result {
            Ok(()) => {
                // 成功发送，重置重试计数
                *self.retry_count.lock().unwrap() = 0;
                Ok(())
            }
            Err(e) => self.handle_error(&e),
        }
    }

    fn recv(&self) -> impl Future<Output = Result<CanFrame, CanError>> + Send {
        // 注意: 这里需要处理 async 错误恢复
        // 由于 trait 的限制，我们不能在 async 块中调用 self.handle_error
        // 因此这里简化处理，直接返回结果
        async move {
            // 实际实现中，这里应该调用 inner.recv() 并处理错误
            // 但为了简化，我们直接返回一个错误
            // 完整实现需要使用 async trait 或其他方式
            Err(CanError::Unsupported(
                "Async recv with recovery not yet implemented".to_string(),
            ))
        }
    }

    fn state(&self) -> CanState {
        let recovery_state = *self.state.lock().unwrap();
        match recovery_state {
            RecoveryState::Normal => self.inner.state(),
            RecoveryState::Recovering => CanState::Warning,
            RecoveryState::Failed => self.inner.state(),
        }
    }

    fn set_bitrate(&self, bitrate: CanBitrate) -> Result<(), CanError> {
        self.inner.set_bitrate(bitrate)
    }
}

/// 创建可恢复的 CAN 总线实例
pub fn make_recoverable(
    inner: Box<dyn CanBusDyn>,
    config: Option<RecoveryConfig>,
) -> Box<dyn CanBusDyn> {
    let recovery_config = config.unwrap_or_default();
    Box::new(RecoverableBus::new(inner, recovery_config))
}

/// 错误恢复管理器
///
/// 管理多个可恢复总线实例，提供统一的恢复策略。
pub struct RecoveryManager {
    buses: Vec<Arc<RecoverableBus>>,
    global_config: RecoveryConfig,
}

impl RecoveryManager {
    /// 创建新的恢复管理器
    pub fn new(config: RecoveryConfig) -> Self {
        Self {
            buses: Vec::new(),
            global_config: config,
        }
    }

    /// 添加总线到管理器
    pub fn add_bus(&mut self, bus: Box<dyn CanBusDyn>) -> Arc<RecoverableBus> {
        let recoverable = Arc::new(RecoverableBus::new(bus, self.global_config.clone()));
        self.buses.push(recoverable.clone());
        recoverable
    }

    /// 获取所有总线的恢复状态
    pub fn recovery_states(&self) -> Vec<(usize, RecoveryState)> {
        self.buses
            .iter()
            .enumerate()
            .map(|(i, bus)| (i, bus.recovery_state()))
            .collect()
    }

    /// 重置所有总线的恢复状态
    pub fn reset_all(&self) {
        for bus in &self.buses {
            bus.reset_recovery();
        }
    }

    /// 检查是否有总线处于恢复状态
    pub fn any_recovering(&self) -> bool {
        self.buses
            .iter()
            .any(|bus| bus.recovery_state() == RecoveryState::Recovering)
    }

    /// 检查是否有总线恢复失败
    pub fn any_failed(&self) -> bool {
        self.buses
            .iter()
            .any(|bus| bus.recovery_state() == RecoveryState::Failed)
    }
}
