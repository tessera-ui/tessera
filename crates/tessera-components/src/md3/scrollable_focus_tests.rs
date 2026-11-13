//! Focus retention during manual scrolling of editable controls.
//!
//! ## Usage
//!
//! Scroll past an active editor without forcing it back into the viewport.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Dp, FocusRequester, MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition, PxRect, State,
    modifier::FocusModifierExt as _, remember, tessera, testing::UiTest,
};

use crate::{
    boxed::boxed,
    column::column,
    md3::{
        modifier::ModifierExt as _, text_field::text_field, text_input::TextInputController,
        theme::material_theme,
    },
    spacer::spacer,
};

use super::ScrollableController;

#[derive(Clone, Copy, Default)]
struct FixtureHandles {
    scroll: Option<State<ScrollableController>>,
    editor: Option<State<TextInputController>>,
    nested: Option<(
        State<ScrollableController>,
        State<ScrollableController>,
        FocusRequester,
    )>,
}

#[tessera]
fn nested_reveal_fixture(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
    let outer = remember(ScrollableController::new);
    let inner = remember(ScrollableController::new);
    let target = remember(FocusRequester::new).get();
    observed.with_mut(|handles| handles.nested = Some((outer, inner, target)));
    material_theme().child(move || {
        column()
            .modifier(
                Modifier::new()
                    .size(Dp(300.0), Dp(240.0))
                    .vertical_scroll(outer)
                    .semantics(SemanticsArgs {
                        test_tag: Some("scroll-outer".into()),
                        merge_descendants: true,
                        ..Default::default()
                    }),
            )
            .children(move || {
                spacer().modifier(Modifier::new().height(Dp(600.0)));
                column()
                    .modifier(
                        Modifier::new()
                            .size(Dp(240.0), Dp(160.0))
                            .vertical_scroll(inner)
                            .semantics(SemanticsArgs {
                                test_tag: Some("scroll-inner".into()),
                                merge_descendants: true,
                                ..Default::default()
                            }),
                    )
                    .children(move || {
                        spacer().modifier(Modifier::new().height(Dp(600.0)));
                        boxed().modifier(
                            Modifier::new()
                                .size(Dp(48.0), Dp(48.0))
                                .focusable()
                                .focus_requester(target)
                                .semantics(SemanticsArgs {
                                    role: Some(tessera_ui::accesskit::Role::Button),
                                    label: Some("Nested focus target".into()),
                                    test_tag: Some("scroll-target".into()),
                                    merge_descendants: true,
                                    ..Default::default()
                                }),
                        );
                        spacer().modifier(Modifier::new().height(Dp(600.0)));
                    });
                spacer().modifier(Modifier::new().height(Dp(600.0)));
            });
    });
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn focus_request_reveals_the_target_through_both_nested_scroll_viewports() {
    let name = concat!(
        module_path!(),
        "::focus_request_reveals_the_target_through_both_nested_scroll_viewports"
    );
    let name = name.split_once("::").unwrap().1;
    crate::test_utils::run_isolated_test(name, || {
        let observed = tessera_ui::testing::TestProbe::default();
        let mut renderer = UiTest::new()
            .viewport(400, 300)
            .rendered(crate::TesseraComponents)
            .content(|| nested_reveal_fixture(observed.clone()));
        renderer.frames(3);
        let (outer, inner, target) = observed.get().nested.unwrap();
        target.request_focus();
        renderer.frames(1);
        assert!(target.is_focused());
        assert!(inner.with(|scroll| scroll.child_position().y) < 0.0);
        assert!(
            outer.with(|scroll| scroll.child_position().y) < 0.0,
            "revealing the inner viewport must continue into its outer viewport"
        );
        let target_bounds = bounds(&renderer, "scroll-target");
        for viewport in [
            bounds(&renderer, "scroll-outer"),
            bounds(&renderer, "scroll-inner"),
        ] {
            assert!(target_bounds.y >= viewport.y);
            assert!(
                target_bounds.y + target_bounds.height <= viewport.y + viewport.height,
                "the focused target must be fully visible in every enclosing viewport"
            );
        }
        let snapshot = renderer.accessibility_snapshot().unwrap();
        assert!(snapshot.nodes.iter().any(|(id, node)| {
            snapshot.focus == *id && node.label() == Some("Nested focus target")
        }));
        let settled = (
            outer.with(|scroll| scroll.child_position()),
            inner.with(|scroll| scroll.child_position()),
        );
        renderer.frames(3);
        assert_eq!(
            settled,
            (
                outer.with(|scroll| scroll.child_position()),
                inner.with(|scroll| scroll.child_position()),
            )
        );
        assert!(target.is_focused());
    });
}

#[tessera]
fn fixture(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
    let scroll = remember(ScrollableController::new);
    let editor = remember(|| {
        let mut editor = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
        editor.set_text("Input");
        editor
    });
    observed.with_mut(|handles| handles.scroll = Some(scroll));
    observed.with_mut(|handles| handles.editor = Some(editor));
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().fill_max_size().vertical_scroll(scroll))
            .children(move || {
                spacer().modifier(Modifier::new().height(Dp(600.0)));
                text_field()
                    .outlined()
                    .controller(editor)
                    .modifier(Modifier::new().width(Dp(240.0)).height(Dp(56.0)).semantics(
                        SemanticsArgs {
                            test_tag: Some("scroll-field".into()),
                            merge_descendants: true,
                            ..Default::default()
                        },
                    ))
                    .label("Label")
                    .accessibility_label("Scrollable editor");
                spacer().modifier(Modifier::new().height(Dp(600.0)));
            });
    });
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn wheel_scroll_keeps_a_focused_editor_offscreen_through_later_idle_frames() {
    let observed = tessera_ui::testing::TestProbe::default();
    let mut renderer = UiTest::new()
        .viewport(360, 240)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(observed.clone()));
    renderer.frames(3);
    let scroll = observed.get().scroll.unwrap();
    let editor = observed.get().editor.unwrap();
    scroll.with_mut(|scroll| scroll.set_scroll_position(PxPosition::from_f32_xy(0.0, -520.0)));
    renderer.frames(3);
    let field = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| {
            node.role() == tessera_ui::accesskit::Role::TextInput
                && node.label() == Some("Scrollable editor")
        })
        .unwrap()
        .1
        .bounds()
        .unwrap();
    let position = PxPosition::from_f32_xy(
        ((field.x0 + field.x1) * 0.5) as f32,
        ((field.y0 + field.y1) * 0.5) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.frames(4);
    assert!(editor.with(|editor| editor.focus_handler().is_focused()));
    let mut previous = scroll.with(|scroll| scroll.child_position().y);
    for _ in 0..2 {
        renderer.pointer_scroll(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(340.0, 120.0),
            0.0,
            200.0,
        );
        renderer.frames(30);
        let moved = scroll.with(|scroll| scroll.child_position().y);
        assert!(
            moved > previous + 100.0,
            "manual wheel should pass the focused editor: previous {previous}, current {moved}"
        );
        assert!(
            bounds(&renderer, "scroll-field").y.to_f32() >= renderer.physical_size().height as f32,
            "the focused field must actually be beyond the viewport"
        );
        assert!(
            editor.with(|editor| editor.focus_handler().is_focused()),
            "a placed offscreen editor should retain focus"
        );
        renderer.frames(30);
        assert!(
            (scroll.with(|scroll| scroll.child_position().y) - moved).abs() < 1.0,
            "idle frames must not reveal the previously focused editor"
        );
        assert!(editor.with(|editor| editor.focus_handler().is_focused()));
        previous = moved;
    }
}

fn bounds(renderer: &UiTest, tag: &str) -> PxRect {
    renderer
        .inspect_components()
        .find_tag(tag)
        .expect("scroll target")
        .bounds()
        .expect("scroll target bounds")
        .rounded()
}
