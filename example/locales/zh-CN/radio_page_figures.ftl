ui-use-radio-buttons-not-switches-when-only-one-item-can-be-selected-from-a-list-labels-shoul = • 当列表中只能选择一项时，请使用单选按钮，而不是开关。
    • 标签应便于快速浏览。
    • 选中项比未选中项更醒目。

ui-what-s-new = 新变化

ui-radio-buttons-use-new-color-mappings-and-support-dynamic-color-selected-icons-use-primary = 单选按钮采用新的颜色映射并支持动态颜色。选中图标使用 Primary，未选中图标使用 On Surface Variant。相邻文本保持 On Surface 颜色。

ui-color-changes-emphasize-the-selection-without-changing-the-label-these-native-choices-demo = 颜色变化会突出所选项，而不改变标签。这些原生选项展示了当前主题的颜色角色。

ui-radio-buttons-are-the-recommended-way-to-make-a-single-selection-from-a-visible-list-of-op = 单选按钮是从可见选项列表中进行单选的推荐方式。一次只能选中一个单选按钮。每个单选按钮都应始终配有清晰的内联标签。

ui-use-radio-buttons-to-select-one-option-from-a-set-and-expose-all-available-options-checkbo = 使用单选按钮从一组选项中选择一项，并展示所有可用选项。复选框则不同：可以同时选择多个独立选项。

ui-choose-the-right-selection-control = 选择合适的选择控件

ui-use-radio-buttons-when-there-are-five-or-fewer-options-consider-a-drop-down-menu-when-savi = 当选项不超过五个时，请使用单选按钮。如果节省空间很重要，可以考虑使用下拉菜单，但请注意，打开菜单会增加点击次数和认知负担。

ui-radio-anatomy = 单选按钮结构

ui-option-1 = 选项 1

ui-radio-button-icon-anatomy = 单选按钮图标结构

ui-1-selected-icon-2-adjacent-label-text-3-unselected-icon = 1. 选中图标
    2. 相邻标签文本
    3. 未选中图标

ui-1-radio-button-icon = 1. 单选按钮图标

ui-radio-buttons-are-often-arranged-in-stacked-layouts-list-them-vertically-with-an-appropria = 单选按钮通常采用堆叠布局。将它们纵向排列，并预先选中一个合适的选项。为每个选项使用相邻标签；简短且有区分度的标签更便于快速浏览。

ui-adjacent-label-text = 相邻标签文本

ui-always-pair-a-radio-button-with-a-label-describing-what-it-selects-each-choice-needs-its-o = 始终为单选按钮配上描述其所选内容的标签。每个选项都需要自己的标签。标签和单选按钮选择的是同一选项；交互过程中，标签文本保持 On Surface 颜色。

ui-selecting-a-radio-button-deselects-the-previous-option-activating-the-selected-option-does = 选择一个单选按钮会取消选择之前的选项。激活已选中的选项不会将其清除。一组单选按钮可以一开始就有选中项，也可以没有选中项，但应尽可能提供合适的默认值。

ui-apply-a-settings-change-immediately-when-appropriate-in-a-form-or-dialog-keep-a-draft-sele = 在适当的时候立即应用设置更改。在表单或对话框中，在用户确认之前保留草稿选择。在本示例中，“取消”会恢复已保存的值，“继续”会提交所选语言。

ui-people-using-assistive-technology-should-be-able-to-navigate-to-a-radio-button-select-a-ra = 使用辅助技术的用户应能够：
    • 导航到单选按钮
    • 选择单选按钮
    • 获得与输入类型相应的反馈

ui-a-radio-button-can-be-selected-or-unselected-selecting-one-deselects-the-others-a-group-ma = 单选按钮可以处于选中或未选中状态。选中一个会取消选中其他项。一组单选按钮一开始可以有一个选中项，也可以没有。选项一旦选中，再次选择它不会将其清除。

ui-to-let-people-opt-out-provide-a-not-applicable-or-no-option-choice-or-a-separate-clear-sel = 如需允许用户不做选择，请提供“不适用”或“无”之类的选项，或单独的“清除选择”操作。用户必须既能选择单选按钮，也能选择其文本标签。

ui-keep-targets-at-least-48-48dp-by-default-let-people-explicitly-choose-a-denser-layout-or-t = 默认情况下，目标尺寸至少保持 48 × 48dp。让用户明确选择更密集的布局或主题。用于撤销密度设置的控件应保持至少 48 × 48dp，以便轻松撤销该设置。

ui-tab-enters-the-group-at-the-selected-radio-or-the-first-option-if-none-is-selected-shift-t = 按 Tab 键会从已选中的单选按钮进入该组；如果没有选中项，则进入第一个选项。按 Shift+Tab 会从已选中的单选按钮进入；如果没有选中项，则进入最后一个选项。箭头键会在选项之间移动焦点和选择。

