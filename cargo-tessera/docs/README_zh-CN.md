# cargo-tessera

`cargo-tessera` 是 Tessera UI 的命令行工具，用于快速创建项目、启动开发服务器以及构建桌面 / Android 版本。

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

`cargo tessera dev` 会持续监听 `src/`、`Cargo.toml` 以及（若存在）`build.rs`，保存后立即重建并重启应用。加上 `--verbose` 可查看底层 `cargo` 命令。

### 构建发布版本

```bash
cargo tessera build --release
```

交叉编译示例：

```bash
cargo tessera build --release --target x86_64-pc-windows-msvc
```

### Android 构建（实验性）

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

## 命令速览

- `cargo tessera new <name>`：创建 Tessera 项目
- `cargo tessera dev`：启动桌面自动重建 / 重启开发
- `cargo tessera build`：桌面构建（支持 `--release` / `--target`）
- `cargo tessera android <build|dev>`：Android 构建与运行辅助

## 许可证

Tessera 采用 [MIT 许可证](https://github.com/tessera-ui/tessera/blob/main/LICENSE) 或 [Apache 2.0 许可证](https://github.com/tessera-ui/tessera/blob/main/LICENSE)双重许可。
