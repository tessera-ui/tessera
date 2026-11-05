# Tessera 发布

采用语义化版本与 Conventional Commits：`BREAKING CHANGE:` 升级主版本，`feat:` 升级次版本，修复升级补丁版本。

预览发布计划：

```sh
rust-script scripts/release-package.rs
```

使用 `--major <package>` 指定主版本升级。添加 `--execute` 将审核后的计划发布到 crates.io，并推送发布提交与标签。