ui-enter-at-the-selected-radio-or-the-first-if-none-is-selected-tab-again-leaves-the-group = 从已选中的单选按钮进入，如果没有选中项则从第一个进入；再次按 Tab 离开该组

ui-enter-at-the-selected-radio-or-the-last-if-none-is-selected-move-backward-out-of-the-group = 从已选中的单选按钮进入，如果没有选中项则从最后一个进入；向后移出该组

ui-space = Space

ui-select-the-focused-option = 选择获得焦点的选项

ui-focus-and-select-the-next-option-wrapping-to-the-first = 聚焦并选中下一个选项，到末尾后回到第一个

ui-focus-and-select-the-previous-option-wrapping-to-the-last = 聚焦并选中上一个选项，到开头后回到最后一个

ui-give-the-group-a-category-name-and-each-radio-its-own-adjacent-label-assistive-technology = 为该组提供一个类别名称，并为每个单选按钮提供各自的相邻标签。辅助技术应播报选项的标签、单选按钮角色、选中或未选中状态，以及它与该组的关系。不要仅凭外观来标识单选按钮。

ui-checkout = 结账

ui-ship-to = 配送至

ui-clear-selection = 清除选择

ui-current-language-spanish = 当前语言：西班牙语

ui-choose-your-language = 选择您的语言

ui-you-can-always-change-this-later = 您以后可以随时更改此设置

ui-selection-restored = 已恢复选择

ui-continue = 继续

ui-language = 语言

ui-notifications = 通知

ui-allow-notifications = 允许通知

ui-turn-off-notifications = 关闭通知

ui-allow-access = 允许访问

ui-microphone-access = 麦克风访问权限

ui-location-access = 位置访问权限

ui-filters = 筛选条件

ui-meal-options = 餐点选项

ui-sort-by = 排序依据

ui-relevance = 相关性

ui-lowest-price = 价格最低

ui-highest-rating = 评分最高

ui-most-reviewed = 评价最多

ui-pickles = 酸黄瓜

ui-tomato = 番茄

ui-lettuce = 生菜

ui-cheese = 奶酪

ui-allow-notifications-independently = 分别允许通知

ui-do-one-selected-option = 推荐：单个选中项

ui-do-multiple-independent-options = 推荐：多个独立选项

ui-do-one-setting-on-or-off = 推荐：单个设置的开或关

ui-caution-horizontal-list = 注意：水平列表

ui-do-stacked-list = 推荐：堆叠列表

ui-horizontal-placement-example = 水平放置示例

ui-stacked-placement-example = 堆叠放置示例

ui-radio-button-color-roles-in-light-and-dark-themes-1-primary-2-on-surface-variant = 浅色和深色主题下的单选按钮颜色角色：
    1. Primary
    2. On Surface Variant

ui-adjacent-text-label-color = 相邻文本标签颜色

ui-use-on-surface-for-adjacent-text-labels-this-color-remains-the-same-when-the-label-or-radi = 相邻文本标签使用 On Surface。当标签或单选按钮处于选中、悬停、聚焦或按下状态时，此颜色保持不变。

ui-states-communicate-the-status-of-a-component-these-examples-retain-real-radio-controls-the = 状态用于传达组件当前的状况。这些示例保留了真实的单选控件；其原生的外围圆圈展示了悬停、聚焦和按下状态层。

ui-1-enabled-2-hover-3-focused-4-pressed-5-disabled = 1. 启用
    2. 悬停
    3. 聚焦
    4. 按下
    5. 停用

ui-20dp-icon = 20dp 图标

ui-40dp-state-layer = 40dp 状态层

ui-48-48dp-target = 48 × 48dp 触控目标

ui-icon-diameter = 图标直径

ui-stroke = 描边

ui-selected-dot = 选中圆点

ui-shipping-destination-value = 配送目的地：{ $v0 }

ui-current-language-value = 当前语言：{ $v0 }

ui-value-value-color-role = { $v0 } { $v1 } 颜色角色

ui-value-value-value-radio = { $v0 } { $v1 } { $v2 } 单选按钮

ui-value-radio-dimensions = { $v0 }单选按钮尺寸

ui-do-use-radio-buttons-for-a-single-choice-checkboxes-for-multiple-choices-and-switches-for = 推荐：单选使用单选按钮，多选使用复选框，独立的开/关设置使用开关。不要嵌套单选按钮组，也不要用多个选中的单选按钮来表示多选列表。

ui-right-down-arrow = 右/下箭头

ui-left-up-arrow = 左/上箭头

ui-not-selected = 未选择

ui-hover-preview = 悬停预览

ui-focus-preview = 焦点预览

ui-pressed-preview = 按下预览
