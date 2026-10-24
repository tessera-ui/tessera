//! Focus retention during manual scrolling of editable controls.
//!
//! ## Usage
//!
//! Scroll past an active editor without forcing it back into the viewport.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    Dp, FocusRequester, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier,
    PressKeyEventType, PxPosition, PxRect, RenderModule, State, modifier::FocusModifierExt as _,
    remember, tessera,
};

use crate::{
    boxed::boxed,
    column::column,
    modifier::{ModifierExt as _, SemanticsArgs},
    spacer::spacer,
    text_field::TextFieldBuilder,
    text_input::TextInputController,
    theme::material_theme,
};

use super::ScrollableController;

thread_local! {
    static SCROLL: Cell<Option<State<ScrollableController>>> = const {Cell::new(None)};
    static EDITOR: Cell<Option<State<TextInputController>>> = const {Cell::new(None)};
    static FIELD_BOUNDS: Cell<Option<PxRect>> = const {Cell::new(None)};
    static NESTED_SCROLL: Cell<Option<(State<ScrollableController>, State<ScrollableController>, FocusRequester)>> = const { Cell::new(None) };
    static OUTER_BOUNDS: Cell<Option<PxRect>> = const { Cell::new(None) };
    static INNER_BOUNDS: Cell<Option<PxRect>> = const { Cell::new(None) };
    static TARGET_BOUNDS: Cell<Option<PxRect>> = const { Cell::new(None) };
}

#[tessera]
fn nested_reveal_fixture() {
    let outer = remember(ScrollableController::new);
    let inner = remember(ScrollableController::new);
    let target = remember(FocusRequester::new).get();
    NESTED_SCROLL.set(Some((outer, inner, target)));
    material_theme().child(move || {
        column()
            .modifier(
                Modifier::new()
                    .size(Dp(300.0), Dp(240.0))
                    .vertical_scroll(outer)
                    .on_globally_positioned(|bounds| OUTER_BOUNDS.set(Some(bounds))),
            )
            .children(move || {
                spacer().modifier(Modifier::new().height(Dp(600.0)));
                column()
                    .modifier(
                        Modifier::new()
                            .size(Dp(240.0), Dp(160.0))
                            .vertical_scroll(inner)
                            .on_globally_positioned(|bounds| INNER_BOUNDS.set(Some(bounds))),
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
                                    ..Default::default()
                                })
                                .on_globally_positioned(|bounds| TARGET_BOUNDS.set(Some(bounds))),
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
        let mut renderer = HeadlessRenderer::new(
            Box::new(nested_reveal_fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 400,
                height: 300,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(3);
        let (outer, inner, target) = NESTED_SCROLL.get().unwrap();
        target.request_focus();
        renderer.step_frame();
        assert!(target.is_focused());
        assert!(inner.with(|scroll| scroll.child_position().y) < 0.0);
        assert!(
            outer.with(|scroll| scroll.child_position().y) < 0.0,
            "revealing the inner viewport must continue into its outer viewport"
        );
        let target_bounds = TARGET_BOUNDS.get().unwrap();
        for viewport in [OUTER_BOUNDS.get().unwrap(), INNER_BOUNDS.get().unwrap()] {
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
        renderer.render_frames(3);
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
fn fixture() {
    let scroll = remember(ScrollableController::new);
    let editor = remember(|| {
        let mut editor = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
        editor.set_text("Input");
        editor
    });
    SCROLL.set(Some(scroll));
    EDITOR.set(Some(editor));
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().fill_max_size().vertical_scroll(scroll))
            .children(move || {
                spacer().modifier(Modifier::new().height(Dp(600.0)));
                TextFieldBuilder::outlined()
                    .controller(editor)
                    .modifier(
                        Modifier::new()
                            .width(Dp(240.0))
                            .height(Dp(56.0))
                            .on_globally_positioned(|bounds| FIELD_BOUNDS.set(Some(bounds))),
                    )
                    .label("Label")
                    .accessibility_label("Scrollable editor");
                spacer().modifier(Modifier::new().height(Dp(600.0)));
            });
    });
}

struct Pipelines;
impl RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        crate::pipelines::register_pipelines(context);
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn wheel_scroll_keeps_a_focused_editor_offscreen_through_later_idle_frames() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 360,
            height: 240,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(3);
    let scroll = SCROLL.get().unwrap();
    let editor = EDITOR.get().unwrap();
    scroll.with_mut(|scroll| scroll.set_scroll_position(PxPosition::from_f32_xy(0.0, -520.0)));
    renderer.render_frames(3);
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
    renderer.render_frames(4);
    assert!(editor.with(|editor| editor.focus_handler().is_focused()));
    let mut previous = scroll.with(|scroll| scroll.child_position().y);
    for _ in 0..2 {
        renderer.pointer_scroll(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(340.0, 120.0),
            0.0,
            200.0,
        );
        renderer.render_frames(30);
        let moved = scroll.with(|scroll| scroll.child_position().y);
        assert!(
            moved > previous + 100.0,
            "manual wheel should pass the focused editor: previous {previous}, current {moved}"
        );
        assert!(
            FIELD_BOUNDS.get().unwrap().y.to_f32() >= renderer.physical_size().height as f32,
            "the focused field must actually be beyond the viewport"
        );
        assert!(
            editor.with(|editor| editor.focus_handler().is_focused()),
            "a placed offscreen editor should retain focus"
        );
        renderer.render_frames(30);
        assert!(
            (scroll.with(|scroll| scroll.child_position().y) - moved).abs() < 1.0,
            "idle frames must not reveal the previously focused editor"
        );
        assert!(editor.with(|editor| editor.focus_handler().is_focused()));
        previous = moved;
    }
}
