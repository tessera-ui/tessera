//! Accessibility boundaries for temporary surfaces.
//!
//! ## Usage
//!
//! Name dialogs and expose their modal role to assistive technology.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{Modifier, accesskit::Role};

pub(crate) fn dialog_semantics(modifier: Modifier, label: Option<String>, modal: bool) -> Modifier {
    dialog_semantics_with_role(modifier, label, modal, Role::Dialog)
}

pub(crate) fn dialog_semantics_with_role(
    modifier: Modifier,
    label: Option<String>,
    modal: bool,
    role: Role,
) -> Modifier {
    modifier.semantics(SemanticsArgs {
        role: Some(role),
        label,
        modal,
        focusable: true,
        merge_descendants: true,
        ..Default::default()
    })
}
