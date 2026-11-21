use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{Dp, Modifier, accesskit::Role, tessera, testing::UiTest};

use crate::spacer::spacer;

use super::{super::content_unavailable_view::content_unavailable_view, form, labeled_content};

#[tessera]
fn control(tag: String) {
    spacer().modifier(
        Modifier::new()
            .size(Dp(60.0), Dp(20.0))
            .semantics(SemanticsArgs {
                test_tag: Some(tag),
                ..Default::default()
            }),
    );
}

#[test]
fn column_forms_align_their_controls_after_the_widest_label() {
    let mut ui = UiTest::new().viewport(400, 200).content(|| {
        form().content(|| {
            labeled_content("Name").content(|| control("short".into()));
            labeled_content("Default browser").content(|| control("long".into()));
        });
    });
    ui.frames(3);
    let frame = ui.frame();
    let short = frame.node("short").bounds().x.0;
    let long = frame.node("long").bounds().x.0;
    assert_eq!(short, long);
    assert!(short > 40, "controls follow the label column");
}

#[test]
fn unavailable_content_publishes_its_title_and_description() {
    let mut ui = UiTest::new().viewport(400, 200).content(|| {
        content_unavailable_view("No Results").description("Try a new search.");
    });
    ui.frame();
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    let (_, node) = snapshot.find("No Results", Role::Group).expect("view");
    assert_eq!(node.description(), Some("Try a new search."));
}
