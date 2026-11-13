//! Material modifier extensions for scrolling and elevation.
//!
//! ## Usage
//!
//! Make content scrollable with Material scrollbars or cast Material elevation shadows.

mod shadow;

use tessera_ui::{Modifier, State};

use crate::md3::{scrollable::ScrollableController, theme::MaterialColorScheme};

pub use shadow::ShadowArgs;

/// Material extensions for scrolling and elevation around component subtrees.
pub trait ModifierExt {
    /// Makes this node a vertical scroll viewport driven by `controller`.
    fn vertical_scroll(self, controller: State<ScrollableController>) -> Modifier;

    /// Makes this node a horizontal scroll viewport driven by `controller`.
    fn horizontal_scroll(self, controller: State<ScrollableController>) -> Modifier;

    /// Makes this node a scroll viewport on the selected axes.
    fn scroll(
        self,
        controller: State<ScrollableController>,
        vertical: bool,
        horizontal: bool,
    ) -> Modifier;

    /// Adds a Material elevation shadow.
    fn shadow(self, scheme: &MaterialColorScheme, args: &ShadowArgs) -> Modifier;
}

impl ModifierExt for Modifier {
    fn vertical_scroll(self, controller: State<ScrollableController>) -> Modifier {
        crate::md3::scrollable::apply_scroll_modifier(self, controller, true, false)
    }

    fn horizontal_scroll(self, controller: State<ScrollableController>) -> Modifier {
        crate::md3::scrollable::apply_scroll_modifier(self, controller, false, true)
    }

    fn scroll(
        self,
        controller: State<ScrollableController>,
        vertical: bool,
        horizontal: bool,
    ) -> Modifier {
        crate::md3::scrollable::apply_scroll_modifier(self, controller, vertical, horizontal)
    }

    fn shadow(self, scheme: &MaterialColorScheme, args: &ShadowArgs) -> Modifier {
        shadow::apply_shadow_modifier(self, scheme, args.clone())
    }
}
