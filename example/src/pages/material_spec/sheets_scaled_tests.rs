use tessera_components::{
    alignment::Alignment, boxed::boxed, column::column, scrollable::ScrollableController,
    spacer::spacer, theme::material_theme,
};
use tessera_ui::{
    CursorEventContent, GestureState, MOUSE_POINTER_ID, PlacementModifierNode, PointerChange,
    PressKeyEventType, PxPosition, ScrollDeltaUnit, ScrollEventContent, ScrollEventSource,
    accesskit::{Node, Role, Toggled},
    current_frame_time,
    modifier::ModifierCapabilityExt as _,
    testing::{TestProbe, UiTest},
};

use super::*;

#[derive(Clone, Copy, PartialEq)]
struct FractionalOrigin;
impl PlacementModifierNode for FractionalOrigin {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        position + PxPosition::from_f32_xy(10.25, 20.5)
    }
}

#[tessera]
fn fixture(scene: usize, probe: TestProbe<Option<State<ScrollableController>>>) {
    let scroll = remember(ScrollableController::new);
    probe.set(Some(scroll));
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
                    .children(move || {
                        spacer().modifier(Modifier::new().height(Dp(160.0)));
                        match scene {
                            0 => {
                                figures::anatomy_figure(false);
                            }
                            1 => {
                                figures::measurements_figure();
                            }
                            2 => {
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

fn node(renderer: &UiTest, label: &str, role: Role) -> Node {
    let snapshot = renderer
        .inspect_accessibility()
        .expect("accessibility tree");
    snapshot
        .find(&crate::res::I18N.text(label), role)
        .unwrap_or_else(|| panic!("missing {label} {role:?}"))
        .1
        .clone()
}

fn click(renderer: &mut UiTest, label: &str, role: Role) {
    let bounds = node(renderer, label, role).bounds().unwrap();
    assert!(bounds.width() > 0.0 && bounds.height() > 0.0);
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.frames(4);
}

fn renderer(scene: usize, samples: u32) -> UiTest<'static> {
    let probe = TestProbe::default();
    let observed = probe.clone();
    let mut renderer = UiTest::new()
        .viewport(640, 800)
        .samples(samples)
        .rendered(crate::TestPipelines)
        .content(move || fixture(scene, probe.clone()));
    renderer.frames(16);
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
    renderer.frames(120);
    assert_eq!(
        observed
            .get()
            .unwrap()
            .with(|scroll| scroll.child_position().y),
        -100.0
    );
    renderer
}

fn assert_figure(renderer: &UiTest, scene: usize, width: f32) {
    let snapshot = renderer.inspect_components();
    let geometry: Geometry = ["figure", "screen", "scrim", "container", "handle"]
        .into_iter()
        .filter_map(|name| {
            snapshot
                .find_tag(&format!("sheet-figure-{name}"))
                .and_then(|node| node.layout_coordinates())
                .map(|value| (name, value))
        })
        .collect();
    let root = geometry["figure"];
    assert_eq!(root.local_to_window.scale(), width / 1024.0);
    assert_eq!(
        (
            root.local_to_window.translation().x,
            root.local_to_window.translation().y
        ),
        (10.25, 80.5)
    );
    assert_eq!(root.local_size.width.0, 1024);
    assert_eq!(root.local_size.height.0, [502, 836][scene]);
    let near = |actual: f64, expected: f64| (actual - expected).abs() < 0.01;
    let container = annotations::relative_precise(&geometry, "container").unwrap();
    let top = [166.0, 399.0][scene];
    assert!(
        near(container.x, 192.0) && near(container.y, top) && near(container.w, 640.0),
        "the 640dp sheet must stay centred on the canonical stage: {container:?}"
    );
    let handle = annotations::relative_precise(&geometry, "handle").unwrap();
    assert!(
        near(handle.x + handle.w / 2.0, 512.0) && near(handle.y, top + 22.0),
        "the drag handle must keep the 22dp top padding from the measurements table: {handle:?}"
    );
    let screen = annotations::relative_precise(&geometry, "screen").unwrap();
    assert!(
        near(container.y + container.h, screen.y + screen.h),
        "the sheet must reach the bottom of the phone screen: container={container:?}, screen={screen:?}"
    );
    if scene == 0 {
        let scale = f64::from(width) / 1024.0;
        for (number, center) in [(1, 284.0), (2, 512.0), (3, 969.0)] {
            let bounds = node(
                renderer,
                &format!("Bottom sheet marker {number}"),
                Role::Note,
            )
            .bounds()
            .unwrap();
            let x = ((bounds.x0 + bounds.x1) / 2.0 - 10.25) / scale;
            assert!(
                (x - center).abs() < 0.01,
                "anatomy marker {number} must stay on its leader line: {x}"
            );
        }
        let marker = node(renderer, "Bottom sheet marker 1", Role::Note)
            .bounds()
            .unwrap();
        assert!(((marker.y1 - 80.5) / scale - 137.0).abs() < 0.01);
    }
}

#[test]
#[ignore = "requires a native GPU adapter for fractional sheet diagram geometry"]
fn sheet_diagrams_keep_markers_on_native_geometry_after_resize() {
    for samples in [1, 4] {
        for scene in 0..2 {
            let mut renderer = renderer(scene, samples);
            assert_figure(&renderer, scene, 640.0);
            renderer.resize(512, 800);
            renderer.frames(16);
            assert_figure(&renderer, scene, 512.0);
        }
    }
}

#[test]
#[ignore = "requires a native GPU adapter for scaled music and filter controls"]
fn narrow_sheet_contexts_keep_playback_photo_updates_and_reopened_filter_state() {
    for samples in [1, 4] {
        let mut music = renderer(2, samples);
        music.resize(512, 800);
        music.frames(16);
        let pause = node(&music, "Pause", Role::Button).bounds().unwrap();
        assert!((pause.width() - 48.0 * 512.0 / 676.0).abs() < 0.001);
        click(&mut music, "Pause", Role::Button);
        node(&music, "Play", Role::Button);
        click(&mut music, "Play", Role::Button);
        node(&music, "Pause", Role::Button);
        let before = music.read_pixels();
        click(&mut music, "Next track", Role::Button);
        let after = music.read_pixels();
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
        let mut filters = renderer(3, samples);
        filters.resize(512, 800);
        filters.frames(16);
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
