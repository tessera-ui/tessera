use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    column::column,
    md3::{modifier::ModifierExt as _, scrollable::ScrollableController, theme::material_theme},
    spacer::spacer,
};
use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Dp, Modifier, PxPosition, State,
    accesskit::{Action, Node, NodeId, Role, Toggled},
    remember, tessera,
    testing::UiTest,
};

use super::tokens;

#[tessera]
fn fixture() {
    material_theme().child(|| {
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(Modifier::new().fill_max_size())
            .children(|| {
                column()
                    .modifier(Modifier::new().width(Dp(1024.0)))
                    .children(tokens);
            });
    });
}

#[tessera]
fn scrolling_fixture(output: tessera_ui::testing::TestProbe<Option<State<ScrollableController>>>) {
    let scroll = remember(ScrollableController::new);
    output.set(Some(scroll));
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().fill_max_size().vertical_scroll(scroll))
            .children(|| {
                tokens();
                spacer().modifier(Modifier::new().height(Dp(600.0)));
            });
    });
}

fn nodes(renderer: &UiTest) -> Vec<(NodeId, Node)> {
    renderer
        .accessibility_snapshot()
        .expect("token selector semantics")
        .nodes
}

fn button(renderer: &UiTest, label: &str) -> NodeId {
    nodes(renderer)
        .into_iter()
        .find(|(_, node)| node.role() == Role::Button && crate::has_label(&node, label))
        .unwrap_or_else(|| {
            let published: Vec<_> = nodes(renderer)
                .into_iter()
                .map(|(_, node)| (node.role(), node.label().map(str::to_owned)))
                .collect();
            panic!("missing token button {label}; published semantics: {published:?}")
        })
        .0
}

fn activate(renderer: &mut UiTest, target_node: NodeId) {
    assert!(renderer.accessibility_action(target_node, Action::Click));
    renderer.frames(6);
}

fn click_button(renderer: &mut UiTest, label: &str) {
    let target = button(renderer, label);
    activate(renderer, target);
}

fn contains_text(renderer: &UiTest, expected: &str) -> bool {
    nodes(renderer).iter().any(|(_, node)| {
        node.label()
            .map(crate::visible_text)
            .is_some_and(|label| label.contains(expected))
            || node.value().is_some_and(|value| value.contains(expected))
    })
}

fn menu_options(renderer: &UiTest) -> Vec<(NodeId, Node)> {
    nodes(renderer)
        .into_iter()
        .filter(|(_, node)| node.role() == Role::MenuItemRadio)
        .collect()
}

#[test]
#[ignore = "Requires a native GPU adapter for actual token menus, tables and accordion controls"]
fn token_selector_switches_all_sets_and_accordion_controls_hide_and_restore_rows() {
    let mut renderer = UiTest::new()
        .viewport(1100, 1800)
        .rendered(crate::TestPipelines)
        .content(fixture);
    renderer.frames(6);
    let options = [
        ("Slider", "Enabled", "md.comp.slider.active.track.height"),
        (
            "Slider - Size - Xsmall",
            "Active track",
            "md.comp.slider.xsmall.active.track.height",
        ),
        (
            "Slider - Size - Small",
            "Active track",
            "md.comp.slider.small.active.track.height",
        ),
        (
            "Slider - Size - Medium",
            "Active track",
            "md.comp.slider.medium.active.track.height",
        ),
        (
            "Slider - Size - Large",
            "Active track",
            "md.comp.slider.large.active.track.height",
        ),
        (
            "Slider - Size - Xlarge",
            "Active track",
            "md.comp.slider.xlarge.active.track.height",
        ),
    ];
    for (selected, group, token) in options {
        click_button(&mut renderer, "Slider token set selector");
        let menu = menu_options(&renderer);
        assert_eq!(menu.len(), 6, "all six token sets are available");
        for (label, ..) in options {
            assert!(menu.iter().any(|(_, node)| crate::has_label(&node, label)));
        }
        let target = menu
            .into_iter()
            .find(|(_, node)| crate::has_label(&node, selected))
            .unwrap()
            .0;
        activate(&mut renderer, target);
        assert!(
            menu_options(&renderer).is_empty(),
            "selection closes the menu"
        );
        assert!(
            !contains_text(&renderer, token),
            "new token groups begin collapsed"
        );
        click_button(&mut renderer, group);
        assert!(
            contains_text(&renderer, token),
            "{selected} displays its own token rows"
        );
        click_button(&mut renderer, "Slider token set selector");
        let reopened = menu_options(&renderer);
        assert_eq!(reopened.len(), 6);
        for (_, node) in &reopened {
            assert_eq!(
                node.toggled(),
                Some(if crate::has_label(&node, selected) {
                    Toggled::True
                } else {
                    Toggled::False
                }),
                "the reopened selector retains {selected}"
            );
        }
        let target = reopened
            .into_iter()
            .find(|(_, node)| crate::has_label(&node, selected))
            .unwrap()
            .0;
        activate(&mut renderer, target);
        assert!(menu_options(&renderer).is_empty());
        assert!(
            contains_text(&renderer, token),
            "reselecting the current set preserves its expanded group"
        );
        click_button(&mut renderer, group);
        assert!(
            !contains_text(&renderer, token),
            "collapsing removes the token rows"
        );
        click_button(&mut renderer, "Expand all");
        assert!(
            contains_text(&renderer, token),
            "Expand all restores token rows"
        );
        click_button(&mut renderer, "Collapse all");
        assert!(
            !contains_text(&renderer, token),
            "Collapse all removes token rows"
        );
    }
}

