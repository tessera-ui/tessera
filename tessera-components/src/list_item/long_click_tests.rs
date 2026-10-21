//! Stationary long actions and click arbitration through real components.
//!
//! ## Usage
//!
//! Keep long holds distinct from selection, child clicks, and cancelled input.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    Dp, FocusManager, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier,
    PressKeyEventType, PxPosition, State,
    accesskit::{Action, ActionRequest, Node, NodeId, TreeId},
    remember, tessera,
};

use crate::{
    button::button,
    column::column,
    modifier::{InteractionState, ModifierExt as _},
    ripple_state::RippleState,
    surface::surface,
    text::text,
    theme::material_theme,
};

use super::{ListItemSelectionMode, list_item};

#[derive(Clone, Copy, PartialEq)]
struct Settings {
    enabled: bool,
    handlers: bool,
    selected: bool,
    mode: ListItemSelectionMode,
    show_surface: bool,
}

thread_local! {
    static SURFACE_INTERACTION: Cell<Option<State<InteractionState>>> = const { Cell::new(None) };
    static SURFACE_RIPPLE: Cell<Option<State<RippleState>>> = const { Cell::new(None) };
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
    static SELECTIONS: Cell<usize> = const { Cell::new(0) };
    static LONGS: Cell<usize> = const { Cell::new(0) };
    static ONLY_LONGS: Cell<usize> = const { Cell::new(0) };
    static CHILD_CLICKS: Cell<usize> = const { Cell::new(0) };
    static PARENT_CLICKS: Cell<usize> = const { Cell::new(0) };
    static PARENT_LONGS: Cell<usize> = const { Cell::new(0) };
    static SURFACE_CLICKS: Cell<usize> = const { Cell::new(0) };
    static SURFACE_LONGS: Cell<usize> = const { Cell::new(0) };
}

#[tessera]
fn fixture() {
    let settings = remember(|| Settings {
        enabled: true,
        handlers: true,
        selected: false,
        mode: ListItemSelectionMode::Single,
        show_surface: true,
    });
    SETTINGS.set(Some(settings));
    let interaction = remember(InteractionState::new);
    let ripple = remember(RippleState::new);
    SURFACE_INTERACTION.set(Some(interaction));
    SURFACE_RIPPLE.set(Some(ripple));
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().width(Dp(320.0)))
            .children(move || {
                list_item()
                    .headline("Selection row")
                    .accessibility_label("Primary target")
                    .selection_mode(settings.get().mode)
                    .selected(settings.get().selected)
                    .enabled(settings.get().enabled)
                    .on_selection_change(move |value| {
                        SELECTIONS.set(SELECTIONS.get() + 1);
                        settings.with_mut(|s| s.selected = value);
                    })
                    .on_long_click(move || {
                        LONGS.set(LONGS.get() + 1);
                        settings.with_mut(|s| {
                            s.mode = ListItemSelectionMode::Multiple;
                            s.selected = true;
                        });
                    });
                let long_only = if settings.get().handlers {
                    Some(tessera_ui::Callback::new(|| {
                        ONLY_LONGS.set(ONLY_LONGS.get() + 1)
                    }))
                } else {
                    None
                };
                list_item()
                    .headline("Hold this row")
                    .accessibility_label("Long-only target")
                    .enabled(settings.get().enabled)
                    .on_long_click_optional(long_only);
                list_item()
                    .accessibility_label("Parent target")
                    .enabled(settings.get().enabled)
                    .on_click(|| PARENT_CLICKS.set(PARENT_CLICKS.get() + 1))
                    .on_long_click(|| PARENT_LONGS.set(PARENT_LONGS.get() + 1))
                    .content(|| {
                        button()
                            .modifier(Modifier::new().width(Dp(100.0)))
                            .accessibility_label("Child target")
                            .on_click(|| CHILD_CLICKS.set(CHILD_CLICKS.get() + 1))
                            .child(|| {
                                text().content("Child");
                            });
                    });
                let surface_long = if settings.get().handlers {
                    Some(tessera_ui::Callback::new(|| {
                        SURFACE_LONGS.set(SURFACE_LONGS.get() + 1)
                    }))
                } else {
                    None
                };
                if settings.get().show_surface {
                    surface()
                        .interaction_state(interaction)
                        .ripple_state(ripple)
                        .modifier(Modifier::new().size(Dp(320.0), Dp(56.0)))
                        .accessibility_label("Surface target")
                        .enabled(settings.get().enabled)
                        .on_click(|| SURFACE_CLICKS.set(SURFACE_CLICKS.get() + 1))
                        .on_long_click_optional(surface_long)
                        .child(|| {
                            text().content("Surface");
                        });
                }
            });
    });
}

