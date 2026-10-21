//! Accessibility boundaries for temporary surfaces.
//!
//! ## Usage
//!
//! Name dialogs and hide background controls while a modal is visible.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{Modifier, RenderSlot, accesskit::Role, layout::layout, tessera};

pub(crate) fn dialog_semantics(modifier: Modifier, label: Option<String>, modal: bool) -> Modifier {
    modifier.semantics(SemanticsArgs {
        role: Some(Role::Dialog),
        label,
        modal,
        focusable: true,
        merge_descendants: true,
        ..Default::default()
    })
}

#[tessera]
pub(crate) fn overlay_background(content: RenderSlot, blocked: bool) {
    let modifier = if blocked {
        Modifier::new().clear_and_set_semantics(SemanticsArgs {
            hidden: true,
            ..Default::default()
        })
    } else {
        Modifier::new()
    };
    layout().modifier(modifier).child_shared(content);
}
