//! Sheet figures under parent scrolling and fractional placement.
//!
//! ## Usage
//!
//! Keep diagram annotations and native sheet actions aligned at narrow widths.

use std::{cell::Cell, time::Duration};

use tessera_components::{
    alignment::Alignment, boxed::boxed, column::column, scrollable::ScrollableController,
    spacer::spacer, theme::material_theme,
};
use tessera_ui::{
    CursorEventContent, GestureState, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID,
    PlacementModifierNode, PointerChange, PressKeyEventType, PxPosition, ScrollDeltaUnit,
    ScrollEventContent, ScrollEventSource,
    accesskit::{Node, Role, Toggled},
    current_frame_time,
    modifier::ModifierCapabilityExt as _,
};

use super::*;

thread_local! {
    static SCENE: Cell<usize> = const { Cell::new(0) };
    pub(super) static GEOMETRY: Cell<Option<State<Geometry>>> = const { Cell::new(None) };
    pub(super) static NOTICE: Cell<Option<State<String>>> = const { Cell::new(None) };
    static SCROLL: Cell<Option<State<ScrollableController>>> = const { Cell::new(None) };
}

#[derive(Clone, Copy, PartialEq)]
struct FractionalOrigin;
impl PlacementModifierNode for FractionalOrigin {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        position + PxPosition::from_f32_xy(10.25, 20.5)
    }
}

#[tessera]
fn fixture() {
    let scroll = remember(ScrollableController::new);
    SCROLL.set(Some(scroll));
    material_theme().child(move || {
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(
                Modifier::new()
                    .fill_max_width()
                    .push_placement(FractionalOrigin),
            )
            .children(move || {
                column()
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .height(Dp(600.0))
                            .vertical_scroll(scroll),
                    )
                    .children(|| {
                        spacer().modifier(Modifier::new().height(Dp(160.0)));
                        match SCENE.get() {
                            0 => {
                                figure(false, true, false);
                            }
                            1 => {
                                figure(false, true, true);
                            }
                            2 => {
                                figure(true, false, false);
                            }
                            3 => {
                                figure(true, true, false);
                            }
                            4 => {
                                figure(true, true, true);
                            }
                            5 => {
                                music_context();
                            }
                            _ => {
                                filter_context();
                            }
                        }
                        spacer().modifier(Modifier::new().height(Dp(700.0)));
                    });
            });
    });
}

struct Pipelines;
impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> Node {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing {label} {role:?}"))
        .1
}

fn click(renderer: &mut HeadlessRenderer, label: &str, role: Role) {
    let bounds = node(renderer, label, role).bounds().unwrap();
    assert!(bounds.width() > 0.0 && bounds.height() > 0.0);
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(4);
}

fn renderer(scene: usize, samples: u32) -> HeadlessRenderer {
    SCENE.set(scene);
    GEOMETRY.set(None);
    NOTICE.set(None);
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 640,
            height: 800,
            sample_count: samples,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .expect("Native GPU adapter");
    renderer.render_frames(16);
    renderer.inject_pointer(
        PointerChange::new(
            current_frame_time().unwrap(),
            MOUSE_POINTER_ID,
            CursorEventContent::Scroll(ScrollEventContent {
                delta_x: 0.0,
                delta_y: -100.0,
                unit: ScrollDeltaUnit::Pixel,
                source: ScrollEventSource::Wheel,
            }),
            GestureState::default(),
        )
        .with_position(PxPosition::from_f32_xy(40.0, 100.0)),
    );
    renderer.render_frames(120);
    assert_eq!(
        SCROLL
            .get()
            .unwrap()
            .with(|scroll| scroll.child_position().y),
        -100.0
    );
    renderer
}

