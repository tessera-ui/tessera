# 为 Tessera 做贡献

[English](../CONTRIBUTING.md)

## 准备环境并验证改动

1. 安装 [Rust](https://rustup.rs/)，运行 `git submodule update --init --recursive`，再运行 `git -C crates/tessera-cosmic-text lfs pull`（需要 [Git LFS](https://git-lfs.com/)）。常规清理不改动上游 fork。
2. Linux 或 macOS 可使用 `nix develop`，也可手动安装平台依赖。Linux 需要 X11（`libx11`、`libxrandr`、`libxcursor`）或 Wayland（`wayland`、`libxkbcommon`）开发包；macOS 需要 Xcode Command Line Tools。
3. 安装 [rust-script](https://rust-script.org/#installation)，然后在仓库根目录运行检查：

   ```sh
   rust-script scripts/check-imports.rs . --check
   cargo test --workspace
   ```

Android 环境见 [CLI 指南](../crates/cargo-tessera/docs/README_zh-CN.md#android-构建)。打包与平台检查见[维护脚本](../scripts/README.md)。

## 提交改动

- 从短期分支向 `main` 提交 pull request。CI 通过后方可 squash merge。
- 标题采用 [Conventional Commits](https://www.conventionalcommits.org/) 格式。破坏性变更在标题中加 `!`，并在描述中以 `BREAKING CHANGE:` 说明变更及迁移办法。
- 遵循 [AGENTS.md](../AGENTS.md)。先更新英文文档，再同步翻译。
- 贡献按 [MIT](../LICENSE-MIT) 或 [Apache-2.0](../LICENSE-APACHE) 授权，由接收者任选其一。只提交你有权按这些条款授权的内容。

维护者请参阅[发布指南](RELEASE_RULE_zh-CN.md)。
