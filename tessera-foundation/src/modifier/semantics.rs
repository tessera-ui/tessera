//! Semantics modifiers for accessibility metadata.
//!
//! ## Usage
//!
//! Attach accessibility roles, labels, and testing tags to component subtrees.

use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, SemanticsModifierNode,
    accesskit::{Action, Live, Role, Toggled},
    modifier::ModifierCapabilityExt as _,
};

use super::layout::Padding;

/// Arguments for the `semantics` modifier.
#[derive(Clone, Default)]
pub struct SemanticsArgs {
    /// Optional accessibility role.
    pub role: Option<Role>,
    /// Optional label announced by assistive technologies.
    pub label: Option<String>,
    /// Optional description announced by assistive technologies.
    pub description: Option<String>,
    /// Optional value text.
    pub value: Option<String>,
    /// Optional numeric value.
    pub numeric_value: Option<f64>,
    /// Optional numeric range.
    pub numeric_range: Option<(f64, f64)>,
    /// Whether the node is focusable.
    pub focusable: bool,
    /// Whether the node is focused.
    pub focused: bool,
    /// Optional toggled state.
    pub toggled: Option<Toggled>,
    /// Whether the node is disabled.
    pub disabled: bool,
    /// Whether the node is hidden from accessibility.
    pub hidden: bool,
    /// Custom accessibility actions.
    pub actions: Vec<Action>,
    /// Optional testing tag (mapped to the accessibility key).
    pub test_tag: Option<String>,
    /// Optional padding applied to semantic bounds.
    pub bounds_padding: Option<Padding>,
    /// Whether to merge child semantics into this node.
    pub merge_descendants: bool,
    /// Optional state description.
    pub state_description: Option<String>,
    /// Optional role description for custom controls.
    pub role_description: Option<String>,
    /// Optional tooltip text.
    pub tooltip: Option<String>,
    /// Live region politeness.
    pub live: Option<Live>,
    /// Optional heading level (1-based).
    pub heading_level: Option<u32>,
    /// Optional scroll ranges.
    pub scroll_x: Option<(f64, f64, f64)>,
    /// Optional scroll Y range as `(value, min, max)`.
    pub scroll_y: Option<(f64, f64, f64)>,
    /// Optional numeric step/jump for range controls.
    pub numeric_value_step: Option<f64>,
    /// Optional numeric jump value for page-wise changes.
    pub numeric_value_jump: Option<f64>,
    /// Optional collection info (rows, cols, hierarchical).
    pub collection_info: Option<(usize, usize, bool)>,
    /// Optional collection item info (row_index, row_span, col_index, col_span,
    /// heading).
    pub collection_item_info: Option<(usize, usize, usize, usize, bool)>,
}

impl SemanticsArgs {
    /// Convenience for `clearAndSetSemantics` behavior (no descendant merge).
    pub fn clear() -> Self {
        Self {
            merge_descendants: false,
            ..Default::default()
        }
    }
}

struct SemanticsModifierNodeImpl {
    args: SemanticsArgs,
}

impl SemanticsModifierNode for SemanticsModifierNodeImpl {
    fn apply(
        &self,
        accessibility: &mut AccessibilityNode,
        _action_handler: &mut Option<AccessibilityActionHandler>,
    ) {
        let SemanticsArgs {
            role,
            label,
            description,
            value,
            numeric_value,
            numeric_range,
            focusable,
            focused,
            toggled,
            disabled,
            hidden,
            actions,
            test_tag,
            bounds_padding,
            merge_descendants,
            state_description,
            role_description,
            tooltip,
            live,
            heading_level,
            scroll_x,
            scroll_y,
            numeric_value_step,
            numeric_value_jump,
            collection_info,
            collection_item_info,
        } = &self.args;

        accessibility.role = *role;
        accessibility.label = label.clone();
        accessibility.description = description.clone();
        accessibility.value = value.clone();
        accessibility.numeric_value = *numeric_value;
        accessibility.min_numeric_value = numeric_range.map(|(min, _)| min);
        accessibility.max_numeric_value = numeric_range.map(|(_, max)| max);
        accessibility.focusable = *focusable;
        accessibility.focused = *focused;
        accessibility.toggled = *toggled;
        accessibility.disabled = *disabled;
        accessibility.hidden = *hidden;
        accessibility.actions = actions.clone();
        accessibility.key = test_tag.clone();
        accessibility.merge_descendants = *merge_descendants;
        accessibility.state_description = state_description.clone();
        accessibility.role_description = role_description.clone();
        accessibility.tooltip = tooltip.clone();
        accessibility.live = *live;
        accessibility.heading_level = *heading_level;
        accessibility.scroll_x = *scroll_x;
        accessibility.scroll_y = *scroll_y;
        accessibility.numeric_value_step = *numeric_value_step;
        accessibility.numeric_value_jump = *numeric_value_jump;
        accessibility.collection_info = *collection_info;
        accessibility.collection_item_info = *collection_item_info;
        accessibility.bounds_padding =
            bounds_padding.map(|padding| tessera_ui::accessibility::AccessibilityPadding {
                left: padding.left.into(),
                top: padding.top.into(),
                right: padding.right.into(),
                bottom: padding.bottom.into(),
            });
    }
}

pub(crate) fn apply_semantics_modifier(
    base: tessera_ui::Modifier,
    args: SemanticsArgs,
) -> tessera_ui::Modifier {
    base.push_semantics(SemanticsModifierNodeImpl { args })
}
