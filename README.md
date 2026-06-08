# OpenCAN

**CAN / CANopen 调试工具** — 跨平台桌面 GUI + 独立协议栈 crate

[![Rust](https://img.shields.io/badge/Rust-2024-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](#license)
[![CI](https://github.com/your-org/opencan/actions/workflows/ci.yml/badge.svg)](https://github.com/your-org/opencan/actions)

---

## ✨ 功能特性

### 🔌 硬件支持
- **SocketCAN** — Linux 原生 CAN 接口
- **ZLG 致远电子** — USBCAN/PCAN 系列
- **Kvaser** — Leaf/USBcan 系列
- **PCAN** — Peak CAN 系列
- **CAN FD** — 64 字节数据帧支持（开发中）

### 📡 CANopen 协议栈
- **DS301** — 标准通信协议（SDO/PDO/NMT/Heartbeat/EMCY/SYNC）
- **DS402** — 运动控制配置文件（CSP/CSV/CST/PP/PV/PT/Homing）
- **EDS 解析** — 电子数据表文件导入导出
- **LSS** — 层设置服务（开发中）

### 🖥️ 桌面 GUI
- **实时监控** — CAN 帧收发 + 过滤 + 搜索
- **SDO 探索器** — 对象字典浏览 + 读写操作
- **DS402 控制** — 状态机可视化 + 运动控制面板
- **网络拓扑** — SVG 节点图 + NMT 命令
- **协议分析** — 高层协议解码 + 错误诊断
- **数据录制** — CAN 帧录制回放 + CSV/ASC 导出
- **快捷键** — 全局键盘快捷键支持

---

## 🚀 快速开始

### 环境要求

- **Rust** 1.75+ (edition 2024)
- **Node.js** 18+ / Bun 1.0+
- **Tauri CLI** 2.x

### 安装

```bash
# 克隆仓库
git clone https://github.com/your-org/opencan.git
cd opencan

# 安装前端依赖
cd frontend && npm install && cd ..

# 安装 Tauri CLI
cargo install tauri-cli
```

### 开发模式

```bash
# 启动开发服务器（前端热重载 + Rust）
just tauri-dev

# 或使用 cargo
cd opencan-gui/src-tauri && cargo tauri dev
```

### 构建

```bash
# 生产构建
just tauri-build

# Debug 构建
just tauri-build-debug
```

---

## 📦 项目结构

```
OpenCAN/
├── crates/
│   ├── can-traits/          # CAN 硬件抽象层
│   ├── canopen-core/        # DS301 标准协议实现
│   ├── canopen-master/      # 主站增强功能
│   └── canopen-ds402/       # DS402 运动控制配置文件
├── opencan-gui/
│   └── src-tauri/           # Tauri 后端
├── frontend/                # React + TypeScript 前端
│   ├── src/
│   │   ├── pages/           # 页面组件
│   │   ├── components/      # 通用组件
│   │   ├── hooks/           # 自定义 Hooks
│   │   ├── lib/             # 工具库
│   │   └── types/           # TypeScript 类型
│   └── ...
└── docs/                    # 文档
```

---

## 🧪 测试

```bash
# Rust 测试
cargo test --workspace

# 前端测试
cd frontend && npm test

# Clippy 检查
cargo clippy --workspace --all-features -- -D warnings

# 格式检查
cargo fmt --check
```

---

## 📚 文档

- [用户手册](docs/user-guide.md) — 安装指南 + 使用教程
- [开发者指南](docs/developer-guide.md) — 架构说明 + 贡献指南
- [API 文档](https://docs.rs/opencan) — Rust crate API
- [变更日志](CHANGELOG.md) — 版本历史

---

## 🛠️ 技术栈

### 后端
- **Rust** 2024 Edition
- **Tauri 2** — 桌面应用框架
- **Tokio 1** — 异步运行时
- **SocketCAN 3.5** — Linux CAN 接口

### 前端
- **React 18** + **TypeScript 5**
- **Vite 6** — 构建工具
- **Tailwind CSS 3** — 样式框架
- **Zustand 5** — 状态管理
- **React Query 5** — 数据获取
- **Lucide React** — 图标库

---

## 🤝 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

### 开发流程

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'feat: add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

### 代码规范

- Rust: 遵循 `rustfmt` 格式 + `clippy` 检查
- TypeScript: 遵循 ESLint + Prettier 规范
- 提交信息: 使用 [Conventional Commits](https://www.conventionalcommits.org/)

---

## 📄 许可证

本项目采用以下任一许可证：

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

---

## 🙏 致谢

感谢以下开源项目：

- [Tauri](https://tauri.app/) — 跨平台桌面应用框架
- [SocketCAN](https://github.com/socketcan-rs/socketcan-rs) — Rust SocketCAN 绑定
- [Zustand](https://github.com/pmndrs/zustand) — React 状态管理
- [Lucide](https://lucide.dev/) — 图标库

---

<div align="center">
  <sub>Built with ❤️ by the OpenCAN Team</sub>
</div>
