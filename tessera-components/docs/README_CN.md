# Tessera basic components

[![English][readme-en-badge]][readme-en-url]

[readme-en-badge]: https://img.shields.io/badge/README-English-blue.svg?style=for-the-badge&logo=readme
[readme-en-url]: https://github.com/tessera-ui/tessera/blob/main/tessera-components/README.md

`tessera-components` 提供了一组基础 UI 组件，用于构建常见的用户界面。这包括按钮、文本、布局容器等。

## Material 默认设置

`button().size(ButtonSize::Medium)` selects an Expressive size preset. Five
presets provide 32/40/56/96/136dp container heights, matching typography,
content insets, icon spacing, and outline thickness. `leading_icon(...)` and
`trailing_icon(...)` size their icons automatically. Use `content_padding(...)`
for per-edge insets or `padding(...)` for a uniform override. Explicit modifiers
continue to bound the container; disabling a clickable button preserves its
minimum touch target and geometry. The Basic Components page demonstrates the
five sizes and visual variants.

Use `.round()` or `.square()` for size-aware resting corners and pressed shape
motion. `.selected(value)` creates a controlled toggle button; update its state
in `on_click`. Filled, elevated, tonal, and outlined variants support toggling;
text buttons remain ordinary actions. Selection reverses the resting shape and
uses the corresponding theme colors. `.shape(...)` fixes a custom shape instead.
Shared `interaction_state(...)` supports previews of hover, focus, and press.

Constrained buttons reserve space for both icons before measuring the grouped
label slot. Labels wrap within the remaining width and can increase the visual
height above the preset minimum. Wrap multiple label elements in a row or column
when they need a layout other than stacking.

`icon_button()` now defaults to the Expressive filled, small, round, uniform
configuration. Use `.standard()` for a transparent action. `IconButtonSize`
provides five heights (32/40/56/96/136dp) and 20/24/24/32/40dp icons;
`IconButtonWidth::{Narrow, Uniform, Wide}` controls horizontal proportions.
Interactive targets remain at least 48dp when constraints permit, including
when disabled. `.selected(value)` creates a controlled toggle for all four
variants; update the value in `on_click`. Round and square presets exchange
resting shapes on selection and share a pressed shape without spring bounce.
Explicit `.shape(...)` suppresses automatic morphing. Standard and outlined
variants use recommended vibrant colors by default; `color_style(IconButtonColorStyle::Inherited)`
uses the surrounding content color for unselected icons and outlines.

`split_button_layout` keeps the trailing action visible when width is limited.
`SplitButtonSize` supplies five minimum heights, matching label typography,
leading/trailing icon sizes, and outline thickness. Content slots arrange their
children in a row. Use `.expanded(value)` on `split_trailing_button` with a menu
controller; this publishes disclosure and popup semantics, rounds the selected
trailing shape, and animates its menu icon. Resting menu icons use the documented
optical offsets and become geometrically centered when expanded. Explicit icon
rotation overrides inherited control motion. Shared `interaction_state(...)`
previews hover, focus, and press; `.shape(...)` fixes a custom outline. Visual
variant setters preserve an explicitly disabled state. The Material Components
page demonstrates all five sizes with working menus and four styles.

组件覆盖和待验证事项见 [Material 对齐清单](MATERIAL_ALIGNMENT.md)。
`floating_toolbar` 提供横向、纵向工具栏，支持标准色、强调色和展开时的附加操作。
`docked_toolbar` 提供全宽、64dp 高的停靠工具栏，操作间距在 4–32dp 之间自适应。
`navigation_drawer` 与 `navigation_drawer_item` 提供导航容器、选中目的地、前置图标和尾部徽标。
示例的 Material Components 页面包含这两类组件的交互演示。

`floating_action_button` 新增 80dp 的 `Medium` 尺寸，建议图标为 28dp。
`extended_floating_action_button("Action")` 提供 56/80/96dp 三档 Expressive 尺寸，
自动设置图标、排版和间距，并支持标签展开动画；收起时仍保留无障碍名称。
没有图标时保持展开。`MaterialShapes::large_increased` 提供中号 FAB 的 20dp 圆角；
自定义形状字面量应通过 `..Default::default()` 继承新增的形状角色。

`floating_action_button_menu` 在页面内容上方提供底部对齐的创建菜单。
通过 `FloatingActionButtonMenuItem` 设置操作，可用 `FloatingActionButtonMenuState`
控制开关。操作项依次展开；点击外部、按 Escape 或选择操作会关闭菜单。
支持主色、次色和第三色，短窗口中列表可滚动且关闭按钮保持固定。
`SemanticsArgs::expanded` 和 `has_popup` 提供展开状态与弹出类型语义；
自定义语义字面量应包含 `..Default::default()`。

`loading_indicator` 提供 Expressive 七形状循环动画，也可通过 `.progress(value)`
显示从圆形到柔和爆发形状的确定进度。`.contained(true)` 启用主题底色，
`.paused(true)` 暂停循环动画并停止帧请求。默认尺寸为 48dp；几何数据共享，
动画复用 GPU 实例缓冲区。原生和 WebAssembly 均支持形状、颜色、尺寸和进度语义。

`slider`、`centered_slider` 和 `range_slider` 的两个手柄均支持方向键、
Home/End、PageUp/PageDown。连续滑块按 1% 调整，离散滑块按配置刻度调整。
范围手柄保持固定的 Tab 顺序且不能相互越过；可通过
`.start_accessibility_label(...)` 和 `.end_accessibility_label(...)`
设置各自的本地化名称，组名称仍由 `.accessibility_label(...)` 设置。

