//! Focus indicator regressions for selectable navigation destinations.
//!
//! ## Usage
//!
//! Verify that focus is visible without selecting or activating a destination.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    Color, Dp, FocusDirection, FocusManager, HeadlessConfig, HeadlessRenderer, Modifier, State,
    accesskit::{Action, ActionRequest, Rect, Role, TreeId},
    remember, tessera,
};

use crate::{
    column::column,
    modifier::ModifierExt as _,
    navigation_bar::{NavigationBarIconPosition, navigation_bar, navigation_bar_item},
    navigation_drawer::{navigation_drawer, navigation_drawer_item},
    navigation_rail::{navigation_rail, navigation_rail_item},
    spacer::spacer,
    theme::{MaterialTheme, material_theme},
};

thread_local! {
    static MODE: Cell<u8> = const { Cell::new(0) };
    static OWNER: Cell<Option<(State<usize>, State<usize>)>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let selected = remember(|| 0_usize);
    let activations = remember(|| 0_usize);
    OWNER.set(Some((selected, activations)));
    material_theme()
        .theme(|| {
            let mut theme = MaterialTheme::default();
            theme.color_scheme.secondary = Color::from_rgb(0.0, 1.0, 0.0);
            theme
        })
        .child(move || {
            column()
                .modifier(Modifier::new().fill_max_size().padding_all(Dp(24.0)))
                .children(move || {
                    if MODE.get() == 0 {
                        navigation_drawer().content(move || {
                            for (index, label) in ["Inbox", "Saved"].into_iter().enumerate() {
                                navigation_drawer_item()
                                    .label(label)
                                    .selected(selected.get() == index)
                                    .on_click(move || {
                                        selected.set(index);
                                        activations.set(activations.get() + 1);
                                    });
                            }
                        });
                    } else if MODE.get() <= 2 {
                        navigation_rail()
                            .expanded(MODE.get() == 2)
                            .content(move || {
                                for (index, label) in ["Inbox", "Saved"].into_iter().enumerate() {
                                    navigation_rail_item()
                                        .label(label)
                                        .selected(selected.get() == index)
                                        .on_click(move || {
                                            selected.set(index);
                                            activations.set(activations.get() + 1);
                                        })
                                        .icon(|| {
                                            spacer()
                                                .modifier(Modifier::new().size(Dp(24.0), Dp(24.0)));
                                        });
                                }
                            });
                    } else {
                        navigation_bar()
                            .icon_position(if MODE.get() == 3 {
                                NavigationBarIconPosition::Top
                            } else {
                                NavigationBarIconPosition::Start
                            })
                            .content(move || {
                                for (index, label) in ["Inbox", "Saved"].into_iter().enumerate() {
                                    navigation_bar_item()
                                        .label(label)
                                        .selected(selected.get() == index)
                                        .on_click(move || {
                                            selected.set(index);
                                            activations.set(activations.get() + 1);
                                        })
                                        .icon(|| {
                                            spacer()
                                                .modifier(Modifier::new().size(Dp(24.0), Dp(24.0)));
                                        });
                                }
                            });
                    }
                });
        });
}

struct Pipelines;
impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        crate::pipelines::register_pipelines(context);
    }
}

fn indicator_pixels(renderer: &HeadlessRenderer, bounds: Rect) -> usize {
    let size = renderer.physical_size();
    let pixels = renderer.read_pixels().unwrap();
    let x0 = (bounds.x0 - 6.0).floor().max(0.0) as u32;
    let y0 = (bounds.y0 - 6.0).floor().max(0.0) as u32;
    let x1 = (bounds.x1 + 6.0).ceil().min(f64::from(size.width)) as u32;
    let y1 = (bounds.y1 + 6.0).ceil().min(f64::from(size.height)) as u32;
    (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let offset = ((y * size.width + x) * 4) as usize;
            pixels[offset] < 10 && pixels[offset + 1] > 240 && pixels[offset + 2] < 10
        })
        .count()
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn navigation_initial_traversal_and_accessibility_focus_do_not_activate_destinations() {
    for mode in 0..5 {
        MODE.set(mode);
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 600,
                height: 800,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(32);
        let snapshot = renderer.accessibility_snapshot().unwrap();
        let (inbox_id, inbox) = snapshot
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Tab && node.label() == Some("Inbox"))
            .unwrap();
        let inbox_id = *inbox_id;
        if matches!(mode, 1 | 2) {
            let bounds = inbox.bounds().unwrap();
            assert_eq!(
                bounds.y0, 60.0,
                "the native rail must keep its 36dp top inset"
            );
            assert_eq!(
                bounds.height(),
                56.0,
                "a normal rail destination must be 56dp tall"
            );
            assert_eq!(bounds.width(), if mode == 1 { 96.0 } else { 220.0 });
        }
        assert!(FocusManager::current().request_move_focus(FocusDirection::Next));
        renderer.step_frame();
        assert_eq!(
            renderer.accessibility_snapshot().unwrap().focus,
            inbox_id,
            "mode {mode}: initial traversal must enter the first actual destination"
        );
        let (selected, activations) = OWNER.get().unwrap();
        assert_eq!(selected.get(), 0, "mode {mode}: entering must not select");
        assert_eq!(
            activations.get(),
            0,
            "mode {mode}: entering must not activate"
        );
        let (id, saved) = snapshot
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Tab && node.label() == Some("Saved"))
            .unwrap();
        let id = *id;
        let bounds = saved.bounds().unwrap();
        assert_eq!(
            indicator_pixels(&renderer, bounds),
            0,
            "mode {mode}: inactive destination must not have a focus outline"
        );
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action: Action::Focus,
            target_tree: TreeId::ROOT,
            target_node: id,
            data: None
        }));
        renderer.render_frames(4);
        let (selected, activations) = OWNER.get().unwrap();
        assert_eq!(selected.get(), 0, "mode {mode}: focus must not select");
        assert_eq!(activations.get(), 0, "mode {mode}: focus must not activate");
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, id);
        assert!(
            indicator_pixels(&renderer, bounds) > 0,
            "mode {mode}: the actual focused destination must draw its secondary outline"
        );
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: id,
            data: None
        }));
        renderer.render_frames(4);
        assert_eq!(selected.get(), 1);
        assert_eq!(activations.get(), 1);
    }
}
