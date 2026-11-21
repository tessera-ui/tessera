use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{Dp, Modifier, accesskit::Role, tessera, testing::UiTest};

use crate::{column::column, spacer::spacer};

use super::{LabelStyle, label};

#[tessera]
fn icon() {
    spacer().modifier(
        Modifier::new()
            .size(Dp(16.0), Dp(16.0))
            .semantics(SemanticsArgs {
                test_tag: Some("icon".into()),
                ..Default::default()
            }),
    );
}

#[tessera]
fn fixture(style: LabelStyle) {
    column().children(move || {
        label("Favorites")
            .style(style)
            .icon(|| icon())
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some("label".into()),
                ..Default::default()
            }));
    });
}

fn width(style: LabelStyle) -> (i32, bool) {
    let mut ui = UiTest::new()
        .viewport(300, 80)
        .content(move || fixture(style));
    let frame = ui.frame();
    let width = frame.node("label").bounds().width.0;
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    let named = snapshot.find("Favorites", Role::Label).is_some();
    (width, named)
}

#[test]
fn styles_choose_the_shown_parts_and_keep_the_title_as_the_accessible_name() {
    let (both, both_named) = width(LabelStyle::Automatic);
    let (icon_only, icon_named) = width(LabelStyle::IconOnly);
    let (title_only, title_named) = width(LabelStyle::TitleOnly);
    assert_eq!(icon_only, 16);
    assert_eq!(both, title_only + 16 + 6, "the icon leads the title by 6pt");
    assert!(both_named && icon_named && title_named);
}