#[test]
#[ignore = "Requires a native GPU adapter for retained token drawing and popup positioning"]
fn translated_expanded_tokens_reuse_drawing_and_keep_the_popup_on_its_selector() {
    let output = tessera_ui::testing::TestProbe::default();
    let mut renderer = UiTest::new()
        .viewport(1100, 600)
        .rendered(crate::TestPipelines)
        .content(|| scrolling_fixture(output.clone()));
    renderer.frames(8);
    click_button(&mut renderer, "Expand all");
    let token = nodes(&renderer)
        .into_iter()
        .find(|(_, node)| {
            node.label()
                .map(crate::visible_text)
                .is_some_and(|label| label.contains("md.comp.slider.active.track.height"))
        })
        .expect("expanded token row");
    click_button(&mut renderer, "Slider token set selector");
    let popup_before = menu_options(&renderer)[0].1.bounds().unwrap();
    let selected = menu_options(&renderer)
        .into_iter()
        .find(|(_, node)| crate::has_label(&node, "Slider"))
        .unwrap()
        .0;
    activate(&mut renderer, selected);
    renderer.frames(24);
    let selector = button(&renderer, "Slider token set selector");
    let selector_before = nodes(&renderer)
        .into_iter()
        .find(|(id, _)| *id == selector)
        .unwrap()
        .1
        .bounds()
        .unwrap();
    let row_before = token.1.bounds().unwrap();

    output
        .get()
        .unwrap()
        .with_mut(|scroll| scroll.set_scroll_position(PxPosition::from_f32_xy(0.0, -32.0)));
    let mut paint = 0;
    let mut measurements = 0;
    let mut geometry = 0;
    for _ in 0..6 {
        renderer.frames(1);
        let work = renderer.diagnostics();
        paint += work.paint_nodes_recorded;
        measurements += renderer
            .work()
            .phase(tessera_ui::testing::WorkPhase::Measure)
            .count();
        geometry += work.geometry_nodes_updated;
    }
    assert!(geometry > 0, "the actual token subtree must move");
    assert_eq!(measurements, 0, "translation must reuse token measurements");
    assert!(
        paint <= 24,
        "moving an unchanged token table must retain its recorded drawing: {paint} recordings"
    );
    let current = nodes(&renderer);
    let selector_after = current
        .iter()
        .find(|(_, node)| crate::has_label(&node, "Slider token set selector"))
        .unwrap()
        .1
        .bounds()
        .unwrap();
    let row_after = current
        .iter()
        .find(|(id, _)| *id == token.0)
        .expect("the token row must retain its semantic identity")
        .1
        .bounds()
        .unwrap();
    assert_eq!(selector_after.y0 - selector_before.y0, -32.0);
    assert_eq!(row_after.y0 - row_before.y0, -32.0);
    assert_eq!(row_after.width(), row_before.width());
    assert_eq!(row_after.height(), row_before.height());
    renderer.frames(1);
    assert_eq!(
        renderer
            .work()
            .phase(tessera_ui::testing::WorkPhase::Paint)
            .count(),
        0
    );
    assert_eq!(
        renderer
            .work()
            .phase(tessera_ui::testing::WorkPhase::Measure)
            .count(),
        0
    );

    click_button(&mut renderer, "Slider token set selector");
    let popup_after = menu_options(&renderer)[0].1.bounds().unwrap();
    assert_eq!(
        popup_after.y0 - popup_before.y0,
        selector_after.y0 - selector_before.y0,
        "the reopened popup must follow its translated selector"
    );
}
