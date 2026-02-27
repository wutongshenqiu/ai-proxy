审查文档与代码的一致性。Argument $ARGUMENTS: full/quick/types/api/specs（default: quick）。

范围说明:
- "quick": 仅检查 reference/types/ 类型定义 vs Rust 源码中的类型
- "full": reference/ 全量 + specs/completed/ 交叉检查 + 链接有效性
- "types": 逐一检查 docs/reference/types/ 下每个文件:
  - enums.md vs crates/core/src/provider.rs, config.rs, cloak.rs 中的枚举定义
  - config.md vs crates/core/src/config.rs 中的配置类型
  - provider.md vs crates/core/src/provider.rs + crates/provider/src/ 中的类型和 trait
  - errors.md vs crates/core/src/error.rs 中的 ProxyError 及 status_code 映射
- "api": API 端点一致性:
  - docs/reference/api-surface.md 端点表 vs crates/server/src/lib.rs 路由定义
  - 每个 handler 的实际参数、返回格式
- "specs": 每个 completed Spec 的 technical-design.md 与对应代码模块的关键声明对比

Steps:
1. 读取目标文档文件
2. 读取对应的 Rust 源码文件
3. 逐项对比: 字段名、类型、枚举变体、方法签名、默认值、serde 属性
4. 输出差异表:

| 差异项 | 文档位置 | 代码位置 | 文档值 | 代码值 | 操作建议 |
|--------|----------|----------|--------|--------|----------|

5. 检查文档内链接有效性（仅 full 模式）
6. 汇总: 总差异数、按严重度分类（错误/遗漏/过时）
