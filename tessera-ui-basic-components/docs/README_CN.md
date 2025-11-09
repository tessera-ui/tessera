# Tessera basic components

[![English][readme-en-badge]][readme-en-url]

[readme-en-badge]: https://img.shields.io/badge/README-English-blue.svg?style=for-the-badge&logo=readme
[readme-en-url]: https://github.com/tessera-ui/tessera/blob/main/tessera-ui-basic-components/README.md

`tessera-ui-basic-components` 提供了一组基础 UI 组件，用于构建常见的用户界面。这包括按钮、文本、布局容器等。

## 已有的组件

### 布局组件

- `column`: 垂直布局容器
- `row`: 水平布局容器
- `boxed`: 带边框的容器
- `spacer`: 可调节大小的空白占位符

### 基础组件

- `surface`: 可定制的表面组件，是非玻璃类组件的基础
- `button`: 可点击按钮
- `switch`: 可切换的开关
- `slider`: 滑动条
- `progress`: 进度条
- `fluid_glass`: 可定制的玻璃表面组件，是玻璃类组件的基础
- `glass_button`: 玻璃风格可点击按钮
- `glass_switch`: 玻璃风格可切换开关
- `glass_slider`: 玻璃风格滑动条
- `glass_progress`: 玻璃风格进度条
- `text`: 文本显示，支持系统字体和彩色emoji
- `image`: 图片显示(支持 AVIF, BMP, DDS, EXR, FF, GIF, HDR, ICO, JPEG, PNG, PNM, QOI, TGA, TIFF, WebP)
- `icon`: 复用向量/栅格管线的图标封装，提供统一的尺寸和可选的 tint
- `icon_button`: 封装 `button` 与 `icon`，方便构建图标按钮
- `glass_icon_button`: 基于 `glass_button` 的玻璃风格图标按钮
- `checkbox`: 复选框
- `tabs`: 选项卡组件
- `text_editor`: 多行文本编辑器
- `bottom_nav_bar`: 底部导航栏
- `side_bar_provider`: 侧边弹出栏，提供了玻璃和非玻璃两种风格
- `dialog_provider`: 对话框，提供了玻璃和非玻璃两种风格
- `bottom_sheet_provider`: 底部弹出栏，提供了玻璃和非玻璃两种风格
- `scrollable`: 可滚动容器，支持垂直和水平滚动
