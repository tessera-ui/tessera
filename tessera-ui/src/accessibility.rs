//! # Accessibility Support
//!
//! This module provides accessibility infrastructure for Tessera UI using
//! AccessKit. It enables screen readers and other assistive technologies to
//! interact with Tessera applications.
//!
//! ## Architecture
//!
//! - **Stable IDs**: Each component can have a stable accessibility ID that
//!   persists across frames
//! - **Semantic Metadata**: Components provide semantic information (role,
//!   label, state, actions)
//! - **Decentralized**: Component libraries decide their own semantics; the
//!   core only provides infrastructure
//!
//! ## Usage
//!
//! Components use the accessibility API through the input handler context:
//!
//! ```
//! use accesskit::{Action, Role};
//! use tessera_ui::tessera;
//!
//! #[tessera]
//! fn my_button(label: String) {
//!     input_handler(move |input| {
//!         // Set accessibility information
//!         input
//!             .accessibility()
//!             .role(Role::Button)
//!             .label(label.clone())
//!             .action(Action::Click);
//!
//!         // Set action handler
//!         input.set_accessibility_action_handler(|action| {
//!             if action == Action::Click {
//!                 // Handle click from assistive technology
//!             }
//!         });
//!     });
//! }
//! ```

mod tree_builder;

use accesskit::{Action, NodeId as AccessKitNodeId, Role, Toggled};

use crate::Px;

pub(crate) use tree_builder::{build_tree_update, dispatch_action};

/// A stable identifier for accessibility nodes.
///
/// This ID is generated based on the component's position in the tree and
/// optional user-provided keys. It remains stable across frames as long as the
/// UI structure doesn't change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccessibilityId(pub u64);

impl AccessibilityId {
    /// Creates a new accessibility ID from a u64.
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Converts to AccessKit's NodeId.
    pub fn to_accesskit_id(self) -> AccessKitNodeId {
        AccessKitNodeId(self.0)
    }

    /// Creates from AccessKit's NodeId.
    pub fn from_accesskit_id(id: AccessKitNodeId) -> Self {
        Self(id.0)
    }

    /// Generates a stable ID from an indextree NodeId.
    ///
    /// indextree uses an arena-based implementation where NodeIds contain:
    /// - A 1-based index into the arena
    /// - A stamp for detecting node reuse
    ///
    /// In Tessera's immediate-mode model, the component tree is cleared and
    /// rebuilt each frame, so there's no node reuse within a frame. This
    /// makes the index stable for the current tree state, which is exactly
    /// what AccessKit requires (IDs only need to be stable within the current
    /// tree).
    ///
    /// # Stability Guarantee
    ///
    /// The ID is stable within a single frame as long as the UI structure
    /// doesn't change. This matches AccessKit's requirement perfectly.
    pub fn from_component_node_id(node_id: indextree::NodeId) -> Self {
        // NodeId implements Into<usize>, giving us the 1-based index
        let index: usize = node_id.into();
        Self(index as u64)
    }
}

/// Padding applied to semantic bounds without affecting layout.
#[derive(Debug, Clone, Copy)]
pub struct AccessibilityPadding {
    /// Left padding in physical pixels.
    pub left: Px,
    /// Top padding in physical pixels.
    pub top: Px,
    /// Right padding in physical pixels.
    pub right: Px,
    /// Bottom padding in physical pixels.
    pub bottom: Px,
}

impl AccessibilityPadding {
    /// Creates zero padding.
    pub const fn zero() -> Self {
        Self {
            left: Px::ZERO,
            top: Px::ZERO,
            right: Px::ZERO,
            bottom: Px::ZERO,
        }
    }
}

