# Tessera Macros

[![English][readme-en-badge]][readme-en-url]

[readme-en-badge]: https://img.shields.io/badge/README-English-blue.svg?style=for-the-badge&logo=readme
[readme-en-url]: https://github.com/tessera-ui/tessera/blob/main/tessera-macros/README.md

`tessera_macros` crate 为 [Tessera UI 框架](https://github.com/tessera-ui/tessera) 提供了过程宏。目前，它包含 `#[tessera]` 属性宏，这对于在 Tessera 框架中创建组件至关重要。

## 概述

`#[tessera]` 宏通过将常规 Rust 函数自动集成到框架的组件树中并注入必要的运行时功能，从而将其转换为 Tessera UI 组件。

## 特性

- **组件集成**: 自动将函数注册为 Tessera 组件树中的组件
- **运行时注入**: 在组件函数内提供对 `layout` 和 `input_handler` 函数的访问
- **简洁的语法**: 以最少的样板代码实现声明式组件定义
- **树管理**: 自动处理组件树节点的创建和清理

## 用法

### 基本组件定义

```rust
use tessera_macros::tessera;

#[tessera]
fn my_component() {
    // 你的组件逻辑在这里
    // 宏自动提供对以下内容的访问：
    // - layout: 用于自定义布局逻辑
    // - input_handler: 用于处理用户交互
}
```

### 带参数的组件

```rust
use tessera_macros::tessera;
use tessera_ui::{Callback, Prop};

#[derive(Clone)]
struct ButtonArgs {
    label: String,
    on_click: Callback,
}

impl Prop for ButtonArgs {
    fn prop_eq(&self, _other: &Self) -> bool {
        false
    }
}

#[tessera]
fn button_component(args: &ButtonArgs) {
    let _ = args;
    // 组件实现
    // 宏处理组件树集成
}
```

### 使用 Layout 和 Input Handler

```rust
use tessera_macros::tessera;
use tessera_ui::{ComputedData, LayoutInput, LayoutOutput, LayoutSpec, MeasurementError, Px};

#[derive(Clone, PartialEq)]
struct FixedLayout;

impl LayoutSpec for FixedLayout {
    fn measure(
        &self,
        _input: &LayoutInput<'_>,
        _output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        Ok(ComputedData {
            width: Px(100),
            height: Px(50),
        })
    }
}

#[tessera]
fn custom_component() {
    // 定义自定义布局行为
    layout(FixedLayout);

    // 处理用户交互
    input_handler(|_| {
        // 处理点击、按键等事件
    });
}
```

## 工作原理

`#[tessera]` 宏执行以下转换：

1. **组件注册**: 将函数以其名称添加到组件树中
2. **运行时访问**: 注入代码以访问 Tessera 运行时
3. **函数注入**: 在组件作用域内提供 `layout` 和 `input_handler` 函数
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
    // 组件树注册
    TesseraRuntime::with_mut(|runtime| {
        runtime.component_tree.add_node(ComponentNode { ... });
    });

    // 注入 layout 和 input_handler 函数
    let layout = |spec: impl LayoutSpec| { /* ... */ };
    let input_handler = |fun: impl Fn(InputHandlerInput) + Send + Sync + 'static| { /* ... */ };

    // 安全地执行原始函数体
    let result = {
        let closure = || {
            // 原始组件逻辑在这里
        };
        closure()
    };

    // 清理组件树
    TesseraRuntime::with_mut(|runtime| {
        runtime.component_tree.pop_node();
    });

    result
}
```

## 示例

### 简单计数器组件

```rust
use tessera_macros::tessera;
use tessera_ui::remember;
use tessera_components::{
    button::{ButtonArgs, button},
    text::{TextArgs, text},
};

#[tessera]
fn counter_component() {
    let count = remember(|| 0i32);

    button(&ButtonArgs::with_child(
        ButtonArgs::filled(move || count.with_mut(|c| *c += 1)),
        move || {
            let label = format!("Count: {}", count.get());
            text(&TextArgs::from(label));
        },
    ));
}
```

### 自定义布局组件

```rust
use tessera_macros::tessera;
use tessera_components::text::{TextArgs, text};
use tessera_ui::{ComputedData, LayoutInput, LayoutOutput, LayoutSpec, MeasurementError, Px};

#[derive(Clone, PartialEq)]
struct FixedLayout;

impl LayoutSpec for FixedLayout {
    fn measure(
        &self,
        _input: &LayoutInput<'_>,
        _output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        Ok(ComputedData {
            width: Px(120),
            height: Px(80),
        })
    }
}

#[tessera]
fn custom_layout() {
    layout(FixedLayout);

    // 子组件
    text(&TextArgs::from("Hello, World!"));
}
```

## 贡献

该 crate 是更大的 Tessera 项目的一部分。有关贡献指南，请参阅主 [Tessera 存储库](https://github.com/tessera-ui/tessera)。

## 许可证

该项目根据与主 Tessera 框架相同的条款获得许可。有关详细信息，请参阅 [LICENSE](../LICENSE) 文件。
