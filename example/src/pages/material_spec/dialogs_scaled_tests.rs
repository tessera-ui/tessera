use tessera_components::{
    alignment::Alignment, boxed::boxed, column::column, modifier::ModifierExt as _,
    scrollable::ScrollableController, spacer::spacer, theme::material_theme,
};
use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    CursorEventContent, Dp, GestureState, MOUSE_POINTER_ID, Modifier, PlacementModifierNode,
    PointerChange, PressKeyEventType, PxPosition, ScrollDeltaUnit, ScrollEventContent,
    ScrollEventSource, State,
    accesskit::{Node, Role},
    modifier::ModifierCapabilityExt as _,
    remember, tessera,
    testing::UiTest,
};

use super::{Geometry, annotations};

#[derive(Clone, Copy, PartialEq)]
struct FractionalOrigin;

impl PlacementModifierNode for FractionalOrigin {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        position + PxPosition::from_f32_xy(10.25, 20.5)
    }
}

#[tessera]
fn fixture(
    scene: usize,
    output: tessera_ui::testing::TestProbe<Option<State<ScrollableController>>>,
) {
    let scroll = remember(ScrollableController::new);
    output.set(Some(scroll));
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
                            .height(Dp(500.0))
                            .vertical_scroll(scroll),
                    )
                    .children(move || {
                        spacer().modifier(Modifier::new().height(Dp(200.0)));
                        match scene {
                            0 => {
                                super::basic_figure(false);
                            }
                            1 => {
                                super::basic_figure(true);
                            }
                            2 => {
                                super::basic_color_figure();
                            }
                            3..=5 => {
                                super::full_spec_figure(scene - 3);
                            }
                            _ => {
                                super::dialog_overview_pair();
                            }
                        }
                        spacer().modifier(Modifier::new().height(Dp(600.0)));
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

fn assert_geometry(renderer: &UiTest, scene: usize, scale: f32) {
    if scene == 6 {
        let bounds = node(renderer, "Dialogs overview figure", Role::Group)
            .bounds()
            .unwrap();
        assert_eq!((bounds.x0, bounds.y0), (10.25, 80.5));
        assert_eq!(
            (bounds.width(), bounds.height()),
            (1024.0 * scale as f64, 622.0 * scale as f64)
        );
        return;
    }
    let snapshot = renderer.inspect_components();
    let values: Geometry = ["figure", "container"]
        .into_iter()
        .map(|name| {
            (
                name,
                snapshot
                    .tagged(&format!("dialog-figure-{name}"))
                    .layout_coordinates()
                    .unwrap(),
            )
        })
        .collect();
    let figure = values["figure"];
    assert_eq!(figure.local_to_window.scale(), scale);
    assert_eq!(
        (
            figure.local_to_window.translation().x,
            figure.local_to_window.translation().y
        ),
        (10.25, 80.5)
    );
    assert_eq!(figure.local_size.width.0, 1024);
    assert_eq!(
        figure.local_size.height.0,
        [698, 426, 486, 463, 508, 1030][scene]
    );
    let container = annotations::relative_precise(&values, "container").unwrap();
    let expected = match scene {
        0 => (356.0, 123.0, 312.0),
        1 => (360.0, 70.0, 312.0),
        2 => (341.0, 114.0, 340.0),
        3 => (298.0, 103.0, 428.0),
        4 => (298.0, 98.0, 428.0),
        _ => (298.0, 67.0, 428.0),
    };
    assert_eq!((container.x, container.y, container.w), expected);
    if scene == 0 {
        let marker = node(renderer, "Anatomy marker 1", Role::Note)
            .bounds()
            .unwrap();
        assert_eq!(
            (marker.x0, marker.y0, marker.width(), marker.height()),
            (
                10.25 + 304.0 * scale as f64,
                80.5 + 147.0 * scale as f64,
                24.0 * scale as f64,
                32.0 * scale as f64
            )
        );
    }
}

fn exercise(scenes: &[usize]) {
    for samples in [1, 4] {
        for &scene in scenes {
            let output = tessera_ui::testing::TestProbe::default();
            let mut renderer = UiTest::new()
                .viewport(640, 800)
                .samples(samples)
                .rendered(crate::TestPipelines)
                .content(|| fixture(scene, output.clone()));
            renderer.frames(16);
            renderer.inject_pointer(
                PointerChange::new(
                    renderer.frame_origin() + renderer.frame_time(),
                    MOUSE_POINTER_ID,
                    CursorEventContent::Scroll(ScrollEventContent {
                        delta_x: 0.0,
                        delta_y: -140.0,
                        unit: ScrollDeltaUnit::Pixel,
                        source: ScrollEventSource::Wheel,
                    }),
                    GestureState::default(),
                )
                .with_position(PxPosition::from_f32_xy(40.0, 100.0)),
            );
            renderer.frames(120);
            assert_eq!(
                output
                    .get()
                    .unwrap()
                    .with(|scroll| scroll.child_position().y),
                -140.0
            );
            assert_geometry(&renderer, scene, 0.625);
            renderer.resize(512, 800);
            renderer.frames(16);
            assert_geometry(&renderer, scene, 0.5);
            let (label, expected_notice, width) = match scene {
                0 => ("Accept", "Accepted", 80.0),
                1 => ("Action 1", "Accepted", 84.0),
                2 => ("Accept color dialog", "Color dialog accepted", 88.0),
                3..=5 => ("Save spec event", "Event saved", 64.0),
                _ => ("Overview basic confirm", "Basic dialog accepted", 112.0),
            };
            let bounds = node(&renderer, label, Role::Button).bounds().unwrap();
            assert_eq!((bounds.width(), bounds.height()), (width * 0.5, 20.0));
            let position = PxPosition::from_f32_xy(
                ((bounds.x0 + bounds.x1) * 0.5) as f32,
                ((bounds.y0 + bounds.y1) * 0.5) as f32,
            );
            renderer.pointer_move(MOUSE_POINTER_ID, position);
            renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
            renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
            renderer.frames(4);
            assert!(
                renderer
                    .inspect_accessibility()
                    .unwrap()
                    .nodes()
                    .iter()
                    .any(|(_, node)| crate::has_label(&node, expected_notice)),
                "the activated action publishes its notice"
            );
        }
    }
}

#[test]
#[ignore = "requires a native GPU adapter for canonical dialog figures and real pointer input"]
fn basic_and_overview_dialogs_keep_canonical_geometry_while_scrolled_and_resized() {
    exercise(&[0, 1, 2, 6]);
}

#[test]
#[ignore = "requires a native GPU adapter for full dialog figures and real pointer input"]
fn full_dialog_specs_keep_native_controls_and_annotations_in_one_transform() {
    exercise(&[3, 4, 5]);
}