`tooltip_provider("Help text")` 在页面上方显示普通提示；用
`tooltip_anchor().child(...)` 包裹触发控件。嵌套提供器时可显式共享
`TooltipController`。悬停等待 500ms，键盘焦点显示提示，触摸长按 500ms
会阻止松手时误触原控件。Escape 和外部按下可关闭提示，其他控件仍可操作。
`TooltipVariant::Rich`、`.headline(...)` 与 `.action_label(...).on_action(...)`
提供富内容提示；`.actions(...)` 支持自定义操作内容，包括两枚文字按钮。
自定义操作可关闭共享控制器。提供器应位于滚动、裁剪容器之外；只有等待中的
计时器请求帧，稳定显示的提示不会持续运行动画时钟。

`Modifier::clip_shape` clips content and pointer hit testing to nested rounded
rectangles or ellipses without resizing it. Use multisampling for smooth mask
edges. Custom UI pipelines must set
`depth_stencil: tessera_ui::ui_stencil_state()`; independent atlas-generation
passes keep their own attachment configuration.

`carousel(item_count)` provides multi-browse, uncontained, mixed-aspect
uncontained, hero, centered hero, and full-screen layouts. Its `.content(...)`
slot receives each item's index and the reactive `full_width()` and
`masked_width()` getters. Read these getters where content adapts to its current
mask, including inside nested content slots. Content retains its full size while
the 28dp rounded mask changes; only visible items are built. Use
`.item_aspect_ratios(...)` for portrait, landscape, and square media in the
mixed-aspect layout, with 8dp spacing and 16dp leading padding by default. Share
`CarouselController` for programmatic navigation. Horizontal dragging and wheel
input, arrow keys, Home/End, and accessibility scroll actions are supported.
Horizontal layouts leave vertical wheel input to the surrounding page.
Uncontained layouts scroll freely by default; other layouts snap. Set spacing
and end padding explicitly for your composition. Full-screen pages fill a
portrait viewport with rectangular masks and 16dp vertical spacing. They use
vertical dragging, wheel input, Up/Down keys, and vertical accessibility scroll
actions; free scrolling is replaced by snapping. The example uses 8dp spacing
for other layouts and 4× multisampling. Keep external controllers and selection
state outside virtualized items when that state must survive scrolling
offscreen.

`text`、`text_input` 和 `text_field` 可通过 `.style(...)` 设置排版。
桌面、Android 和 WebAssembly 使用随库提供的 Roboto Regular、Medium 和 Bold 字体。
`MaterialTypography` 同时提供字号、行高、字重和字距；
`theme.typography.emphasized()` 返回强调排版规格。
自定义 `TextStyle` 字面量应继承主题中的文字样式，或用 `..Default::default()` 初始化新增属性。
显示文本优先在单词之间换行，仅当单词宽于可用行宽时按字符拆分。

导航栏默认采用 64dp 高的灵活布局。导航栏宽度达到 600dp 时，图标从标签上方移至标签前方。
可用 `NavigationBarIconPosition::Top` 或 `Start` 固定布局。

日期选择器的标准日历宽度为 360dp，标准宽度下的日期点击区域为 48dp，
并支持可滚动年份网格和可编辑的 `MM/DD/YYYY` 输入模式。
宽度受限时，日期和年份网格仍分别保持七列和三列。
在 `dialog_provider` 中嵌入 `date_picker_dialog` 时，请将提供器的内边距设为 `Dp(0.0)`，
因为日期选择器已包含标题和操作区域的内边距。
包含星期标签和操作按钮的标准日历对话框尺寸为 360×568dp。

时间选择器支持 256dp 拨盘、小时和分钟选择、24 小时内圈、AM/PM 选择和键盘输入。
`time_picker_dialog` 提供标题、模式切换和操作按钮布局，外层使用对话框提供器默认的 24dp 内边距。

视觉对比应使用相同的配色方案、字体、密度、区域设置和组件状态。
仅对齐排版和布局 token，不能证明不同渲染器或参考主题之间已达到像素级一致。

参考规范：[排版](https://m3.material.io/styles/typography/type-scale-tokens)、
[日期选择器](https://m3.material.io/components/date-pickers/specs)、
[时间选择器](https://m3.material.io/components/time-pickers/specs)和
[导航栏](https://m3.material.io/components/navigation-bar/specs)。

`menu_provider` uses expressive vertical-menu shapes and standard surface colors.
Set `.grouped(true)` and compose `menu_group` children with `MenuPosition` for
separate sections and 2dp gaps. `MenuColorStyle::Vibrant` supplies tertiary-based
emphasis inherited by groups, items, and nested menus. Labels reserve icon and
shortcut space, optional selection publishes menu-radio semantics, and long
menus scroll within their height limit. `menu_item().submenu_content(...)`
anchors cascading menus beside their parent item on a shared overlay layer.
Leaf actions close the complete chain; the opposite arrow or Escape returns one
level, and Tab
leaves the complete chain before traversing the page. Disabled or removed parent
items close their descendant menus without leaving stale focus requests. Menu
shadows extend outside the panel while content is clipped to its rounded shape.
Scrolling grouped menus clip overflowing content along the scroll axis and retain
lateral shadows.

Pass `.anchor(current_anchor)` to follow layout changes while a menu is open;
coordinates are relative to the provider. Without this prop, `open_at` retains a
fixed anchor snapshot. Placement tries the opposite side, including the requested
gap, before clamping to the available viewport. The example tracks its normal
and split-button triggers and closes their menus when the active source leaves
the viewport or is removed.
