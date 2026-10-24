//! Scrolling specification pages with a reserved classic track.
//!
//! ## Usage
//!
//! Browse long specifications without changing content width after layout.

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    column::column,
    modifier::ModifierExt as _,
    row::row,
    scrollable::{ScrollBarBehavior, ScrollableController, scrollbar_v},
};
use tessera_ui::{Dp, Modifier, RenderSlot, State, tessera};

const TRACK_WIDTH: Dp = Dp(15.0);

#[tessera]
pub(super) fn spec_scroll_viewport(
    controller: State<ScrollableController>,
    viewport_modifier: Option<Modifier>,
    children: Option<RenderSlot>,
) {
    let children = children.unwrap_or_default();
    let viewport_modifier = viewport_modifier.unwrap_or_default();
    row()
        .modifier(Modifier::new().fill_max_size())
        .children(move || {
            column()
                .modifier(
                    viewport_modifier
                        .clone()
                        .fill_max_size()
                        .weight(1.0)
                        .vertical_scroll(controller),
                )
                .children_shared(children);
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().width(TRACK_WIDTH).fill_max_height())
                .children(move || {
                    scrollbar_v()
                        .state(controller)
                        .thickness(TRACK_WIDTH)
                        .scrollbar_behavior(ScrollBarBehavior::AlwaysVisible);
                });
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "scroll_viewport_tests.rs"]
mod tests;
