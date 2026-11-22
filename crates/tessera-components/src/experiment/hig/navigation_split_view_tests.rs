use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{Modifier, tessera, testing::UiTest};

use crate::spacer::spacer;

use super::{NavigationSplitViewStyle, NavigationSplitViewVisibility, navigation_split_view};

#[tessera]
fn filler(tag: String) {
    spacer().modifier(Modifier::new().fill_max_size().semantics(SemanticsArgs {
        test_tag: Some(tag),
        ..Default::default()
    }));
}

fn detail_bounds(visibility: NavigationSplitViewVisibility) -> (i32, i32) {
    let mut ui = UiTest::new().viewport(800, 400).content(move || {
        navigation_split_view()
            .column_visibility(visibility)
            .sidebar(|| filler("sidebar".into()))
            .content(|| filler("content".into()))
            .detail(|| filler("detail".into()));
    });
    let bounds = ui.frame().node("detail").bounds();
    (bounds.x.0, bounds.width.0)
}

#[test]
fn column_visibility_shows_or_collapses_the_leading_columns() {
    // Sidebar 220pt, divider, content 260pt, divider, then the detail.
    assert_eq!(
        detail_bounds(NavigationSplitViewVisibility::All),
        (482, 318)
    );
    assert_eq!(
        detail_bounds(NavigationSplitViewVisibility::DoubleColumn),
        (261, 539)
    );
    assert_eq!(
        detail_bounds(NavigationSplitViewVisibility::DetailOnly),
        (0, 800)
    );
}

fn detail_start(style: NavigationSplitViewStyle) -> i32 {
    let mut ui = UiTest::new().viewport(600, 400).content(move || {
        navigation_split_view()
            .style(style)
            .sidebar(|| filler("sidebar".into()))
            .detail(|| filler("detail".into()));
    });
    ui.frame().node("detail").bounds().x.0
}

#[test]
fn prominent_detail_limits_the_sidebar_to_a_third_of_the_width() {
    // The sidebar asks for 220pt; the detail starts after it and the divider.
    assert_eq!(detail_start(NavigationSplitViewStyle::Balanced), 221);
    assert_eq!(detail_start(NavigationSplitViewStyle::ProminentDetail), 201);
}
