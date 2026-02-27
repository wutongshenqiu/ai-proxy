验证代码质量，提交并推送。Argument $ARGUMENTS: commit message（可选）。

Steps:
1. Run `make lint` — 发现问题则先运行 `make fmt` 修复格式，再检查 clippy 问题并修复
2. Run `make test` — 发现失败则修复
3. 检查改动是否涉及以下文件，如有则提醒同步文档:
   - crates/core/src/provider.rs 或 config.rs 变更 → 检查 docs/reference/types/ 是否需要更新
   - crates/server/src/handler/ 或 lib.rs 路由变更 → 检查 docs/reference/api-surface.md
   - crates/provider/src/ 新增 executor → 检查 docs/playbooks/add-provider.md
   - crates/translator/src/ 新增翻译器 → 检查 docs/playbooks/add-translator.md
4. 检查是否有关联的 Spec — 如有活跃 Spec，确认 status 是否需要更新
5. `git add` 改动文件（排除 config.yaml / .env 等敏感文件）
6. 如果 $ARGUMENTS 非空，用其作为 commit message；否则从分支名 + 改动推导（conventional commit 格式: feat:/fix:/docs:/refactor:/test:/chore:）
7. `git commit`
8. `git push`（如远程分支不存在则 `git push -u origin HEAD`）
9. 报告推送结果