fn node(renderer: &HeadlessRenderer, label: &str) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label))
        .unwrap()
}

fn center(renderer: &HeadlessRenderer, label: &str) -> PxPosition {
    let bounds = node(renderer, label).1.bounds().unwrap();
    PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    )
}

fn action(renderer: &mut HeadlessRenderer, label: &str, action: Action) -> bool {
    let id = node(renderer, label).0;
    renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: id,
        data: None,
    })
}

fn press(renderer: &mut HeadlessRenderer, point: PxPosition) {
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.step_frame();
}

fn release(renderer: &mut HeadlessRenderer, point: PxPosition) {
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(3);
}

fn check(samples: u32) {
    for scale in [1.0, 1.5] {
        for counter in [
            &SELECTIONS,
            &LONGS,
            &ONLY_LONGS,
            &CHILD_CLICKS,
            &PARENT_CLICKS,
            &PARENT_LONGS,
            &SURFACE_CLICKS,
            &SURFACE_LONGS,
        ] {
            counter.set(0);
        }
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: (480.0 * scale) as u32,
                height: (400.0 * scale) as u32,
                scale_factor: scale,
                sample_count: samples,
                frame_time: Duration::from_millis(50),
                ..Default::default()
            },
        )
        .unwrap();
        let outside = PxPosition::from_f32_xy((470.0 * scale) as f32, (390.0 * scale) as f32);
        renderer.pointer_move(MOUSE_POINTER_ID, outside);
        renderer.render_frames(4);
        let baseline = renderer.read_pixels().unwrap();
        let settings = SETTINGS.get().unwrap();
        let primary = center(&renderer, "Primary target");
        press(&mut renderer, primary);
        release(&mut renderer, primary);
        assert_eq!(SELECTIONS.get(), 1);
        settings.with_mut(|s| s.selected = false);
        renderer.render_frames(3);
        press(&mut renderer, primary);
        renderer.pointer_move(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(primary.x + scale as f32, primary.y),
        );
        renderer.step_frame();
        renderer.render_frames(8);
        assert_eq!(LONGS.get(), 0);
        renderer.step_frame();
        assert_eq!(
            LONGS.get(),
            1,
            "a hold with sub-slop jitter must fire at its frame deadline"
        );
        assert!(settings.get().selected);
        renderer.render_frames(30);
        assert_eq!(LONGS.get(), 1);
        release(&mut renderer, primary);
        assert_eq!(
            SELECTIONS.get(),
            1,
            "a long hold must suppress selection on release after mode changes"
        );
        assert!(settings.get().selected);
        press(&mut renderer, primary);
        release(&mut renderer, primary);
        assert_eq!(SELECTIONS.get(), 2);
        assert!(!settings.get().selected);
        let only = center(&renderer, "Long-only target");
        let only_node = node(&renderer, "Long-only target").1;
        assert!(only_node.supports_action(Action::ShowContextMenu));
        assert!(only_node.supports_action(Action::Focus));
        assert!(!only_node.supports_action(Action::Click));
        press(&mut renderer, only);
        release(&mut renderer, only);
        assert_eq!(ONLY_LONGS.get(), 0);
        assert!(action(
            &mut renderer,
            "Long-only target",
            Action::ShowContextMenu
        ));
        renderer.render_frames(2);
        assert_eq!(ONLY_LONGS.get(), 1);
        let child = center(&renderer, "Child target");
        press(&mut renderer, child);
        renderer.render_frames(12);
        release(&mut renderer, child);
        assert_eq!(CHILD_CLICKS.get(), 1);
        assert_eq!(PARENT_CLICKS.get(), 0);
        assert_eq!(PARENT_LONGS.get(), 0);
        let bounds = node(&renderer, "Parent target").1.bounds().unwrap();
        let parent = PxPosition::from_f32_xy(
            (bounds.x1 - 8.0 * scale) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        press(&mut renderer, parent);
        renderer.render_frames(12);
        release(&mut renderer, parent);
        assert_eq!(PARENT_LONGS.get(), 1);
        assert_eq!(PARENT_CLICKS.get(), 0);
        let direct = center(&renderer, "Surface target");
        press(&mut renderer, direct);
        renderer.render_frames(12);
        release(&mut renderer, direct);
        assert_eq!(SURFACE_LONGS.get(), 1);
        assert_eq!(SURFACE_CLICKS.get(), 0);
        press(&mut renderer, direct);
        renderer.pointer_move(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(direct.x + 40.0 * scale as f32, direct.y),
        );
        renderer.render_frames(12);
        release(&mut renderer, direct);
        assert_eq!(SURFACE_LONGS.get(), 1);
        assert_eq!(SURFACE_CLICKS.get(), 0);
        press(&mut renderer, direct);
        renderer.pointer_scroll(MOUSE_POINTER_ID, direct, 0.0, -40.0);
        renderer.render_frames(12);
        release(&mut renderer, direct);
        assert_eq!(SURFACE_LONGS.get(), 1);
        assert_eq!(SURFACE_CLICKS.get(), 0);
        press(&mut renderer, direct);
        renderer.render_frames(3);
        FocusManager::current().clear_focus(true);
        renderer.render_frames(12);
        release(&mut renderer, direct);
        assert_eq!(SURFACE_LONGS.get(), 1);
        assert_eq!(SURFACE_CLICKS.get(), 0);
        press(&mut renderer, direct);
        renderer.render_frames(3);
        settings.with_mut(|s| s.enabled = false);
        renderer.render_frames(12);
        release(&mut renderer, direct);
        assert_eq!(SURFACE_LONGS.get(), 1);
        assert!(!action(
            &mut renderer,
            "Surface target",
            Action::ShowContextMenu
        ));
        settings.with_mut(|s| s.enabled = true);
        renderer.render_frames(3);
        press(&mut renderer, only);
        renderer.render_frames(3);
        settings.with_mut(|s| s.handlers = false);
        renderer.render_frames(12);
        release(&mut renderer, only);
        assert_eq!(ONLY_LONGS.get(), 1);
        settings.with_mut(|s| {
            s.handlers = true;
            s.selected = false;
            s.mode = ListItemSelectionMode::Single;
        });
        FocusManager::current().clear_focus(true);
        renderer.pointer_move(MOUSE_POINTER_ID, outside);
        renderer.render_frames(40);
        assert!(
            renderer.read_pixels().unwrap() == baseline,
            "cancelled and removed handlers must restore idle feedback"
        );
        let direct = center(&renderer, "Surface target");
        press(&mut renderer, direct);
        assert!(
            SURFACE_INTERACTION
                .get()
                .unwrap()
                .with(InteractionState::is_pressed)
        );
        settings.with_mut(|s| s.show_surface = false);
        renderer.render_frames(30);
        assert!(
            !SURFACE_INTERACTION
                .get()
                .unwrap()
                .with(InteractionState::is_pressed),
            "unmount must release externally owned interaction state"
        );
        assert!(
            SURFACE_RIPPLE
                .get()
                .unwrap()
                .with(|state| state.animation_snapshot())
                .is_none(),
            "unmount must release externally owned ripple state"
        );
        settings.with_mut(|s| s.show_surface = true);
        FocusManager::current().clear_focus(true);
        renderer.pointer_move(MOUSE_POINTER_ID, outside);
        renderer.render_frames(40);
        assert!(
            renderer.read_pixels().unwrap() == baseline,
            "remount must restore idle feedback"
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn long_actions_fire_stationary_suppress_clicks_and_cancel_with_input_changes_msaa1() {
    crate::test_utils::run_isolated_test(
        "list_item::long_click_tests::long_actions_fire_stationary_suppress_clicks_and_cancel_with_input_changes_msaa1",
        || check(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn long_actions_fire_stationary_suppress_clicks_and_cancel_with_input_changes_msaa4() {
    crate::test_utils::run_isolated_test(
        "list_item::long_click_tests::long_actions_fire_stationary_suppress_clicks_and_cancel_with_input_changes_msaa4",
        || check(4),
    );
}
