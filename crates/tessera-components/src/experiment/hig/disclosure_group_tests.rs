use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Dp, Modifier,
    accesskit::{Action, Role},
    tessera,
    testing::UiTest,
};

use crate::{column::column, spacer::spacer};

use super::{OutlineItem, disclosure_group, outline_group};

#[tessera]
fn fixture() {
    column().children(|| {
        disclosure_group("Advanced").content(|| {
            spacer().modifier(
                Modifier::new()
                    .size(Dp(100.0), Dp(40.0))
                    .semantics(SemanticsArgs {
                        role: Some(Role::Button),
                        label: Some("Reset".into()),
                        ..Default::default()
                    }),
            );
        });
    });
}

fn content_height(ui: &UiTest) -> Option<f64> {
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    let (_, reset) = snapshot.find("Reset", Role::Button)?;
    let (id, _) = snapshot
        .nodes()
        .iter()
        .find(|(_, node)| node.role() == Role::Group && node.children().len() == 1)?;
    let group = snapshot.get(*id)?.bounds()?;
    reset.bounds()?;
    Some(group.y1 - group.y0)
}

#[test]
fn expanding_reveals_the_content_over_time_and_the_label_toggles_it() {
    let mut ui = UiTest::new().viewport(300, 200).content(fixture);
    ui.frame();
    assert!(
        content_height(&ui).is_none(),
        "collapsed content is not shown"
    );
    let (triangle, node) = ui.accessibility_node("Advanced");
    assert_eq!(node.role(), Role::DisclosureTriangle);
    assert!(ui.accessibility_action(triangle, Action::Expand));
    ui.frames(6);
    let partial = content_height(&ui).expect("revealing content");
    assert!(
        partial > 0.0 && partial < 44.0,
        "the content grows into view: {partial}"
    );
    ui.frames(20);
    assert_eq!(content_height(&ui), Some(44.0));
    let label = ui.frame().node("text_node").bounds();
    ui.click_at(tessera_ui::PxPosition::from_f32_xy(
        (label.x.0 + 4) as f32,
        (label.y.0 + 4) as f32,
    ));
    ui.frames(20);
    assert!(
        content_height(&ui).is_none(),
        "clicking the label collapses the group"
    );
}

#[test]
fn outline_children_appear_under_their_expanded_parent() {
    let mut ui = UiTest::new().viewport(300, 300).content(|| {
        outline_group(vec![
            OutlineItem::new("Documents").with_children(vec![OutlineItem::new("Report.pdf")]),
            OutlineItem::new("Downloads"),
        ]);
    });
    ui.frame();
    let shown = |ui: &UiTest, label: &str| {
        ui.inspect_accessibility()
            .expect("published semantics")
            .nodes()
            .iter()
            .any(|(_, node)| node.label() == Some(label) || node.value() == Some(label))
    };
    assert!(!shown(&ui, "Report.pdf"));
    let (documents, _) = ui.accessibility_node("Documents");
    assert!(ui.accessibility_action(documents, Action::Expand));
    ui.frames(20);
    assert!(shown(&ui, "Report.pdf"));
}
