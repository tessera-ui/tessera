# 为 tessera 做贡献

[![English][contributing-en-badge]][contributing-en-url]

[contributing-en-badge]: https://img.shields.io/badge/CONTRIBUTING-English-blue.svg?style=for-the-badge&logo=contributing
[contributing-en-url]: ../CONTRIBUTING.md

感谢您对 `tessera` 的兴趣！我们欢迎任何形式的贡献，包括但不限于代码、文档、issue。

## 开发工具

如果您需要为 `tessera` 做代码贡献，除最新 nightly rust 以外，强烈建议安装以下工具:

- [`xbuild`](https://github.com/rust-mobile/xbuild) 我们用它来构建和测试 Android 版本。未来也可能用它适配 iOS 版本。
- [`rust-script`](https://rust-script.org/#installation) 我们用它来运行[一些 rust 脚本](scripts)。对开发有帮助。

## 代码贡献规范

为了确保代码质量和一致性，保证仓库整洁，请遵循以下规范：

## 入门

您可以通过以下二种方式之一来设置您的开发环境：

1. **Nix (推荐)**: 在 Linux 和 macOS 上一键完成环境配置。
2. **手动设置**: 手动安装rust和其他依赖。

### 选项 A - Nix 包管理器 (推荐)

如果您安装了 [Nix](https://nixos.org/download.html)，您可以通过运行以下命令来获取一个包含所有依赖的开发环境：

```bash
nix develop            # 桌面开发 shell
nix develop .#android  # 安卓开发 shell
```

### 选项 B - 手动设置

如果您倾向于手动设置环境，请按照下面对应您的操作系统的说明进行操作。

#### 1. 安装 Rust

首先，请访问 [rustup.rs](https://rustup.rs/) 按照官方指南安装 Rust 工具链。

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 2. 安装系统依赖

##### 针对 Linux

对于已经有桌面环境的 Linux 用户，您大概率不需要这一步，以下内容是针对那些无桌面环境的用户。

如果您目前没有桌面环境，您需要选择安装 X11 或 Wayland 的开发包。通常您只需要安装其中一套。

- **使用 X11**:

  - _Debian/Ubuntu_: `sudo apt install libx11-dev libxrandr-dev libxcursor-dev`
  - _Arch/Manjaro_: `sudo pacman -S libx11 libxrandr libxcursor`
  - _Fedora_: `sudo dnf install libX11-devel libXrandr-devel libXcursor-devel`

- **使用 Wayland**:
  - _Debian/Ubuntu_: `sudo apt install libwayland-dev libxkbcommon-dev`
  - _Arch/Manjaro_: `sudo pacman -S wayland libxkbcommon`
  - _Fedora_: `sudo dnf install wayland-devel libxkbcommon-devel`

##### 针对 macOS

如果您已经安装了 Xcode 命令行工具，则无需额外的系统依赖。

##### 针对 Windows

安装 Rust 后无需额外的依赖。所需的工具已包含在 Windows 的标准 Rust 安装中。

#### 3. 安装 Vulkan SDK (可选)

Vulkan SDK 是**可选的**。仅当您需要进行着色器验证或调试时才需要它。对于常规的 UI 开发，您可以跳过此步骤。

如果您需要，可以从 [vulkan.lunarg.com](https://vulkan.lunarg.com/) 下载，运行安装程序，并按照其安装后的说明进行操作。

### 代码使用的语言

任何代码、包括在代码中的文档(如 `rustdoc` 注释)和注释都必须使用英语编写，除非功能上有必要专门指出这个词。

### 代码检查

- 请确保您的代码格式符合本项目规范，规则如下

  - `rustfmt edition 2024` 默认规则
  - `5条 use 导入规则`：
    1. 导入分为四组，严格按照以下顺序排列：
       - 第 1 组：标准库（`std`、`core`、`alloc`）
       - 第 2 组：第三方 crate
       - 第 3 组：当前 crate 根（`crate::`）
       - 第 4 组：当前 crate 的子模块（`super::`、`self::`）
    2. 不同组之间必须有且仅有一个空行。
    3. 同组内的导入必须连续排列，中间不得有空行。
    4. 每组内的导入需按字母顺序排序。
    5. 同一根路径下的导入应合并为一个 `use` 语句。

- 格式化工具

  - 您可以使用 `cargo fmt` 命令来格式化代码。这会自动应用上述的第一条格式化规范。
  - 但是，建议(于项目根目录下)始终使用

    ```bash
    rust-script scripts/check-imports.rs . --fix
    ```

    来检查和修复导入规则，它还会顺便调用`rustfmt`进行格式化。因为它会直接应用上述的所有格式化规范，并且不会忽略脚本文件（而 `cargo fmt` 只会格式化 `Cargo.toml` 管理的内容）。

  - Nix 用户：只需在 `nix develop` 环境中输入 `fmt` 即可——这是一个智能别名，可以从任何目录运行上述相同的命令。

### 提交规范

- 请确保您的提交信息清晰、简洁，并且描述了所做的更改。
- 提交格式必须遵循 [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0) 规范。我们对 scope 无特殊要求，你可以不使用 scope，但是建议用于标识更改的范围或功能。
- **破坏性变更**：如果您的提交引入了破坏性变更，您**必须**在提交正文或页脚中包含 `BREAKING CHANGE:`。这是必须的，因为我们的发布脚本依赖此标志来自动触发**主版本号 (Major)** 更新。
- 推送前，请确保
  - 您的代码通过了所有测试(`cargo test`)。
  - 您已更新相关文档（如果适用）。
  - 您的代码符合[代码检查规范](#代码检查)。

## 安卓构建特别说明

由于 `NativeActivity` 的限制，为安卓构建需要一些特殊的考量。我们使用 [`xbuild`](https://github.com/rust-mobile/xbuild) 来处理交叉编译和打包的复杂性。

- **先决条件**: 确保您已正确设置 Android NDK 和 SDK。
- **问题排查**: 如果您在安卓构建过程中遇到问题，`xbuild` 提供了一个诊断工具。运行以下命令来检查您的环境并识别潜在问题：

```bash
x doctor
```

- **Nix 用户**: 安卓开发 shell (`nix develop .#android`) 中已预先配置好了 `xbuild`。

请注意，安卓支持仍处于实验阶段，您可能会遇到一些问题。

## 文档贡献规范

为了确保文档的清晰和一致性，保证项目的协作可能，请遵循以下规范：

### 文档使用的语言

任何文档内容，包括 README、Wiki 页面和其他文档，都必须至少有英语版本，且以英语版本为主。其他 i18n 版本可以在英语版本的基础上进行翻译，但必须确保英语版本始终存在，且其内容是最新的。

注意，因为`cargo doc`生成的文档完全不支持 i18n，我们也不可能把所有版本的文档都放在代码里，所以请不要在代码中添加任何非英语的文档内容。

### 文档翻译规范

- 文档翻译必须基于英语版本进行，确保翻译内容与英语版本保持一致。
- 英文版本的文档不需要加 i18n 后缀，其他语言版本的文档需要在文件名中添加对应的语言后缀，如 `README_zh-CN.md`。
- 建议翻译过的文档不是直接放在英语版本所在目录，而是其相对目录下的`docs`文件夹中，除非必须这样做。

### 文档格式

最好能通过 markdown lint，不过我们并不强制要求。请确保文档内容清晰、易读，并且格式一致。

### 文档提交

- 对本仓库的直接提交/pr 规范请查看[提交规范](#提交规范)
- 对相关仓库，如官网、Wiki 等的提交，请遵循各自仓库的贡献指南。

## 许可证

我们默认您的贡献遵循项目的双许可证条款。本项目采用 [MIT 许可证](../LICENSE-MIT) 或 [Apache 许可证 2.0](../LICENSE-APACHE) 双许可证模式，您可以选择其中任意一种许可证。

通过向本项目提交贡献，您同意您的贡献将在与项目相同的许可证条款下发布，即 MIT OR Apache-2.0 双许可证。这意味着：

- 您的贡献将同时在 MIT 许可证和 Apache 许可证 2.0 下可用
- 用户可以选择在这两个许可证中的任意一个下使用您的贡献
- 您确认您有权在这些许可证条款下提供您的贡献
