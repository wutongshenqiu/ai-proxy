创建 Pull Request。Argument $ARGUMENTS: 无（使用当前分支）。

Steps:
1. Run `make lint` — 发现问题则修复
2. Run `make test` — 确保测试通过
3. 检查改动是否涉及文档同步:
   - 涉及 crates/core/src/ 类型变更 → 提醒检查 docs/reference/types/
   - 涉及 crates/server/src/handler/ 或路由变更 → 提醒检查 docs/reference/api-surface.md
   - 涉及新 provider/translator → 提醒检查 docs/reference/architecture.md
   如有未同步项，先修复再继续。
4. 从分支名和 commit 历史推导 PR 标题（conventional commit 格式，70字符内）
5. 生成 PR body:

```
## Summary
<1-3 bullet points summarizing changes>

## Changes
<按 crate 分组列出主要改动>

## Spec & Reference Doc Impact
<列出涉及的 Spec 和需要更新的文档，或 "None">

## Test Plan
- [ ] `make lint` passes
- [ ] `make test` passes
- [ ] <specific test scenarios>
```

6. `git push -u origin HEAD`（如需要）
7. `gh pr create --title "..." --body "..."`
8. 报告 PR URL

## Post-Create: Merge Checklist

PR 创建后、合并前:
1. `gh pr checks {PR#}` 确认 CI 通过
2. 确认 merge 状态: `gh pr view {PR#} --json mergeStateStatus,mergeable`
3. 合并: `gh pr merge {PR#} --squash --delete-branch`
