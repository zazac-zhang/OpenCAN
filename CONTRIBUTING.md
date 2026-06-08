# Contributing to OpenCAN

感谢你对 OpenCAN 项目的关注！本文档将帮助你了解如何参与贡献。

---

## 📋 目录

- [开发环境](#开发环境)
- [代码规范](#代码规范)
- [提交规范](#提交规范)
- [Pull Request 流程](#pull-request-流程)
- [报告问题](#报告问题)
- [功能建议](#功能建议)

---

## 🛠️ 开发环境

### 必需工具

- **Rust** 1.75+ (edition 2024)
- **Node.js** 18+ 或 **Bun** 1.0+
- **Tauri CLI** 2.x
- **Git** 2.30+

### 可选工具

- **just** — 任务运行器
- **cargo-watch** — 文件变更自动编译
- **prettier** — 代码格式化
- **eslint** — TypeScript 检查

### 环境配置

```bash
# 1. Fork 并克隆仓库
git clone https://github.com/YOUR_USERNAME/opencan.git
cd opencan

# 2. 安装 Rust 工具链
rustup default stable
rustup component add clippy rustfmt

# 3. 安装前端依赖
cd frontend && npm install && cd ..

# 4. 安装 Tauri CLI
cargo install tauri-cli

# 5. 安装 just（可选）
cargo install just

# 6. 验证环境
cargo check --workspace
cd frontend && npm run typecheck
```

---

## 📝 代码规范

### Rust

```bash
# 格式化
cargo fmt

# Clippy 检查
cargo clippy --workspace --all-features -- -D warnings

# 测试
cargo test --workspace
```

**规范要点：**
- 使用 `rustfmt` 默认配置
- 所有公开 API 必须有文档注释
- 使用 `thiserror` 定义错误类型
- 使用 `tracing` 进行日志记录
- 避免 `unwrap()`，使用 `?` 操作符
- 优先使用 `impl Trait` 而非 `Box<dyn Trait>`

### TypeScript / React

```bash
cd frontend

# TypeScript 检查
npm run typecheck

# Lint 检查
npm run lint

# 格式化
npm run format

# 测试
npm test
```

**规范要点：**
- 使用 TypeScript 严格模式
- 组件使用函数式组件 + Hooks
- 状态管理使用 Zustand
- 样式使用 Tailwind CSS
- 使用 `cn()` 工具函数合并类名
- 组件 Props 使用 interface 定义

---

## 📦 提交规范

本项目使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范。

### 提交格式

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### 类型说明

| 类型 | 说明 | 示例 |
|------|------|------|
| `feat` | 新功能 | `feat(sdo): add block transfer support` |
| `fix` | Bug 修复 | `fix(frame): correct PDO unpack logic` |
| `docs` | 文档更新 | `docs(readme): add build instructions` |
| `style` | 代码格式 | `style: fix indentation` |
| `refactor` | 重构 | `refactor(core): extract SDO client module` |
| `test` | 测试 | `test(sdo): add block transfer tests` |
| `chore` | 构建/工具 | `chore: update dependencies` |
| `perf` | 性能优化 | `perf(frontend): add code splitting` |

### 范围说明

| 范围 | 说明 |
|------|------|
| `core` | canopen-core crate |
| `master` | canopen-master crate |
| `ds402` | canopen-ds402 crate |
| `traits` | can-traits crate |
| `gui` | Tauri 后端 |
| `frontend` | React 前端 |
| `ci` | CI/CD 配置 |

### 示例

```bash
# 功能添加
git commit -m "feat(ds402): add homing mode support"

# Bug 修复
git commit -m "fix(sdo): handle timeout in segmented transfer"

# 文档更新
git commit -m "docs: add user guide for DS402 control"

# 重构
git commit -m "refactor(traits): unify backend initialization"

# 测试
git commit -m "test(core): add NMT state machine tests"
```

---

## 🔄 Pull Request 流程

### 1. 创建分支

```bash
# 从 main 分支创建功能分支
git checkout -b feature/amazing-feature

# 或修复分支
git checkout -b fix/bug-description
```

### 2. 开发

```bash
# 编写代码
# ...

# 运行测试
cargo test --workspace
cd frontend && npm test

# 检查代码
cargo clippy --workspace --all-features -- -D warnings
cargo fmt --check
```

### 3. 提交

```bash
git add .
git commit -m "feat(scope): description"
```

### 4. 推送

```bash
git push origin feature/amazing-feature
```

### 5. 创建 Pull Request

- 标题使用 Conventional Commits 格式
- 描述中说明：
  - 做了什么
  - 为什么这么做
  - 如何测试
  - 关联的 Issue（如有）
- 确保 CI 检查通过
- 请求 Code Review

### PR 模板

```markdown
## 描述

简要描述本次更改。

## 更改类型

- [ ] 新功能
- [ ] Bug 修复
- [ ] 文档更新
- [ ] 重构
- [ ] 测试
- [ ] 其他

## 测试

说明如何测试这些更改。

## 关联 Issue

Closes #123
```

---

## 🐛 报告问题

使用 [GitHub Issues](https://github.com/your-org/opencan/issues) 报告问题。

### Bug 报告模板

```markdown
## 环境信息

- OS: [e.g., Ubuntu 22.04, macOS 14, Windows 11]
- Rust: [e.g., 1.75.0]
- Node.js: [e.g., 18.17.0]
- OpenCAN: [e.g., v0.4.0]

## 问题描述

清晰描述遇到的问题。

## 复现步骤

1. 步骤 1
2. 步骤 2
3. 步骤 3

## 期望行为

描述期望的行为。

## 实际行为

描述实际的行为。

## 截图/日志

如果适用，添加截图或日志。
```

---

## 💡 功能建议

欢迎提出新功能建议！请使用 Issue 模板：

```markdown
## 功能描述

清晰描述建议的功能。

## 使用场景

说明这个功能解决什么问题。

## 建议实现

如果有想法，描述可能的实现方式。

## 替代方案

考虑过的其他解决方案。
```

---

## 📚 开发资源

### 项目架构

- [开发者指南](docs/developer-guide.md) — 详细架构说明
- [API 文档](https://docs.rs/opencan) — Rust crate API

### 参考资料

- [CANopen 规范](https://www.can-cia.org/)
- [CiA 301](https://www.can-cia.org/canopen/specification/) — 基础协议
- [CiA 402](https://www.can-cia.org/canopen/specification/) — 运动控制
- [Tauri 文档](https://tauri.app/)
- [React 文档](https://react.dev/)

---

## ❓ 获取帮助

- 💬 [GitHub Discussions](https://github.com/your-org/opencan/discussions)
- 📧 [邮件列表](mailto:dev@opencan.org)
- 🐛 [Issue Tracker](https://github.com/your-org/opencan/issues)

---

## 📄 许可证

贡献即表示你同意你的代码将在 MIT 或 Apache 2.0 许可证下发布。

---

<div align="center">
  <sub>感谢你的贡献！🎉</sub>
</div>
