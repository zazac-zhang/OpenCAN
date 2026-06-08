# OpenCAN TODO

> 审查日期：2026-06-08（修正版）

## 🔧 高优先级

- [ ] **SDO Server block transfer 实现** — `canopen-core/src/protocol/sdo/server.rs:43`，Client 端 `block_upload()` 已实现，Server 端缺失
- [ ] **前端 ESLint 配置** — 缺少 `.eslintrc.*` 或 `eslint.config.*`，`npm run lint` 无法正常工作
- [ ] **lib.rs 文档注释修正** — ~~Kvaser/PCAN/ZLG 标记为 Stub~~ ✅ 已修正

## 🔧 中优先级

- [ ] **NetworkDiagnostics 模块** — `canopen-master` 中的总线统计、错误追踪（lib.rs 注释中标记为 planned，尚无实现）

## 📋 计划中

- [ ] **SDO Server 增强** — 访问控制策略完善、多并发会话支持
- [ ] **PDO 动态映射运行时验证** — 映射参数变更后的合法性校验
- [ ] **EDS 解析器健壮性** — 边界情况处理、错误提示优化
- [ ] **前端 Vitest 单元测试** — 补充关键组件和 hooks 的测试覆盖
- [ ] **前端 Playwright E2E 测试** — 核心流程的端到端测试
- [ ] **Kvaser CAN FD 支持** — 当前仅 Classic CAN，FD 支持待实现
- [ ] **PCAN CAN FD 支持** — 当前仅 Classic CAN，FD 支持待实现

## ✅ 已实现（审查确认）

- [x] **Kvaser 后端** — 完整 FFI 实现（`canlib32.dll`），`CanBus` + `CanBusFactory`
- [x] **PCAN 后端** — 完整 FFI 实现（`PCANBasic.dll`），`CanBus` + `CanBusFactory`
- [x] **ZLG 后端** — 完整 FFI 实现（`zlgcan.dll`/`libzlgcan.so`），含 CAN FD 支持
- [x] **SDO Client block transfer** — `block_upload()` 已实现
- [x] **Icon RGBA 格式** — 将 grayscale PNG 转换为 RGBA，修复 `tauri::generate_context!()` 编译失败
- [x] **CI 前端检查** — ci.yml 新增 frontend job（typecheck + lint + build）
- [x] **Release 自动化** — release.yml 实现 4 平台 Tauri 构建发布
- [x] **Agent skills 配置** — docs/agents/ 配置完成
