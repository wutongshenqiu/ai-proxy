管理 Spec 生命周期。Argument $ARGUMENTS: 子命令（create/list/status/advance/td）。

子命令:

1. `create "标题"` — 创建新 Spec:
   - 读取 docs/specs/_index.md 确定下一个编号 (SPEC-NNN)
   - 在 docs/specs/active/ 下创建 SPEC-NNN/ 目录
   - 从 docs/specs/_templates/prd.md 复制模板，填充编号和标题
   - 在 docs/specs/_index.md 的表中注册，Status 设为 `Draft`
   - 输出创建结果和下一步指引

2. `list [active|completed|all]` — 列出 Specs:
   - 读取 docs/specs/_index.md
   - 按状态过滤输出（默认 `active`）
   - 显示: Spec ID | 标题 | Status | Location

3. `status SPEC-NNN` — 查看 Spec 详情:
   - 读取对应 Spec 目录下的 prd.md 和 technical-design.md（如存在）
   - 输出完整状态信息、摘要、相关代码路径

4. `advance SPEC-NNN` — 推进 Spec 到下一阶段:
   - 读取当前状态，推进到下一阶段
   - 更新 docs/specs/_index.md
   - 状态流转: Draft → Active → Completed
   - Active → Completed 时将目录从 active/ 移动到 completed/

5. `td SPEC-NNN` — 创建 Technical Design:
   - 从 docs/specs/_templates/technical-design.md 复制模板到 Spec 目录
   - 填充编号、标题
   - 如果 PRD 已有内容，基于 PRD 的 Goals 和 User Stories 预填充 TD 的 Overview

示例:
```
/spec create "WebSocket 支持"
/spec list active
/spec list all
/spec status SPEC-008
/spec advance SPEC-008
/spec td SPEC-008
```
