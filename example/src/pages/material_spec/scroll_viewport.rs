use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    column::column,
    md3::{
        modifier::ModifierExt as _,
        scrollable::{ScrollBarBehavior, ScrollableController, scrollbar_v},
    },
    row::row,
};
use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{Dp, Modifier, RenderSlot, State, tessera};

const TRACK_WIDTH: Dp = Dp(15.0);

#[tessera]
pub(super) fn spec_scroll_viewport(
    controller: State<ScrollableController>,
    viewport_modifier: Option<Modifier>,
    scroll_content: Option<bool>,
    children: Option<RenderSlot>,
) {
    let children = children.unwrap_or_default();
    let viewport_modifier = viewport_modifier.unwrap_or_default();
    let viewport_modifier = viewport_modifier.fill_max_size().weight(1.0);
    let viewport_modifier = if scroll_content.unwrap_or(true) {
        viewport_modifier.vertical_scroll(controller)
    } else {
        viewport_modifier
    };
    row()
        .modifier(Modifier::new().fill_max_size())
        .children(move || {
            column()
                .modifier(viewport_modifier.clone())
                .children_shared(children);
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().width(TRACK_WIDTH).fill_max_height())
                .children(move || {
                    scrollbar_v()
                        .controller(controller)
                        .thickness(TRACK_WIDTH)
                        .scrollbar_behavior(ScrollBarBehavior::AlwaysVisible);
                });
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "scroll_viewport_tests.rs"]
mod tests;
