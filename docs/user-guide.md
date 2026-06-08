# OpenCAN 用户手册

本手册将帮助你了解如何使用 OpenCAN 进行 CAN / CANopen 调试。

---

## 📋 目录

- [快速开始](#快速开始)
- [连接设备](#连接设备)
- [CAN 帧监控](#can-帧监控)
- [SDO 操作](#sdo-操作)
- [DS402 运动控制](#ds402-运动控制)
- [网络管理](#网络管理)
- [数据录制与导出](#数据录制与导出)
- [键盘快捷键](#键盘快捷键)
- [常见问题](#常见问题)

---

## 🚀 快速开始

### 1. 启动应用

```bash
# 开发模式
just tauri-dev

# 或
cd opencan-gui/src-tauri && cargo tauri dev
```

### 2. 连接 CAN 设备

1. 点击顶部工具栏的 **连接** 按钮
2. 选择硬件后端（SocketCAN / ZLG / Kvaser / PCAN）
3. 选择设备和波特率
4. 点击 **连接**

### 3. 开始调试

连接成功后，你可以：
- 查看实时 CAN 帧流
- 发送 CAN 帧
- 执行 SDO 读写操作
- 控制 DS402 设备

---

## 🔌 连接设备

### 支持的硬件

| 硬件 | 后端 | 平台 | 说明 |
|------|------|------|------|
| SocketCAN | socketcan | Linux | Linux 原生 CAN 接口 |
| ZLG USBCAN | zlg | Windows/Linux | 致远电子 USBCAN 系列 |
| Kvaser Leaf | kvaser | Windows/Linux | Kvaser CAN 适配器 |
| PCAN-USB | pcan | Windows/Linux | Peak CAN 适配器 |

### 连接参数

- **设备** — 选择具体的 CAN 接口
- **波特率** — CAN 总线波特率（125K/250K/500K/1M）
- **FD 波特率** — CAN FD 数据阶段波特率（2M/5M/8M）

### 连接状态

- 🟢 **已连接** — 正常通信
- 🔴 **断开** — 连接已断开
- 🟡 **连接中** — 正在建立连接

---

## 📡 CAN 帧监控

### 实时帧流

左侧 **Frame Monitor** 面板显示实时 CAN 帧：

```
时间戳     方向  ID      DLC  数据
12:34:56  TX    0x123   8    01 02 03 04 05 06 07 08
12:34:57  RX    0x456   4    AB CD EF 00
```

### 帧过滤

使用过滤器面板筛选帧：

- **ID 范围** — 如 `0x100-0x200`
- **方向** — TX / RX
- **帧类型** — 标准帧 / 扩展帧 / FD 帧
- **关键字** — 搜索帧数据

### 预设过滤器

保存常用过滤器配置：

1. 设置过滤条件
2. 点击 **保存预设**
3. 输入预设名称
4. 下次直接从下拉菜单选择

### 发送帧

1. 点击 **发送** 标签
2. 输入帧 ID（如 `0x123`）
3. 输入数据（如 `01 02 03 04`）
4. 选择帧类型（标准/扩展/FD）
5. 点击 **发送**

---

## 🔄 SDO 操作

### SDO Explorer

**CANOpen > SDO** 标签页提供交互式 SDO 探索器：

#### 浏览对象字典

- 左侧面板显示 OD 树形视图
- 按 Index 范围分组：
  - **1000-1FFF** — 通信参数
  - **2000-5FFF** — 厂商特定
  - **6000-9FFF** — 设备配置文件（DS402）
  - **C000-FFFF** — 配置文件特定

#### 读取条目

1. 展开 Index 节点
2. 点击 SubIndex 条目
3. 右侧面板显示详细信息
4. 点击 **Read** 按钮
5. 显示 Hex/Bin/Dec 格式的值

#### 写入条目

1. 选择可写的条目（Access: RW）
2. 在 **Write** 输入框输入十六进制值
3. 点击 **Write** 按钮
4. 确认写入成功

#### SDO 历史

- 底部面板显示 SDO 操作历史
- 支持回放操作
- 支持导出 CSV

---

## 🎮 DS402 运动控制

### DS402 Control 页面

**CANOpen > DS402** 标签页提供完整的 DS402 控制界面：

#### 状态机流程图

SVG 可视化 CiA 402 状态机：

- **8 个状态节点** — Not Ready → Switch On Disabled → Ready → Switched On → Operation Enabled → Quick Stop → Fault Reaction → Fault
- **转换边** — 显示 ControlWord 值（0x6040）
- **当前状态** — 高亮显示 + 脉冲动画
- **点击转换** — 直接发送 SDO 命令

#### 控制字按钮

快速操作按钮：

| 按钮 | ControlWord | 说明 |
|------|-------------|------|
| Shutdown | 0x0006 | 准备启动 |
| Switch On | 0x0007 | 开启设备 |
| Enable Operation | 0x000F | 使能操作 |
| Disable Operation | 0x0007 | 禁用操作 |
| Quick Stop | 0x0002 | 快速停止 |
| Fault Reset | 0x0080 | 故障复位 |

#### StatusWord 解析

实时解析 StatusWord（0x6041）的 14 个位：

- Bit 0: Ready to Switch On
- Bit 1: Switched On
- Bit 2: Operation Enabled
- Bit 3: Fault
- Bit 4: Voltage Enabled
- Bit 5: Quick Stop Active
- ...

#### 运动模式

支持的操作模式：

| 模式 | 缩写 | 说明 |
|------|------|------|
| Profile Position | PP | 位置轮廓 |
| Profile Velocity | PV | 速度轮廓 |
| Profile Torque | PT | 力矩轮廓 |
| Homing | HM | 回零 |
| Cyclic Sync Position | CSP | 同步位置 |
| Cyclic Sync Velocity | CSV | 同步速度 |
| Cyclic Sync Torque | CST | 同步力矩 |

#### 目标控制

1. 选择运动模式
2. 设置目标值（位置/速度/力矩）
3. 点击 **Set Target** 发送 SDO
4. 查看实际值反馈

---

## 🌐 网络管理

### Network Topology

**CANOpen > Network** 标签页显示网络拓扑：

#### 节点可视化

- **Master 节点** — 中心位置
- **Slave 节点** — 半圆排列
- **颜色编码**：
  - 🟢 绿色 — Operational
  - 🟡 黄色 — PreOperational
  - 🔴 红色 — Stopped
  - ⚫ 灰色 — Offline

#### 节点操作

1. **点击节点** — 选中并显示详细信息
2. **拖拽节点** — 重新排列布局
3. **NMT 命令** — 对选中节点发送：
   - Start — 启动节点
   - Stop — 停止节点
   - Pre-Op — 进入预操作状态
   - Reset — 重置节点

#### 网络扫描

点击 **Scan** 按钮自动发现网络上的节点。

---

## 🎙️ 数据录制与导出

### 录制 CAN 帧

1. 点击工具栏的 **录制** 按钮（或按 `Space`）
2. 开始录制 CAN 帧
3. 再次点击停止录制

### 导出数据

支持两种格式：

#### CSV 格式

```csv
timestamp, direction, id, dlc, data, frame_type
1234567890, RX, 0x123, 8, 01 02 03 04 05 06 07 08, Standard
```

#### ASC 格式（Vector）

```
  1.234567 1  123             Rx   d 8 01 02 03 04 05 06 07 08
```

### 导出操作

1. 点击 **导出** 按钮
2. 选择格式（CSV / ASC）
3. 选择保存位置
4. 文件自动下载

---

## ⌨️ 键盘快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+K` | 命令面板 |
| `Ctrl+L` | 切换侧边栏 |
| `Space` | 开始/停止录制 |
| `Ctrl+1` | 切换到 CAN 帧监控 |
| `Ctrl+2` | 切换到 SDO 探索器 |
| `Ctrl+3` | 切换到 DS402 控制 |
| `Ctrl+4` | 切换到网络拓扑 |
| `Escape` | 关闭弹窗/取消操作 |

---

## ❓ 常见问题

### Q: 无法连接到 CAN 设备

**A:** 检查以下几点：
1. 设备是否正确连接
2. 驱动是否已安装
3. 波特率是否匹配
4. 设备是否被其他程序占用

### Q: SocketCAN 连接失败

**A:** Linux 下需要创建虚拟 CAN 接口：
```bash
sudo modprobe vcan
sudo ip link add dev vcan0 type vcan
sudo ip link set up vcan0
```

### Q: SDO 超时

**A:** 可能的原因：
1. 目标节点未响应
2. Index/SubIndex 不存在
3. 波特率不匹配
4. 节点处于错误状态

### Q: DS402 状态机卡在某个状态

**A:** 检查 StatusWord 位：
- Bit 3 (Fault) — 是否有故障
- Bit 5 (Quick Stop) — 是否触发了快速停止
- 尝试发送 Fault Reset (0x0080)

### Q: 前端界面无响应

**A:** 尝试：
1. 刷新页面
2. 检查控制台错误
3. 重启应用
4. 清除浏览器缓存

---

## 📞 获取帮助

- 📖 [开发者指南](developer-guide.md)
- 🐛 [Issue Tracker](https://github.com/your-org/opencan/issues)
- 💬 [GitHub Discussions](https://github.com/your-org/opencan/discussions)
- 📧 [邮件列表](mailto:dev@opencan.org)
