//! Stationary long actions and click arbitration through real components.
//!
//! ## Usage
//!
//! Keep long holds distinct from selection, child clicks, and cancelled input.

use std::time::Duration;

use tessera_foundation::modifier::{InteractionState, ModifierExt as _};
use tessera_ui::{
    Dp, MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition, State,
    accesskit::Action,
    focus, remember, tessera,
    testing::{TestProbe, UiTest},
};

use crate::{
    column::column,
    md3::{
        button::button, ripple_state::RippleState, surface::surface, text::text,
        theme::material_theme,
    },
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

#[derive(Clone, Copy, Default, PartialEq)]
struct FixtureHandles {
    settings: Option<State<Settings>>,
    interaction: Option<State<InteractionState>>,
    ripple: Option<State<RippleState>>,
}

#[derive(Clone, Copy, Default)]
struct Events {
    selections: usize,
    longs: usize,
    only_longs: usize,
    child_clicks: usize,
    parent_clicks: usize,
    parent_longs: usize,
    surface_clicks: usize,
    surface_longs: usize,
}

#[tessera]
fn fixture(probe: TestProbe<FixtureHandles>, events: TestProbe<Events>) {
    let settings = remember(|| Settings {
        enabled: true,
        handlers: true,
        selected: false,
        mode: ListItemSelectionMode::Single,
        show_surface: true,
    });
    let interaction = remember(InteractionState::new);
    let ripple = remember(RippleState::new);
    probe.set(FixtureHandles {
        settings: Some(settings),
        interaction: Some(interaction),
        ripple: Some(ripple),
    });
    material_theme().child({
        let events = events.clone();
        move || {
            column()
                .modifier(Modifier::new().width(Dp(320.0)))
                .children({
                    let events = events.clone();
                    move || {
                        list_item()
                            .headline("Selection row")
                            .accessibility_label("Primary target")
                            .selection_mode(settings.get().mode)
                            .selected(settings.get().selected)
                            .enabled(settings.get().enabled)
                            .on_selection_change({
                                let events = events.clone();
                                move |value| {
                                    events.with_mut(|events| events.selections += 1);
                                    settings.with_mut(|s| s.selected = value);
                                }
                            })
                            .on_long_click({
                                let events = events.clone();
                                move || {
                                    events.with_mut(|events| events.longs += 1);
                                    settings.with_mut(|s| {
                                        s.mode = ListItemSelectionMode::Multiple;
                                        s.selected = true;
                                    });
                                }
                            });
                        let long_only = if settings.get().handlers {
                            Some(tessera_ui::Callback::new({
                                let events = events.clone();
                                move || events.with_mut(|events| events.only_longs += 1)
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
                            .on_click({
                                let events = events.clone();
                                move || events.with_mut(|events| events.parent_clicks += 1)
                            })
                            .on_long_click({
                                let events = events.clone();
                                move || events.with_mut(|events| events.parent_longs += 1)
                            })
                            .content({
                                let events = events.clone();
                                move || {
                                    button()
                                        .modifier(Modifier::new().width(Dp(100.0)))
                                        .accessibility_label("Child target")
                                        .on_click({
                                            let events = events.clone();
                                            move || {
                                                events.with_mut(|events| events.child_clicks += 1)
                                            }
                                        })
                                        .child(|| {
                                            text("Child");
                                        });
                                }
                            });
                        let surface_long = if settings.get().handlers {
                            Some(tessera_ui::Callback::new({
                                let events = events.clone();
                                move || events.with_mut(|events| events.surface_longs += 1)
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
                                .on_click({
                                    let events = events.clone();
                                    move || events.with_mut(|events| events.surface_clicks += 1)
                                })
                                .on_long_click_optional(surface_long)
                                .child(|| {
                                    text("Surface");
                                });
                        }
                    }
                });
        }
    });
}

fn center(renderer: &UiTest, label: &str) -> PxPosition {
    let bounds = renderer.accessibility_node(label).1.bounds().unwrap();
    PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    )
}

fn action(renderer: &mut UiTest, label: &str, action: Action) -> bool {
    let id = renderer.accessibility_node(label).0;
    renderer.accessibility_action(id, action)
}

fn press(renderer: &mut UiTest, point: PxPosition) {
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.frames(1);
}

fn release(renderer: &mut UiTest, point: PxPosition) {
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.frames(3);
}

fn check() {
    for scale in [1.0, 1.5] {
        let probe = TestProbe::default();
        let observed = probe.clone();
        let events = TestProbe::<Events>::default();
        let observed_events = events.clone();
        let mut renderer = UiTest::new()
            .viewport((480.0 * scale) as u32, (400.0 * scale) as u32)
            .scale_factor(scale)
            .frame_step(Duration::from_millis(50))
            .rendered(crate::TesseraComponents)
            .content(move || fixture(probe.clone(), events.clone()));
        let outside = PxPosition::from_f32_xy((470.0 * scale) as f32, (390.0 * scale) as f32);
        renderer.pointer_move(MOUSE_POINTER_ID, outside);
        renderer.frames(4);
        let baseline = renderer.read_pixels();
        let settings = observed.get().settings.unwrap();
        let primary = center(&renderer, "Primary target");
        press(&mut renderer, primary);
        release(&mut renderer, primary);
        assert_eq!(observed_events.with(|events| events.selections), 1);
        settings.with_mut(|s| s.selected = false);
        renderer.frames(3);
        press(&mut renderer, primary);
        renderer.pointer_move(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(primary.x + scale as f32, primary.y),
        );
        renderer.frames(1);
        renderer.frames(7);
        assert_eq!(observed_events.with(|events| events.longs), 0);
        renderer.frames(1);
        assert_eq!(
            observed_events.with(|events| events.longs),
            1,
            "a hold with sub-slop jitter must fire at its frame deadline"
        );
        assert!(settings.get().selected);
        renderer.frames(30);
        assert_eq!(observed_events.with(|events| events.longs), 1);
        release(&mut renderer, primary);
        assert_eq!(
            observed_events.with(|events| events.selections),
            1,
            "a long hold must suppress selection on release after mode changes"
        );
        assert!(settings.get().selected);
        press(&mut renderer, primary);
        release(&mut renderer, primary);
        assert_eq!(observed_events.with(|events| events.selections), 2);
        assert!(!settings.get().selected);
        let only = center(&renderer, "Long-only target");
        let only_node = renderer.accessibility_node("Long-only target").1;
        assert!(only_node.supports_action(Action::ShowContextMenu));
        assert!(only_node.supports_action(Action::Focus));
        assert!(!only_node.supports_action(Action::Click));
        press(&mut renderer, only);
        release(&mut renderer, only);
        assert_eq!(observed_events.with(|events| events.only_longs), 0);
        assert!(action(
            &mut renderer,
            "Long-only target",
            Action::ShowContextMenu
        ));
        renderer.frames(2);
        assert_eq!(observed_events.with(|events| events.only_longs), 1);
        let child = center(&renderer, "Child target");
        press(&mut renderer, child);
        renderer.frames(12);
        release(&mut renderer, child);
        assert_eq!(observed_events.with(|events| events.child_clicks), 1);
        assert_eq!(observed_events.with(|events| events.parent_clicks), 0);
        assert_eq!(observed_events.with(|events| events.parent_longs), 0);
        let bounds = renderer
            .accessibility_node("Parent target")
            .1
            .bounds()
            .unwrap();
        let parent = PxPosition::from_f32_xy(
            (bounds.x1 - 8.0 * scale) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        press(&mut renderer, parent);
        renderer.frames(12);
        release(&mut renderer, parent);
        assert_eq!(observed_events.with(|events| events.parent_longs), 1);
        assert_eq!(observed_events.with(|events| events.parent_clicks), 0);
        let direct = center(&renderer, "Surface target");
        press(&mut renderer, direct);
        renderer.frames(12);
        release(&mut renderer, direct);
        assert_eq!(observed_events.with(|events| events.surface_longs), 1);
        assert_eq!(observed_events.with(|events| events.surface_clicks), 0);
        press(&mut renderer, direct);
        renderer.pointer_move(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(direct.x + 40.0 * scale as f32, direct.y),
        );
        renderer.frames(12);
        release(&mut renderer, direct);
        assert_eq!(observed_events.with(|events| events.surface_longs), 1);
        assert_eq!(observed_events.with(|events| events.surface_clicks), 0);
        press(&mut renderer, direct);
        renderer.pointer_scroll(MOUSE_POINTER_ID, direct, 0.0, -40.0);
        renderer.frames(12);
        release(&mut renderer, direct);
        assert_eq!(observed_events.with(|events| events.surface_longs), 1);
        assert_eq!(observed_events.with(|events| events.surface_clicks), 0);
        press(&mut renderer, direct);
        renderer.frames(3);
        focus::clear_focus(true);
        renderer.frames(12);
        release(&mut renderer, direct);
        assert_eq!(observed_events.with(|events| events.surface_longs), 1);
        assert_eq!(observed_events.with(|events| events.surface_clicks), 0);
        press(&mut renderer, direct);
        renderer.frames(3);
        settings.with_mut(|s| s.enabled = false);
        renderer.frames(12);
        release(&mut renderer, direct);
        assert_eq!(observed_events.with(|events| events.surface_longs), 1);
        assert!(!action(
            &mut renderer,
            "Surface target",
            Action::ShowContextMenu
        ));
        settings.with_mut(|s| s.enabled = true);
        renderer.frames(3);
        press(&mut renderer, only);
        renderer.frames(3);
        settings.with_mut(|s| s.handlers = false);
        renderer.frames(12);
        release(&mut renderer, only);
        assert_eq!(observed_events.with(|events| events.only_longs), 1);
        settings.with_mut(|s| {
            s.handlers = true;
            s.selected = false;
            s.mode = ListItemSelectionMode::Single;
        });
        focus::clear_focus(true);
        renderer.pointer_move(MOUSE_POINTER_ID, outside);
        renderer.frames(40);
        assert!(
            renderer.read_pixels() == baseline,
            "cancelled and removed handlers must restore idle feedback"
        );
        let direct = center(&renderer, "Surface target");
        press(&mut renderer, direct);
        assert!(
            observed
                .get()
                .interaction
                .unwrap()
                .with(InteractionState::is_pressed)
        );
        settings.with_mut(|s| s.show_surface = false);
        renderer.frames(30);
        assert!(
            !observed
                .get()
                .interaction
                .unwrap()
                .with(InteractionState::is_pressed),
            "unmount must release externally owned interaction state"
        );
        assert!(
            observed
                .get()
                .ripple
                .unwrap()
                .with(|state| state.animation())
                .is_none(),
            "unmount must release externally owned ripple state"
        );
        settings.with_mut(|s| s.show_surface = true);
        focus::clear_focus(true);
        renderer.pointer_move(MOUSE_POINTER_ID, outside);
        renderer.frames(40);
        assert!(
            renderer.read_pixels() == baseline,
            "remount must restore idle feedback"
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn long_actions_fire_stationary_suppress_clicks_and_cancel_with_input_changes() {
    crate::test_utils::run_isolated_test(
        "md3::list_item::long_click_tests::long_actions_fire_stationary_suppress_clicks_and_cancel_with_input_changes",
        check,
    );
}
