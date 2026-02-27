Review a pull request. Argument $ARGUMENTS: PR number.

Steps:
1. Get PR info: `gh pr view $ARGUMENTS --json title,body,headRefName,baseRefName,files`
2. Get diff (safe mode, no checkout): `gh pr diff $ARGUMENTS`
3. Review the diff against project conventions (CLAUDE.md + AGENTS.md):
   - **Code style**: Rust Edition 2024 conventions, proper error handling (thiserror), async-trait usage
   - **Type safety**: proper Rust types, no unwrap() in production code, proper error propagation
   - **API conventions**: consistent endpoint patterns, proper error response format (ProxyError)
   - **Test coverage**: new code should have corresponding tests
   - **Security**: no hardcoded API keys, no secrets in code, proper auth checks
   - **Provider compatibility**: changes should not break existing provider support
   - **Spec & Reference doc sync**: do docs/reference/ updates needed? Is there a related Spec?
   - **Config compatibility**: config.yaml changes should be backward compatible
4. Output a structured review:

```
## PR Review: #{PR_NUMBER} — {title}

### Summary
<brief description of what the PR does>

### Findings

#### 🔴 Critical
<issues that must be fixed before merge>

#### 🟡 Warning
<issues that should be addressed>

#### 🔵 Info
<suggestions and observations>

### Doc Impact
<list any docs/reference/ or docs/specs/ files that need updating>

### Verdict
<APPROVE / REQUEST_CHANGES / COMMENT>
```
