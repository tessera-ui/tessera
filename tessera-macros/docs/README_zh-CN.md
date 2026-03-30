# Tessera Macros

[![English][readme-en-badge]][readme-en-url]

[readme-en-badge]: https://img.shields.io/badge/README-English-blue.svg?style=for-the-badge&logo=readme
[readme-en-url]: https://github.com/tessera-ui/tessera/blob/main/tessera-macros/README.md

`tessera_macros` crate 为 [Tessera UI 框架](https://github.com/tessera-ui/tessera) 提供过程宏。当前主要公开能力是 `#[tessera]` 属性宏，以及 entry/shard 相关宏。

## 概述

`#[tessera]` 宏通过将普通 Rust 函数集成到框架的组件树中并建立所需的构建 / replay 上下文，从而将其转换为 Tessera UI 组件。

## 特性

- **组件集成**: 自动将函数注册为 Tessera 组件树中的组件
- **运行时上下文**: 建立框架所需的组件/节点运行时上下文
- **简洁的语法**: 以最少的样板代码实现声明式组件定义
- **树管理**: 自动处理组件树节点的创建和清理

## 用法

### 基本组件定义

```rust
use tessera_macros::tessera;

#[tessera]
fn my_component() {
    // 你的组件逻辑在这里
}
```

### 带参数的组件

```rust
use tessera_macros::tessera;
use tessera_ui::Callback;

#[tessera]
fn button_component(label: String, on_click: Callback) {
    let _ = (label, on_click);
    // 组件实现
}
```

公开组件通过宏生成的 builder 语法调用：

```rust
button_component()
    .label("Confirm")
    .on_click(Callback::new(|| {}));
```

## 工作原理

`#[tessera]` 宏执行以下转换：

1. **组件注册**: 将函数以其名称添加到组件树中
2. **运行时访问**: 注入构建与 replay 所需的内部运行时代码
3. **上下文注入**: 建立 Tessera 在构建与 replay 阶段使用的内部组件上下文
4. **树管理**: 处理从组件树中推入和弹出节点
5. **错误安全**: 包装原始函数体，以防止提前返回破坏组件树

### 宏应用之前

```rust
#[tessera]
fn my_component() {
    // 组件逻辑
}
```

### 宏应用之后 (概念上)

```rust
fn my_component() {
    // 建立构建上下文、注册节点并附着 replay 元数据。
    // 在受管的组件作用域内执行原始函数体。
}
```

## 示例

### 简单计数器组件

```rust
use tessera_macros::tessera;
use tessera_ui::remember;
use tessera_components::{
    button::button,
    text::text,
};

#[tessera]
fn counter_component() {
    let count = remember(|| 0i32);

    button()
        .on_click(Callback::new(move || count.with_mut(|c| *c += 1)))
        .child(|| {
            let label = format!("Count: {}", count.get());
            text().content(label);
        });
}
```

## 贡献

该 crate 是更大的 Tessera 项目的一部分。有关贡献指南，请参阅主 [Tessera 存储库](https://github.com/tessera-ui/tessera)。

## 许可证

该项目根据与主 Tessera 框架相同的条款获得许可。有关详细信息，请参阅 [LICENSE](../LICENSE) 文件。
