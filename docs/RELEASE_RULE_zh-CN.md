# Tessera Release Rule

[![English][release-en-badge]][release-en-url]

[release-en-badge]: https://img.shields.io/badge/RELEASE%20RULE-English-blue.svg?style=for-the-badge&logo=release
[release-en-url]: RELEASE_RULE.md

## 版本号规则

版本号由三部分组成：`主版本号.次版本号.修订号`，例如 `1.0.0`。

- **主版本号**：破坏性变更 (`BREAKING CHANGE:`) 或 roadmap 彻底完成时（通过 `--major` 标志手动执行）增加。
- **次版本号**：任何功能更新 (`feat:`) 时增加。
- **修订号**：任何 bug 修复或者小的改进时增加。

## 发布流程

必须使用 `scripts/release-package.rs` 脚本发布。它会根据 Conventional Commits 自动处理版本号的更新、打包、生成更新日志并推送。发布流程仅限于以下三个包：`tessera-ui`、`tessera-ui-basic-components` 和 `tessera-ui-macros`。

### 工作原理

1. **自动版本分析**：脚本会分析所有可发布包（`tessera-ui`, `tessera-ui-basic-components`, `tessera-ui-macros`）自上次 tag 以来的 Git 历史。
   - 在消息体中包含 `BREAKING CHANGE` 的提交将导致**主版本号**更新。
   - `feat:` 提交将导致**次版本号**更新。
   - 任何其他类型的提交（`fix:`、`refactor:` 等）将导致**修订号**更新。
2. **依赖传递**：如果一个底层包被更新，任何依赖于它的可发布包也将自动进行一次**修订号**更新，以确保版本一致性。发布顺序由依赖图的拓扑排序决定。
3. **手动主版本更新**：要进行主版本更新，您必须使用 `--major <package-name>` 标志。此操作保留给路线图里程碑。

### 使用方法

1. 建议先 dry run 发布脚本，看看是否符合预期。脚本默认以 dry-run 模式运行。

   示例：基于 `tessera-ui` 的变更触发一次发布分析。如果 `tessera-ui` 包含 `feat` 提交，它的次版本号将会增加，而依赖于它的包的修订号也会增加。

   ```bash
   rust-script scripts/release-package.rs
   ```

   示例：为 `tessera-ui` 强制进行一次主版本更新。这也会为依赖于它的包触发修订号更新。

   ```bash
   rust-script scripts/release-package.rs --major tessera-ui
   ```

   脚本会输出一个清晰的“发布计划”表格，显示哪些包将被发布以及它们的版本号将如何变动。

2. 如果 dry run 没有问题，您可以添加 `--execute` 标志来执行实际的发布。

   ```bash
   # 基于自动分析执行发布
   rust-script scripts/release-package.rs --execute

   # 为 tessera-ui 执行一次主版本更新的发布
   rust-script scripts/release-package.rs --major tessera-ui --execute
   ```

这会执行实际的发布操作，包括更新版本号、生成更新日志、发布到 crates.io，并推送到远程仓库。
