//! List row height, wrapping, and alignment regressions.
//!
//! ## Usage
//!
//! Keep list content consistent across layout and state changes.

use tessera_ui::{Dp, Modifier, tessera, testing::layout_test};

use crate::{
    boxed::boxed,
    column::column,
    icon::icon,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    spacer::spacer,
    text::text,
    theme::material_theme,
};

use super::{ListItemVerticalAlignment, list_item};

fn tagged(name: &str) -> Modifier {
    Modifier::new().semantics(SemanticsArgs {
        test_tag: Some(name.into()),
        ..Default::default()
    })
}

#[tessera]
fn fixture(mode: u8, alignment: Option<ListItemVerticalAlignment>, minimum: Option<Dp>) {
    material_theme().child(move || {
        column().children(move || {
            list_item()
                .modifier(tagged("item"))
                .headline("Headline")
                .vertical_alignment_optional(alignment)
                .min_height_optional(minimum)
                .overline_text_optional((mode == 2).then(|| "Overline".into()))
                .supporting_text_optional(match mode {
                    1 | 2 => Some("Supporting".into()),
                    3 => Some("First line\nSecond line".into()),
                    4 => Some("Supporting text wraps".into()),
                    _ => None,
                })
                .leading(|| {
                    boxed().modifier(tagged("leading")).children(|| {
                        icon().vector(filled::HOME_SVG);
                    });
                })
                .trailing(|| {
                    spacer().modifier(tagged("trailing").size(Dp(20.0), Dp(20.0)));
                });
        });
    });
}

#[test]
fn list_rows_keep_token_heights_and_reclassify_multiline_supporting_content() {
    crate::test_utils::run_isolated_test(
        "list_item::tests::list_rows_keep_token_heights_and_reclassify_multiline_supporting_content",
        || {
            for (mode, height, inset) in [(0, 56, 18), (1, 72, 26), (2, 88, 10), (3, 88, 10)] {
                let snapshot = layout_test(move || {
                    fixture(mode);
                })
                .viewport_px(320, 200)
                .run();
                snapshot.node("item").position(0, 0).size(320, height);
                snapshot.node("leading").position(16, inset).size(20, 20);
                snapshot.node("trailing").position(284, inset).size(20, 20);
            }
            for (width, height, inset) in [(180, 88, 10), (320, 72, 26)] {
                let snapshot = layout_test(|| {
                    fixture(4);
                })
                .viewport_px(width, 200)
                .run();
                snapshot.node("item").size(width as i32, height);
                snapshot.node("leading").y(inset);
            }
            for (alignment, inset) in [
                (ListItemVerticalAlignment::Top, 10),
                (ListItemVerticalAlignment::Center, 40),
                (ListItemVerticalAlignment::Bottom, 70),
            ] {
                let snapshot = layout_test(move || {
                    fixture(0).alignment(alignment).minimum(Dp(100.0));
                })
                .viewport_px(320, 200)
                .run();
                snapshot.node("item").size(320, 100);
                snapshot.node("leading").y(inset);
                snapshot.node("trailing").y(inset);
            }
        },
    );
}

#[tessera]
fn custom_content_fixture(multiline: bool, narrow: bool) {
    material_theme().child(move || {
        column().children(move || {
            list_item()
                .modifier(tagged("custom-item"))
                .headline("This fallback must not be measured")
                .overline_text("Ignored overline")
                .supporting_text("Ignored supporting text")
                .leading(|| {
                    spacer().modifier(tagged("custom-leading").size(Dp(20.0), Dp(20.0)));
                })
                .trailing(|| {
                    spacer().modifier(tagged("custom-trailing").size(Dp(20.0), Dp(20.0)));
                })
                .overline_content(|| {
                    spacer().modifier(tagged("custom-overline").size(Dp(40.0), Dp(16.0)));
                })
                .content(move || {
                    spacer().modifier(
                        tagged("custom-headline")
                            .size(Dp(if narrow { 200.0 } else { 80.0 }), Dp(24.0)),
                    );
                })
                .supporting_content(move || {
                    spacer().modifier(
                        tagged("custom-supporting")
                            .size(Dp(80.0), Dp(if multiline { 40.0 } else { 20.0 })),
                    );
                });
        });
    });
}

#[tessera]
fn wrapped_slot_fixture() {
    material_theme().child(|| {
        column().children(move || {
            list_item()
                .modifier(tagged("wrapped-item"))
                .content(|| {
                    text().content("Headline");
                })
                .supporting_content(|| {
                    boxed().modifier(tagged("wrapped-supporting")).children(|| {
                        text().content("Supporting text wraps");
                    });
                })
                .leading(|| {
                    spacer().modifier(Modifier::new().size(Dp(20.0), Dp(20.0)));
                })
                .trailing(|| {
                    spacer().modifier(Modifier::new().size(Dp(20.0), Dp(20.0)));
                });
        });
    });
}

#[test]
fn custom_list_slots_replace_text_and_share_main_column_constraints() {
    crate::test_utils::run_isolated_test(
        "list_item::tests::custom_list_slots_replace_text_and_share_main_column_constraints",
        || {
            for (width, multiline, narrow, height, main_width) in
                [(320, false, false, 88, 80), (160, true, true, 100, 64)]
            {
                let snapshot = layout_test(move || {
                    custom_content_fixture(multiline, narrow);
                })
                .viewport_px(width, 200)
                .run();
                snapshot.node("custom-item").size(width as i32, height);
                snapshot
                    .node("custom-leading")
                    .position(16, 10)
                    .size(20, 20);
                snapshot
                    .node("custom-trailing")
                    .position(width as i32 - 36, 10)
                    .size(20, 20);
                snapshot
                    .node("custom-overline")
                    .position(48, 10)
                    .size(40, 16);
                snapshot
                    .node("custom-headline")
                    .position(48, 26)
                    .size(main_width, 24);
                snapshot
                    .node("custom-supporting")
                    .position(48, 50)
                    .size(main_width.min(80), if multiline { 40 } else { 20 });
            }
            for (width, height, supporting_height) in [(180, 88, 40), (320, 72, 20)] {
                let snapshot = layout_test(|| {
                    wrapped_slot_fixture();
                })
                .viewport_px(width, 200)
                .run();
                snapshot.node("wrapped-item").size(width as i32, height);
                snapshot
                    .node("wrapped-supporting")
                    .height(supporting_height);
            }
        },
    );
}
