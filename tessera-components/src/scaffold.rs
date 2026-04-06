//! Scaffold layout for persistent bars and app content.
//!
//! ## Usage
//!
//! Layer top/bottom bars, floating buttons, and snackbars above app content.
use tessera_ui::{Dp, Modifier, RenderSlot, layout::layout, tessera};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    modifier::{ModifierExt as _, Padding},
};

fn scaffold_content_padding(base: Padding, top_bar_height: Dp, bottom_bar_height: Dp) -> Padding {
    Padding::new(
        base.left,
        Dp(base.top.0 + top_bar_height.0),
        base.right,
        Dp(base.bottom.0 + bottom_bar_height.0),
    )
}

fn overlay_offset(alignment: Alignment, offset: [Dp; 2], bottom_bar_height: Dp) -> [Dp; 2] {
    let base_y = match alignment {
        Alignment::BottomStart | Alignment::BottomCenter | Alignment::BottomEnd => {
            Dp(-bottom_bar_height.0)
        }
        _ => Dp(0.0),
    };
    [offset[0], Dp(offset[1].0 + base_y.0)]
}

/// # scaffold
///
/// Layout top/bottom bars with floating content for app screens with persistent
/// actions.
///
/// ## Usage
///
/// Use for screens with app bars, floating actions, and transient messages.
///
/// ## Parameters
///
/// - `modifier` — optional modifier chain applied to the scaffold container.
/// - `content_padding` — padding applied around the content area.
/// - `content` — optional main content slot.
/// - `top_bar_height` — reserved height for the top bar.
/// - `bottom_bar_height` — reserved height for the bottom bar.
/// - `top_bar` — optional top bar slot.
/// - `bottom_bar` — optional bottom bar slot.
/// - `floating_action_button` — optional floating action button slot.
/// - `floating_action_button_alignment` — optional floating action button
///   alignment.
/// - `floating_action_button_offset` — additional x/y offset applied to the
///   floating action button.
/// - `snackbar_host` — optional snackbar host slot.
/// - `snackbar_alignment` — optional snackbar host alignment.
/// - `snackbar_offset` — additional x/y offset applied to the snackbar host.
///
/// ## Examples
///
/// ```
/// use tessera_components::app_bar::{AppBarDefaults, top_app_bar};
/// use tessera_components::scaffold::scaffold;
/// use tessera_components::text::text;
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     material_theme()
///         .theme(|| MaterialTheme::default())
///         .child(|| {
///             let counter = remember(|| 1u32);
///             scaffold()
///                 .top_bar_height(AppBarDefaults::TOP_APP_BAR_HEIGHT)
///                 .top_bar(|| {
///                     top_app_bar().title("Inbox");
///                 })
///                 .content(|| {
///                     text().content("Hello scaffold");
///                 });
///             assert_eq!(counter.get(), 1);
///         });
/// }
/// ```
#[tessera]
pub fn scaffold(
    modifier: Option<Modifier>,
    content_padding: Padding,
    content: Option<RenderSlot>,
    top_bar_height: Dp,
    bottom_bar_height: Dp,
    top_bar: Option<RenderSlot>,
    bottom_bar: Option<RenderSlot>,
    floating_action_button: Option<RenderSlot>,
    floating_action_button_alignment: Option<Alignment>,
    floating_action_button_offset: [Dp; 2],
    snackbar_host: Option<RenderSlot>,
    snackbar_alignment: Option<Alignment>,
    snackbar_offset: [Dp; 2],
) {
    let modifier = modifier.unwrap_or_else(|| Modifier::new().fill_max_size());
    let content_padding =
        scaffold_content_padding(content_padding, top_bar_height, bottom_bar_height);
    let fab_alignment = floating_action_button_alignment.unwrap_or(Alignment::BottomEnd);
    let fab_offset = overlay_offset(
        fab_alignment,
        floating_action_button_offset,
        bottom_bar_height,
    );
    let snackbar_alignment = snackbar_alignment.unwrap_or(Alignment::BottomCenter);
    let snackbar_offset = overlay_offset(snackbar_alignment, snackbar_offset, bottom_bar_height);

    layout().modifier(modifier).child(move || {
        boxed().children(move || {
            if let Some(content) = content {
                layout()
                    .modifier(Modifier::new().padding(content_padding).fill_max_size())
                    .child(move || {
                        content.render();
                    });
            }
            if let Some(bottom_bar) = bottom_bar {
                layout()
                    .modifier(Modifier::new().align(Alignment::BottomCenter))
                    .child(move || {
                        bottom_bar.render();
                    });
            }
            if let Some(top_bar) = top_bar {
                layout()
                    .modifier(Modifier::new().align(Alignment::TopCenter))
                    .child(move || {
                        top_bar.render();
                    });
            }
            if let Some(snackbar_host) = snackbar_host {
                layout()
                    .modifier(
                        Modifier::new()
                            .align(snackbar_alignment)
                            .offset(snackbar_offset[0], snackbar_offset[1]),
                    )
                    .child(move || {
                        snackbar_host.render();
                    });
            }
            if let Some(floating_action_button) = floating_action_button {
                layout()
                    .modifier(
                        Modifier::new()
                            .align(fab_alignment)
                            .offset(fab_offset[0], fab_offset[1]),
                    )
                    .child(move || {
                        floating_action_button.render();
                    });
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use tessera_ui::{
        ComputedData, LayoutInput, LayoutOutput, LayoutPolicy, MeasurementError, Modifier,
        NoopRenderPolicy, Px, layout::layout, tessera,
    };

    use crate::modifier::{ModifierExt as _, SemanticsArgs};

    use super::scaffold;

    #[derive(Clone, PartialEq)]
    struct FixedTestLayout {
        width: i32,
        height: i32,
    }

    impl LayoutPolicy for FixedTestLayout {
        fn measure(
            &self,
            _input: &LayoutInput<'_>,
            _output: &mut LayoutOutput<'_>,
        ) -> Result<ComputedData, MeasurementError> {
            Ok(ComputedData {
                width: Px::new(self.width),
                height: Px::new(self.height),
            })
        }
    }

    #[tessera]
    fn fixed_test_box(tag: String, width: i32, height: i32) {
        layout()
            .layout_policy(FixedTestLayout { width, height })
            .render_policy(NoopRenderPolicy)
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some(tag),
                ..Default::default()
            }));
    }

    #[tessera]
    fn scaffold_layout_case() {
        scaffold()
            .modifier(Modifier::new().constrain(
                Some(tessera_ui::AxisConstraint::exact(Px::new(100))),
                Some(tessera_ui::AxisConstraint::exact(Px::new(80))),
            ))
            .top_bar_height(Px::new(10).into())
            .bottom_bar_height(Px::new(12).into())
            .top_bar(|| {
                fixed_test_box()
                    .tag("scaffold_top".to_string())
                    .width(100)
                    .height(10);
            })
            .bottom_bar(|| {
                fixed_test_box()
                    .tag("scaffold_bottom".to_string())
                    .width(100)
                    .height(12);
            })
            .content(|| {
                fixed_test_box()
                    .tag("scaffold_content".to_string())
                    .width(20)
                    .height(8);
            });
    }

    #[test]
    fn scaffold_positions_top_bottom_and_content_slots() {
        tessera_ui::assert_layout! {
            viewport: (120, 100),
            content: {
                scaffold_layout_case();
            },
            expect: {
                node("scaffold_top").position(0, 0).size(100, 10);
                node("scaffold_content").position(0, 10).size(20, 8);
                node("scaffold_bottom").position(0, 68).size(100, 12);
            }
        }
    }
}
