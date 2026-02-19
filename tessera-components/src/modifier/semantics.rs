//! Semantics modifiers for accessibility metadata.
//!
//! ## Usage
//!
//! Attach accessibility roles, labels, and testing tags to component subtrees.

use tessera_ui::{
    RenderSlot,
    accesskit::{Action, Live, Role, Toggled},
    tessera,
};

use super::layout::Padding;

/// Arguments for the `semantics` modifier.
#[derive(PartialEq, Clone, Default)]
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
    /// Creates a new empty semantics configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Convenience for `clearAndSetSemantics` behavior (no descendant merge).
    pub fn clear() -> Self {
        Self {
            merge_descendants: false,
            ..Default::default()
        }
    }

    /// Set the accessibility role.
    pub fn role(mut self, role: Role) -> Self {
        self.role = Some(role);
        self
    }

    /// Set the accessibility label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the accessibility description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set a text value.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Set a numeric value.
    pub fn numeric_value(mut self, value: f64) -> Self {
        self.numeric_value = Some(value);
        self
    }

    /// Set a numeric range.
    pub fn numeric_range(mut self, min: f64, max: f64) -> Self {
        self.numeric_range = Some((min, max));
        self
    }

    /// Mark the node focusable.
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    /// Mark the node focused.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Set the toggled state.
    pub fn toggled(mut self, toggled: Toggled) -> Self {
        self.toggled = Some(toggled);
        self
    }

    /// Mark the node disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Mark the node hidden from accessibility.
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// Replace the list of accessibility actions.
    pub fn actions(mut self, actions: Vec<Action>) -> Self {
        self.actions = actions;
        self
    }

    /// Add a single accessibility action.
    pub fn add_action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    /// Set a testing tag (stored as the accessibility key).
    pub fn test_tag(mut self, tag: impl Into<String>) -> Self {
        self.test_tag = Some(tag.into());
        self
    }

    /// Apply padding to semantic bounds.
    pub fn bounds_padding(mut self, padding: Padding) -> Self {
        self.bounds_padding = Some(padding);
        self
    }

    /// Control whether descendant semantics are merged.
    pub fn merge_descendants(mut self, merge: bool) -> Self {
        self.merge_descendants = merge;
        self
    }

    /// Set a state description announced with the control.
    pub fn state_description(mut self, description: impl Into<String>) -> Self {
        self.state_description = Some(description.into());
        self
    }

    /// Set a role description for custom controls.
    pub fn role_description(mut self, description: impl Into<String>) -> Self {
        self.role_description = Some(description.into());
        self
    }

    /// Set tooltip text.
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Set live region politeness.
    pub fn live(mut self, live: Live) -> Self {
        self.live = Some(live);
        self
    }

    /// Mark as heading with level (1-based).
    pub fn heading_level(mut self, level: u32) -> Self {
        self.heading_level = Some(level);
        self
    }

    /// Set scroll X value and min/max.
    pub fn scroll_x(mut self, value: f64, min: f64, max: f64) -> Self {
        self.scroll_x = Some((value, min, max));
        self
    }

    /// Set scroll Y value and min/max.
    pub fn scroll_y(mut self, value: f64, min: f64, max: f64) -> Self {
        self.scroll_y = Some((value, min, max));
        self
    }

    /// Set numeric step for range controls.
    pub fn numeric_value_step(mut self, step: f64) -> Self {
        self.numeric_value_step = Some(step);
        self
    }

    /// Set numeric jump for range controls.
    pub fn numeric_value_jump(mut self, jump: f64) -> Self {
        self.numeric_value_jump = Some(jump);
        self
    }

    /// Set collection info.
    pub fn collection_info(mut self, rows: usize, cols: usize, hierarchical: bool) -> Self {
        self.collection_info = Some((rows, cols, hierarchical));
        self
    }

    /// Set collection item info.
    pub fn collection_item_info(
        mut self,
        row_index: usize,
        row_span: usize,
        col_index: usize,
        col_span: usize,
        heading: bool,
    ) -> Self {
        self.collection_item_info = Some((row_index, row_span, col_index, col_span, heading));
        self
    }
}

#[derive(Clone, PartialEq)]
struct ModifierSemanticsArgs {
    semantics: SemanticsArgs,
    child: RenderSlot,
}

pub(crate) fn modifier_semantics(args: SemanticsArgs, child: RenderSlot) {
    let render_args = ModifierSemanticsArgs {
        semantics: args,
        child,
    };
    modifier_semantics_node(&render_args);
}

#[tessera]
fn modifier_semantics_node(args: &ModifierSemanticsArgs) {
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
    } = args.semantics.clone();

    args.child.render();

    input_handler(move |input| {
        let mut builder = input.accessibility();

        if let Some(role) = role {
            builder = builder.role(role);
        }
        if let Some(label) = label.as_ref() {
            builder = builder.label(label.clone());
        }
        if let Some(description) = description.as_ref() {
            builder = builder.description(description.clone());
        }
        if let Some(value) = value.as_ref() {
            builder = builder.value(value.clone());
        }
        if let Some(state_description) = state_description.as_ref() {
            builder = builder.state_description(state_description.clone());
        }
        if let Some(role_description) = role_description.as_ref() {
            builder = builder.role_description(role_description.clone());
        }
        if let Some(tooltip) = tooltip.as_ref() {
            builder = builder.tooltip(tooltip.clone());
        }
        if let Some(level) = heading_level {
            builder = builder.heading_level(level);
        }
        if let Some(numeric_value) = numeric_value {
            builder = builder.numeric_value(numeric_value);
        }
        if let Some((min, max)) = numeric_range {
            builder = builder.numeric_range(min, max);
        }
        if focusable {
            builder = builder.focusable();
        }
        if focused {
            builder = builder.focused();
        }
        if let Some(live) = live {
            builder = builder.live(live);
        }
        if let Some(step) = numeric_value_step {
            builder = builder.numeric_value_step(step);
        }
        if let Some(jump) = numeric_value_jump {
            builder = builder.numeric_value_jump(jump);
        }
        if let Some((value, min, max)) = scroll_x {
            builder = builder.scroll_x(value, min, max);
        }
        if let Some((value, min, max)) = scroll_y {
            builder = builder.scroll_y(value, min, max);
        }
        if let Some((rows, cols, hierarchical)) = collection_info {
            builder = builder.collection_info(rows, cols, hierarchical);
        }
        if let Some((row_index, row_span, col_index, col_span, heading)) = collection_item_info {
            builder =
                builder.collection_item_info(row_index, row_span, col_index, col_span, heading);
        }
        if let Some(toggled) = toggled {
            builder = builder.toggled(toggled);
        }
        if disabled {
            builder = builder.disabled();
        }
        if hidden {
            builder = builder.hidden();
        }
        if !actions.is_empty() {
            builder = builder.actions(actions.clone());
        }
        if let Some(tag) = test_tag.as_ref() {
            builder = builder.test_tag(tag.clone());
        }
        if let Some(padding) = bounds_padding {
            let left = padding.left;
            let top = padding.top;
            let right = padding.right;
            let bottom = padding.bottom;
            builder = builder.bounds_padding_dp(left, top, right, bottom);
        }
        if !merge_descendants {
            builder = builder.clear_and_set();
        }

        builder.commit();
    });
}
