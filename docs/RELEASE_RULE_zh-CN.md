# Tessera 发布

采用语义化版本与 Conventional Commits：`BREAKING CHANGE:` 升级主版本，`feat:` 升级次版本，修复升级补丁版本。

预览发布计划：

```sh
rust-script scripts/release-package.rs
```

使用 `rust-script scripts/release-package.rs --check` 验证源码包、脚手架与安装后的资源，不执行发布。发布工作区之前，先发布带版本的 fork 依赖。

发布说明在准备发布时人工撰写。对发布计划中的每个包，阅读自上一个标签以来的历史，在其 `CHANGELOG.md` 顶部添加一节：

```md
## [v1.2.0] - 2026-11-16

### Breaking changes

- 改了什么，以及如何迁移。

### Added

- ...

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/name-v1.1.0...name-v1.2.0)
```

按需使用 `### Breaking changes`、`### Added`、`### Changed`、`### Fixed` 和 `### Performance`，归纳而非罗列提交。发布计划会输出每个包的对比链接。仅因依赖升级而发布的包用一行说明即可。

使用 `--major <package>` 指定主版本升级。在分支上运行 `--prepare` 校验发布说明并写入版本与依赖要求，通过标题为 `release: prepare workspace versions` 的 pull request 合并。如果发布前 `main` 又合入了改动，将它们整合到该分支、更新说明，并重新运行发布计划与 `--prepare`。随后在最新 `main` 的干净 checkout 中运行 `--execute`，验证并发布各包，然后推送标签。