/// Semantic information for an accessibility node.
///
/// This structure contains all the metadata that assistive technologies need
/// to understand and interact with a UI component.
#[derive(Debug, Clone)]
pub struct AccessibilityNode {
    /// The role of this node (button, text input, etc.)
    pub role: Option<Role>,
    /// A human-readable label for this node
    pub label: Option<String>,
    /// A detailed description of this node
    pub description: Option<String>,
    /// The current value (for text inputs, sliders, etc.)
    pub value: Option<String>,
    /// Numeric value (for sliders, progress bars, etc.)
    pub numeric_value: Option<f64>,
    /// Minimum numeric value
    pub min_numeric_value: Option<f64>,
    /// Maximum numeric value
    pub max_numeric_value: Option<f64>,
    /// Whether this node can receive focus
    pub focusable: bool,
    /// Whether this node is currently focused
    pub focused: bool,
    /// Toggled/checked state (for checkboxes, switches, radio buttons)
    pub toggled: Option<Toggled>,
    /// Whether this node is disabled
    pub disabled: bool,
    /// Whether this node is hidden from accessibility
    pub hidden: bool,
    /// Supported actions
    pub actions: Vec<Action>,
    /// Custom accessibility key provided by the component
    pub key: Option<String>,
    /// Whether to merge child semantics into this node. When false, child
    /// semantics are ignored, similar to Compose's `clearAndSetSemantics`.
    pub merge_descendants: bool,
    /// Optional padding applied to the semantic bounds without affecting
    /// layout.
    pub bounds_padding: Option<AccessibilityPadding>,
    /// Optional state description announced in addition to the label.
    pub state_description: Option<String>,
    /// Optional custom role description for custom controls.
    pub role_description: Option<String>,
    /// Optional tooltip text exposed as a name override.
    pub tooltip: Option<String>,
    /// Live region politeness.
    pub live: Option<accesskit::Live>,
    /// Optional heading level (1-based). When set and no role is provided,
    /// role will default to `Heading`.
    pub heading_level: Option<u32>,
    /// Optional scroll x value and range.
    pub scroll_x: Option<(f64, f64, f64)>,
    /// Optional scroll y value and range.
    pub scroll_y: Option<(f64, f64, f64)>,
    /// Optional numeric step for range-based controls.
    pub numeric_value_step: Option<f64>,
    /// Optional numeric jump value for range-based controls.
    pub numeric_value_jump: Option<f64>,
    /// Optional collection info: row_count, column_count, hierarchical.
    pub collection_info: Option<(usize, usize, bool)>,
    /// Optional collection item info: row_index, row_span, col_index, col_span,
    /// heading.
    pub collection_item_info: Option<(usize, usize, usize, usize, bool)>,
    /// Optional editable text flag.
    pub is_editable_text: bool,
}

impl AccessibilityNode {
    /// Creates a new empty accessibility node.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the role of this node.
    pub fn with_role(mut self, role: Role) -> Self {
        self.role = Some(role);
        self
    }

    /// Sets the label of this node.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the description of this node.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the value of this node.
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Sets the numeric value of this node.
    pub fn with_numeric_value(mut self, value: f64) -> Self {
        self.numeric_value = Some(value);
        self
    }

    /// Sets the numeric range of this node.
    pub fn with_numeric_range(mut self, min: f64, max: f64) -> Self {
        self.min_numeric_value = Some(min);
        self.max_numeric_value = Some(max);
        self
    }

    /// Marks this node as focusable.
    pub fn focusable(mut self) -> Self {
        self.focusable = true;
        self
    }

    /// Marks this node as focused.
    pub fn focused(mut self) -> Self {
        self.focused = true;
        self
    }

    /// Sets the toggled/checked state of this node.
    pub fn with_toggled(mut self, toggled: Toggled) -> Self {
        self.toggled = Some(toggled);
        self
    }

    /// Marks this node as disabled.
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    /// Marks this node as hidden from accessibility.
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    /// Adds an action that this node supports.
    pub fn with_action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    /// Adds multiple actions that this node supports.
    pub fn with_actions(mut self, actions: impl IntoIterator<Item = Action>) -> Self {
        self.actions.extend(actions);
        self
    }

    /// Sets a custom accessibility key for stable ID generation.
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }
}

impl Default for AccessibilityNode {
    fn default() -> Self {
        Self {
            role: None,
            label: None,
            description: None,
            value: None,
            numeric_value: None,
            min_numeric_value: None,
            max_numeric_value: None,
            focusable: false,
            focused: false,
            toggled: None,
            disabled: false,
            hidden: false,
            actions: Vec::new(),
            key: None,
            merge_descendants: true,
            bounds_padding: None,
            state_description: None,
            role_description: None,
            tooltip: None,
            live: None,
            heading_level: None,
            scroll_x: None,
            scroll_y: None,
            numeric_value_step: None,
            numeric_value_jump: None,
            collection_info: None,
            collection_item_info: None,
            is_editable_text: false,
        }
    }
}

/// Handler for accessibility actions.
///
/// When an assistive technology requests an action (like clicking a button),
/// this handler is invoked.
pub type AccessibilityActionHandler = Box<dyn Fn(Action) + Send + Sync>;
