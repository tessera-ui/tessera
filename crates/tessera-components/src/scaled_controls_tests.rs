use std::time::Duration;

use tessera_ui::{
    Color, Dp, FloatRect, LayoutCoordinates, MOUSE_POINTER_ID, Modifier, PressKeyEventType, Px,
    PxPosition, State,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, TreeId},
    focus,
    headless::{HeadlessConfig, HeadlessRenderer},
    modifier::ModifierCapabilityExt as _,
    remember, scaled_box, tessera,
    testing::text_matches,
    winit::event::Ime,
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    icon::icon,
    material_icons::filled,
    modifier::ModifierExt as _,
    shape_def::Shape,
    text_field::text_field,
    text_input::TextInputController,
    theme::material_theme,
    time_picker::{TimePickerDisplayMode, TimePickerController, time_picker},
};

#[derive(Clone, Copy, Default)]
struct FixtureHandles {
    editor: Option<State<TextInputController>>,
    clipped: Option<State<bool>>,
    time: Option<State<TimePickerController>>,
}

#[derive(Clone, Copy, PartialEq)]
struct FractionalOrigin;

impl tessera_ui::PlacementModifierNode for FractionalOrigin {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        position + PxPosition::from_f32_xy(10.25, 20.5)
    }
}

fn observe(name: &'static str) -> Modifier {
    Modifier::new().semantics(crate::modifier::SemanticsArgs {
        test_tag: Some(name.into()),
        merge_descendants: true,
        ..Default::default()
    })
}

#[tessera]
fn field_fixture(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
    let editor = remember(|| {
        let mut editor = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
        editor.set_text("Input");
        editor
    });
    let clipped = remember(|| false);
    observed.with_mut(|handles| handles.editor = Some(editor));
    observed.with_mut(|handles| handles.clipped = Some(clipped));
    material_theme().child(move || {
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(
                Modifier::new()
                    .fill_max_size()
                    .background(Color::from_rgb(0.21, 0.55, 0.31)),
            )
            .children(move || {
                let mut modifier = Modifier::new().push_placement(FractionalOrigin);
                if clipped.get() {
                    modifier = modifier.clip_shape(Shape::rounded_rectangle(Dp(8.0)));
                }
                boxed()
                    .alignment(Alignment::TopStart)
                    .modifier(modifier)
                    .children(move || {
                        scaled_box(0.625).child(move || {
                            text_field()
                                .outlined()
                                .controller(editor)
                                .label("Label")
                                .accessibility_label("Scaled outlined editor")
                                .modifier(observe("field").width(Dp(240.0)))
                                .leading_icon(|| {
                                    boxed().modifier(observe("icon")).children(|| {
                                        icon().painter(filled::PERSON_SVG).size(Dp(24.0));
                                    });
                                });
                        });
                    });
            });
    });
}

#[tessera]
fn time_fixture(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
    let time = remember(|| TimePickerController::new(10, 8, false, TimePickerDisplayMode::Picker));
    observed.with_mut(|handles| handles.time = Some(time));
    material_theme().child(move || {
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(
                Modifier::new()
                    .fill_max_size()
                    .background(Color::from_rgb(0.21, 0.55, 0.31)),
            )
            .children(move || {
                boxed()
                    .alignment(Alignment::TopStart)
                    .modifier(Modifier::new().push_placement(FractionalOrigin))
                    .children(move || {
                        scaled_box(0.625).child(move || {
                            time_picker().controller(time);
                        });
                    });
            });
    });
}

fn renderer<Output: 'static>(
    entry: fn(tessera_ui::testing::TestProbe<FixtureHandles>) -> Output,
    samples: u32,
    observed: &tessera_ui::testing::TestProbe<FixtureHandles>,
) -> HeadlessRenderer {
    let mut renderer = HeadlessRenderer::new(
        Box::new({
            let observed = observed.clone();
            move || {
                entry(observed.clone());
            }
        }),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 300,
            height: 360,
            sample_count: samples,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    renderer
}

fn coordinates(renderer: &HeadlessRenderer, name: &str) -> LayoutCoordinates {
    renderer
        .inspect_components()
        .find_tag(name)
        .expect("native control")
        .layout_coordinates()
        .expect("control geometry")
}

fn control(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == role && text_matches(node.label(), label))
        .unwrap_or_else(|| panic!("missing native {label} {role:?}"))
}

