运行测试。Argument $ARGUMENTS: all/unit/specific（default: all）。

模式:
- "all" — 运行全部测试: `cargo test --workspace`
- "unit" — 仅单元测试（不含集成测试）: `cargo test --workspace --lib`
- 其他值 — 作为测试过滤器: `cargo test --workspace $ARGUMENTS`

Steps:
1. Run `cargo check --workspace` — 先确保编译通过
2. 按模式执行测试
3. 如有失败:
   - 列出每个失败测试的名称和错误信息
   - 定位对应的源文件和测试文件
   - 分析失败原因（编译错误/断言失败/panic）
4. 汇总: 通过数 / 失败数 / 忽略数

示例:
```
/test                    # 全部测试
/test unit               # 仅单元测试
/test test_should_cloak  # 运行名称匹配的测试
/test cloak              # 运行 cloak 相关测试
```
