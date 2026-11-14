# Tessera 发布

采用语义化版本与 Conventional Commits：`BREAKING CHANGE:` 升级主版本，`feat:` 升级次版本，修复升级补丁版本。

预览发布计划：

```sh
rust-script scripts/release-package.rs
```

使用 `rust-script scripts/release-package.rs --check` 验证源码包、脚手架与安装后的资源，不执行发布。发布工作区之前，先发布带版本的 fork 依赖。

使用 `--major <package>` 指定主版本升级。`--prepare` 写入版本、依赖要求与更新日志以供审核。`--execute` 从干净 checkout 验证并发布包，成功后推送发布提交与标签。
