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

`cargo tessera dev` 会持续监听 `src/`、`Cargo.toml` 以及（若存在）`build.rs`，保存后立即重新编译并运行。加上 `--verbose` 可查看底层 `cargo` 命令。

### 构建发布版本

```bash
cargo tessera build --release
```

交叉编译示例：

```bash
cargo tessera build --release --target x86_64-pc-windows-msvc
```

### Android 构建（实验性）

先安装 [`xbuild`](https://github.com/rust-mobile/xbuild)（`cargo install xbuild --features vendored`），并在项目 `Cargo.toml` 中加入：

```toml
[package.metadata.tessera.android]
package = "com.example.myapp"
arch = "arm64"
format = "apk"
```

Android 相关命令位于独立子命令下：

```bash
# 使用 xbuild 产出 APK/AAB
cargo tessera android build --release --format apk

# 在设备或模拟器上运行，必须指定 --device（可通过 `x devices` 查看）
cargo tessera android dev --device adb:1234
```

`--arch`、`--package`、`--format` 可覆盖元数据。如果 `x build` / `x run` 失败，请安装 `xbuild` 或执行 `x doctor` 进行排查。

## 命令速览

- `cargo tessera new <name>`：创建 Tessera 项目
- `cargo tessera dev`：启动桌面热重载开发
- `cargo tessera build`：桌面构建（支持 `--release` / `--target`）
- `cargo tessera android <build|dev>`：Android 构建与运行辅助

## 许可证

可选择 [MIT](../../LICENSE) 或 [Apache-2.0](../../LICENSE)。
