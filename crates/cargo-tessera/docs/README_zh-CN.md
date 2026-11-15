# cargo-tessera

Tessera UI 框架的命令行工具，用于项目脚手架、开发与构建。

## 安装

```bash
cargo install cargo-tessera
```

## 使用

### 创建新项目

```bash
cargo tessera new my-app
cd my-app
```

### 启动开发服务器

```bash
cargo tessera dev
```

`cargo tessera dev` 会在源码、清单、应用配置或资源变化时重建并重启应用。加上 `--verbose` 可查看底层 `cargo` 命令。

### 构建发布版本

```bash
cargo tessera build --release
```

交叉编译示例：

```bash
cargo tessera build --release --target x86_64-pc-windows-msvc
```

### 平台打包

桌面打包使用 `cargo tessera package -p my-app --release`，需先安装
`cargo-packager`。`--formats` 可选择 `nsis,wix`、`app,dmg` 或
`deb,appimage,pacman`；`--bin` 选择可执行文件。签名和安装器设置写在
`Cargo.toml` 的 `[package.metadata.packager]` 中。

### 配置

应用名称、标识符和图标写在 `tessera.toml` 的 `[app]` 中，未配置图标时使用
Tessera 标志；资源和本地化分别写在 `[assets]` 和 `[i18n]` 中。版本取自 Cargo。
完整说明见[配置](../README.md#configuration)。权限、SDK 版本、entitlements 和签名
写在各平台自己的文件里。

### iOS 与 macOS（Xcode）

需要 macOS、Xcode 和 XcodeGen（`brew install xcodegen`）：

```sh
# 在 gen/apple 生成 Xcode 项目（仅需一次）
cargo tessera apple init -p my-app

# 构建模拟器应用，或使用 --device 构建设备 IPA
cargo tessera ios build -p my-app
```

`gen/apple` 属于你的项目，应提交到版本库并在 Xcode 中编辑，其中包含 iOS 和 macOS
两个 target。cargo-tessera 每次只重写 `tessera.xcconfig` 和 `Tessera.xcassets`。
详见[英文文档](../README.md#build-for-ios-and-macos-with-xcode)。

### Android 构建

请安装 Android SDK/NDK、Gradle 8.14 支持的 JDK（17 或 21），并确保 `adb` 在 PATH 中。

```bash
# 在 gen/android 生成 Gradle 项目（仅需一次）
cargo tessera android init

# 构建 APK 或 AAB
cargo tessera android build --release --format apk

# 在设备上安装运行，源码变化时重新部署
cargo tessera android dev --device 8cd1353b
```

`gen/android` 属于你的项目：权限写在 `AndroidManifest.xml`，`minSdk` 和依赖写在
`app/build.gradle.kts`。cargo-tessera 只重写 `.gitignore` 中列出的文件，详见
[英文文档](../README.md#build-for-android)。

## 许可证

Tessera 采用 [MIT 许可证](../../../LICENSE-MIT) 或 [Apache 2.0 许可证](../../../LICENSE-APACHE)双重许可。