fn click(renderer: &mut HeadlessRenderer, position: PxPosition) {
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(4);
}

fn pixel(pixels: &[u8], x: usize, y: usize) -> [u8; 4] {
    pixels[(y * 300 + x) * 4..][..4].try_into().unwrap()
}

fn save_pixels(renderer: &HeadlessRenderer, pixels: &[u8], name: &str, samples: u32) {
    if let Some(directory) = std::env::var_os("TESSERA_SCALE_QA_DIR") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        let size = renderer.physical_size();
        tessera_ui::headless::write_png_file(
            directory.join(format!("{name}-msaa{samples}.png")),
            size.width,
            size.height,
            pixels,
        )
        .unwrap();
    }
}

#[test]
#[ignore = "requires a native GPU adapter for scaled outlined field pixels and editing"]
fn scaled_outlined_field_keeps_native_insets_notch_ime_focus_and_ancestor_clipping() {
    crate::test_utils::run_isolated_test(
        "scaled_controls_tests::scaled_outlined_field_keeps_native_insets_notch_ime_focus_and_ancestor_clipping",
        || {
            for samples in [1, 4] {
                let observed = tessera_ui::testing::TestProbe::default();
                let mut renderer = renderer(field_fixture, samples, &observed);
                let field = coordinates(&renderer, "field");
                assert_eq!(field.local_size.width, Px(240));
                assert_eq!(field.local_size.height, Px(56));
                assert_eq!(
                    field.window_bounds(),
                    FloatRect {
                        x: 10.25,
                        y: 20.5,
                        width: 150.0,
                        height: 35.0
                    }
                );
                let icon = coordinates(&renderer, "icon").window_bounds();
                assert_eq!((icon.width, icon.height), (15.0, 15.0));
                assert_eq!(
                    icon.x + icon.width * 0.5 - field.window_bounds().x,
                    15.0,
                    "the native 48px leading slot occupies 30 window pixels"
                );
                let (input, node) = control(&renderer, "Scaled outlined editor", Role::TextInput);
                let editor_bounds = node.bounds().unwrap();
                assert_eq!(editor_bounds.x0 - f64::from(field.window_bounds().x), 32.5);
                assert!(editor_bounds.x1 <= 160.25 && editor_bounds.y1 <= 55.5);
                let pixels = renderer.read_pixels().unwrap();
                save_pixels(&renderer, &pixels, "scaled-field-initial", samples);
                let background = pixel(&pixels, 2, 2);
                let label = control(&renderer, "Label", Role::Label).1.bounds().unwrap();
                assert_eq!(
                    pixel(&pixels, 19, 20),
                    background,
                    "the pixel inside the scaled floating label notch preserves the parent: {label:?}"
                );
                assert_ne!(
                    pixel(&pixels, 130, 21),
                    background,
                    "the scaled top outline remains painted"
                );
                click(
                    &mut renderer,
                    PxPosition::from_f32_xy(
                        (editor_bounds.x1 - 2.0) as f32,
                        ((editor_bounds.y0 + editor_bounds.y1) * 0.5) as f32,
                    ),
                );
                assert_eq!(renderer.accessibility_snapshot().unwrap().focus, input);
                renderer.inject_ime_event(Ime::Commit("!".into()));
                renderer.render_frames(4);
                assert_eq!(
                    observed.get().editor.unwrap().with(|editor| editor.text()),
                    "Input!"
                );
                assert_eq!(
                    control(&renderer, "Scaled outlined editor", Role::TextInput)
                        .1
                        .value(),
                    Some("Input!")
                );
                save_pixels(
                    &renderer,
                    &renderer.read_pixels().unwrap(),
                    "scaled-field-focused",
                    samples,
                );
                focus::clear_focus(true);
                renderer.render_frames(4);
                assert!(
                    !observed
                        .get()
                        .editor
                        .unwrap()
                        .with(|editor| editor.focus_handler().is_focused()),
                    "focus must be cleared before the unclipped hit-test control"
                );
                click(&mut renderer, PxPosition::from_f32_xy(11.25, 21.5));
                assert!(
                    observed
                        .get()
                        .editor
                        .unwrap()
                        .with(|editor| editor.focus_handler().is_focused()),
                    "the same corner reaches the native field before ancestor clipping"
                );
                focus::clear_focus(true);
                observed.get().clipped.unwrap().set(true);
                renderer.render_frames(4);
                assert!(
                    !observed
                        .get()
                        .editor
                        .unwrap()
                        .with(|editor| editor.focus_handler().is_focused()),
                    "focus must be cleared before testing the clipped corner"
                );
                click(&mut renderer, PxPosition::from_f32_xy(11.25, 21.5));
                assert!(
                    !observed
                        .get()
                        .editor
                        .unwrap()
                        .with(|editor| editor.focus_handler().is_focused()),
                    "a point in the rounded ancestor's clipped corner cannot focus its native field"
                );
                assert_eq!(
                    coordinates(&renderer, "field").window_bounds(),
                    field.window_bounds()
                );
            }
        },
    );
}

