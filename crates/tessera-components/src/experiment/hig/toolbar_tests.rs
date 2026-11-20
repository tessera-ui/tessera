use tessera_ui::{accesskit::Rect, tessera, testing::UiTest};

use super::{ToolbarSymbol, toolbar, toolbar_button};

#[tessera]
fn fixture(navigation: bool) {
    let bar = toolbar().title("Documents").actions(|| {
        toolbar_button().label("Share");
    });
    if navigation {
        bar.navigation(|| {
            toolbar_button()
                .symbol(ToolbarSymbol::Back)
                .accessibility_label("Back");
        });
    }
}

fn bounds(ui: &UiTest, label: &str) -> Rect {
    ui.accessibility_node(label)
        .1
        .bounds()
        .expect("published bounds")
}

#[test]
fn items_and_title_follow_the_window_toolbar_insets() {
    let mut ui = UiTest::new().viewport(600, 200).content(|| fixture(true));
    ui.frames(2);
    let divider = ui.node("divider").bounds();
    assert_eq!((divider.y.0, divider.width.0), (52, 600));
    let back = bounds(&ui, "Back");
    assert_eq!((back.x0, back.y0, back.x1, back.y1), (8.0, 8.0, 44.0, 44.0));
    let share = bounds(&ui, "Share");
    assert_eq!((share.y0, share.y1, share.x1), (8.0, 44.0, 592.0));
    assert_eq!(bounds(&ui, "Documents").x0, 56.0);

    let mut ui = UiTest::new().viewport(600, 200).content(|| fixture(false));
    ui.frames(2);
    assert_eq!(bounds(&ui, "Documents").x0, 20.0);
}
