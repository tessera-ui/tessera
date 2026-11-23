# 发布 Tessera

[English](RELEASE_RULE.md)

1. 确认所需的 fork 版本已发布。运行 `rust-script scripts/release-package.rs` 预览计划。发布遵循语义化版本与 Conventional Commits；用 `--major <package>` 指定主版本升级。
2. 在计划所列各包的 `CHANGELOG.md` 顶部添加发布说明。使用计划给出的 `## [vX.Y.Z] - YYYY-MM-DD` 标题和版本对比链接。概述用户可见的变化，并说明破坏性变更的迁移办法；仅更新依赖时，一行即可。
3. 运行 `rust-script scripts/release-package.rs --check` 验证而不发布。在分支上用 `--prepare` 运行脚本，并提交标题为 `release: prepare workspace versions` 的 pull request。若合并前 `main` 有新的改动，整合后更新说明并重新预览、准备发布。
4. 在最新 `main` 的干净检出中运行 `rust-script scripts/release-package.rs --execute`，验证、发布并推送标签。