#[test]
#[ignore = "requires a native GPU adapter for a scaled clock dial and pointer selection"]
fn scaled_time_picker_keeps_dial_targets_and_maps_physical_digit_click_to_state() {
    crate::test_utils::run_isolated_test(
        "scaled_controls_tests::scaled_time_picker_keeps_dial_targets_and_maps_physical_digit_click_to_state",
        || {
            for samples in [1, 4] {
                let observed = tessera_ui::testing::TestProbe::default();
                let mut renderer = renderer(time_fixture, samples, &observed);
                assert_eq!(
                    coordinates(&renderer, "time-picker")
                        .local_to_window
                        .scale(),
                    0.625
                );
                let dial = control(&renderer, "Hours", Role::Group).1.bounds().unwrap();
                assert_eq!((dial.width(), dial.height()), (160.0, 160.0));
                let (digit, node) = control(&renderer, "Hour 3 of 12", Role::Button);
                let bounds = node.bounds().unwrap();
                assert_eq!((bounds.width(), bounds.height()), (30.0, 30.0));
                let before = renderer.read_pixels().unwrap();
                save_pixels(&renderer, &before, "scaled-time-before", samples);
                assert!(renderer.dispatch_accessibility_action(ActionRequest {
                    action: Action::Focus,
                    target_tree: TreeId::ROOT,
                    target_node: digit,
                    data: None,
                }));
                renderer.render_frames(3);
                assert_eq!(
                    observed.get().time.unwrap().with(TimePickerController::hour),
                    10,
                    "accessibility focus must not activate a clock digit"
                );
                click(
                    &mut renderer,
                    PxPosition::from_f32_xy(
                        ((bounds.x0 + bounds.x1) * 0.5) as f32,
                        ((bounds.y0 + bounds.y1) * 0.5) as f32,
                    ),
                );
                assert_eq!(observed.get().time.unwrap().with(TimePickerController::hour), 3);
                assert_eq!(
                    observed.get().time.unwrap().with(TimePickerController::minute),
                    8
                );
                let after = renderer.read_pixels().unwrap();
                save_pixels(&renderer, &after, "scaled-time-after", samples);
                let changed = (dial.y0.ceil() as usize..dial.y1.floor() as usize).any(|y| {
                    (dial.x0.ceil() as usize..dial.x1.floor() as usize)
                        .any(|x| pixel(&before, x, y) != pixel(&after, x, y))
                });
                assert!(
                    changed,
                    "the real clock dial must repaint after the physical selection"
                );
            }
        },
    );
}