fn assert_figure(scene: usize, width: f32) {
    let geometry = GEOMETRY.get().unwrap().get();
    let root = geometry["figure"];
    let canonical_width = if scene >= 2 { 1280.0 } else { 1024.0 };
    assert_eq!(root.local_to_window.scale(), width / canonical_width);
    assert_eq!(
        (
            root.local_to_window.translation().x,
            root.local_to_window.translation().y
        ),
        (10.25, 80.5)
    );
    assert_eq!(root.local_size.width.0, canonical_width as i32);
    assert_eq!(root.local_size.height.0, [502, 836, 1045, 1045, 500][scene]);
    let container = annotations::relative_precise(&geometry, "container").unwrap();
    assert!((container.w - if scene >= 2 { 400.0 } else { 640.0 }).abs() < 0.001);
}

#[test]
#[ignore = "requires a native GPU adapter for fractional sheet diagram geometry and pointer input"]
fn sheet_diagrams_keep_canonical_annotations_and_native_actions_after_resize() {
    for samples in [1, 4] {
        for scene in 0..5 {
            let mut renderer = renderer(scene, samples);
            assert_figure(scene, 640.0);
            renderer.resize(512, 800);
            renderer.render_frames(16);
            assert_figure(scene, 512.0);
            if scene >= 2 {
                click(&mut renderer, "Sheet figure close", Role::Button);
                assert_eq!(NOTICE.get().unwrap().get(), "Sheet close action selected");
                if scene >= 3 {
                    click(&mut renderer, "Sheet figure back", Role::Button);
                    assert_eq!(NOTICE.get().unwrap().get(), "Back action selected");
                }
            }
        }
    }
}

#[test]
#[ignore = "requires a native GPU adapter for scaled music and filter controls"]
fn narrow_sheet_contexts_keep_playback_photo_updates_and_reopened_filter_state() {
    for samples in [1, 4] {
        let mut music = renderer(5, samples);
        music.resize(512, 800);
        music.render_frames(16);
        let pause = node(&music, "Pause", Role::Button).bounds().unwrap();
        assert!((pause.width() - 48.0 * 512.0 / 676.0).abs() < 0.001);
        click(&mut music, "Pause", Role::Button);
        node(&music, "Play", Role::Button);
        click(&mut music, "Play", Role::Button);
        node(&music, "Pause", Role::Button);
        let before = music.read_pixels().unwrap();
        click(&mut music, "Next track", Role::Button);
        let after = music.read_pixels().unwrap();
        let scale: f64 = 512.0 / 676.0;
        let (x0, x1) = (
            (10.25 + 132.0 * scale).ceil() as usize,
            (10.25 + 196.0 * scale).floor() as usize,
        );
        let (y0, y1) = (
            (80.5 + 140.0 * scale).ceil() as usize,
            (80.5 + 208.0 * scale).floor() as usize,
        );
        let changed = (y0..y1)
            .flat_map(|y| (x0..x1).map(move |x| (y * 512 + x) * 4))
            .filter(|&i| before[i..i + 4] != after[i..i + 4])
            .count();
        assert!(
            changed > (x1 - x0) * (y1 - y0) / 10,
            "Next must update the real cropped cover photo"
        );
        drop(music);
        let mut filters = renderer(6, samples);
        filters.resize(512, 800);
        filters.render_frames(16);
        assert_eq!(
            node(&filters, "Events", Role::CheckBox).toggled(),
            Some(Toggled::False)
        );
        click(&mut filters, "Events", Role::CheckBox);
        click(&mut filters, "Projects", Role::CheckBox);
        click(&mut filters, "Close filters", Role::Button);
        assert!(
            filters
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .all(|(_, n)| n.role() != Role::CheckBox)
        );
        click(&mut filters, "Open filters", Role::Button);
        for (label, selected) in [("Events", true), ("Projects", false), ("Reminders", true)] {
            assert_eq!(
                node(&filters, label, Role::CheckBox).toggled(),
                Some(if selected {
                    Toggled::True
                } else {
                    Toggled::False
                })
            );
        }
    }
}
