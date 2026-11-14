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
`deb,appimage,pacman`；`--bin` 选择可执行文件。各平台统一从
`tessera-app.toml` 读取 `name`、`identifier` 和 `icon`。图标支持方形 SVG（推荐）
和 PNG，版本取自 Cargo。详见[应用打包配置](../README.md#package-a-desktop-application)。

macOS 安装 Xcode 后可构建 iOS：

```sh
cargo tessera ios build -p my-app
cargo tessera ios build -p my-app --release --device \
  --signing-identity "Apple Development: ..." \
  --provisioning-profile app.mobileprovision
```

默认输出当前主机的 Simulator `.app`，`--device` 输出 IPA。
设备安装需要匹配的证书、描述文件及 entitlements。
`[package.metadata.tessera.ios]` 可配置 `info-plist`、`entitlements` 和
`resources`，路径相对于包目录。
`IPHONEOS_DEPLOYMENT_TARGET` 可覆盖默认的 iOS 15 最低版本。

### Android 构建

请确保已安装 Android SDK/NDK，并且 `adb` 已加入 PATH。

Android 相关命令位于独立子命令下：

```bash
# 初始化 Android Gradle 项目（仅需一次）
cargo tessera android init

# 使用 Gradle 产出 APK/AAB
cargo tessera android build --release --format apk

# 在设备或模拟器上运行，必须指定 --device（可通过 `adb devices` 查看）
cargo tessera android dev --device 8cd1353b
```

## 许可证

Tessera 采用 [MIT 许可证](../../../LICENSE-MIT) 或 [Apache 2.0 许可证](../../../LICENSE-APACHE)双重许可。
