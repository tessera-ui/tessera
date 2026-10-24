//! Native controls rendered and operated through a fractional scale boundary.
//!
//! ## Usage
//!
//! Verify native control rendering and interaction after uniform scaling.

use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    time::Duration,
};

use tessera_ui::{
    Color, Dp, FloatRect, FocusManager, FocusModifierExt as _, HeadlessConfig, HeadlessRenderer,
    LayoutCoordinates, MOUSE_POINTER_ID, Modifier, PressKeyEventType, Px, PxPosition, State,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, TreeId},
    modifier::ModifierCapabilityExt as _,
    remember, scaled_box, tessera,
    winit::event::Ime,
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    icon::icon,
    material_icons::filled,
    modifier::ModifierExt as _,
    shape_def::Shape,
    text_field::TextFieldBuilder,
    text_input::TextInputController,
    theme::material_theme,
    time_picker::{TimePickerDisplayMode, TimePickerState, time_picker},
};

thread_local! {
    static COORDINATES: RefCell<BTreeMap<&'static str, LayoutCoordinates>> = const { RefCell::new(BTreeMap::new()) };
    static EDITOR: Cell<Option<State<TextInputController>>> = const { Cell::new(None) };
    static CLIPPED: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static TIME: Cell<Option<State<TimePickerState>>> = const { Cell::new(None) };
}

#[derive(Clone, Copy, PartialEq)]
struct FractionalOrigin;

impl tessera_ui::PlacementModifierNode for FractionalOrigin {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        position + PxPosition::from_f32_xy(10.25, 20.5)
    }
}

fn observe(name: &'static str) -> Modifier {
    Modifier::new().on_layout_coordinates(move |coordinates: LayoutCoordinates| {
        COORDINATES.with_borrow_mut(|values| {
            values.insert(name, coordinates);
        });
    })
}

#[tessera]
fn field_fixture() {
    let editor = remember(|| {
        let mut editor = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
        editor.set_text("Input");
        editor
    });
    let clipped = remember(|| false);
    EDITOR.set(Some(editor));
    CLIPPED.set(Some(clipped));
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
                            TextFieldBuilder::outlined()
                                .controller(editor)
                                .label("Label")
                                .accessibility_label("Scaled outlined editor")
                                .modifier(observe("field").width(Dp(240.0)))
                                .leading_icon(|| {
                                    boxed().modifier(observe("icon")).children(|| {
                                        icon().vector(filled::PERSON_SVG).size(Dp(24.0));
                                    });
                                });
                        });
                    });
            });
    });
}

#[tessera]
fn time_fixture() {
    let time = remember(|| TimePickerState::new(10, 8, false, TimePickerDisplayMode::Picker));
    TIME.set(Some(time));
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
                            time_picker().state(time).modifier(observe("picker"));
                        });
                    });
            });
    });
}

fn renderer(entry: fn(), samples: u32) -> HeadlessRenderer {
    COORDINATES.with_borrow_mut(BTreeMap::clear);
    let mut renderer = HeadlessRenderer::new(
        Box::new(entry),
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

fn coordinates(name: &str) -> LayoutCoordinates {
    COORDINATES.with_borrow(|values| *values.get(name).expect("observed native control"))
}

fn control(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
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
        tessera_ui::write_png_file(
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
                let mut renderer = renderer(field_fixture, samples);
                let field = coordinates("field");
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
                let icon = coordinates("icon").window_bounds();
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
                assert_eq!(EDITOR.get().unwrap().with(|editor| editor.text()), "Input!");
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
                FocusManager::current().clear_focus(true);
                renderer.render_frames(4);
                assert!(
                    !EDITOR
                        .get()
                        .unwrap()
                        .with(|editor| editor.focus_handler().is_focused()),
                    "focus must be cleared before the unclipped hit-test control"
                );
                click(&mut renderer, PxPosition::from_f32_xy(11.25, 21.5));
                assert!(
                    EDITOR
                        .get()
                        .unwrap()
                        .with(|editor| editor.focus_handler().is_focused()),
                    "the same corner reaches the native field before ancestor clipping"
                );
                FocusManager::current().clear_focus(true);
                CLIPPED.get().unwrap().set(true);
                renderer.render_frames(4);
                assert!(
                    !EDITOR
                        .get()
                        .unwrap()
                        .with(|editor| editor.focus_handler().is_focused()),
                    "focus must be cleared before testing the clipped corner"
                );
                click(&mut renderer, PxPosition::from_f32_xy(11.25, 21.5));
                assert!(
                    !EDITOR
                        .get()
                        .unwrap()
                        .with(|editor| editor.focus_handler().is_focused()),
                    "a point in the rounded ancestor's clipped corner cannot focus its native field"
                );
                assert_eq!(coordinates("field").window_bounds(), field.window_bounds());
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
                let mut renderer = renderer(time_fixture, samples);
                assert_eq!(coordinates("picker").local_to_window.scale(), 0.625);
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
                    TIME.get().unwrap().with(TimePickerState::hour),
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
                assert_eq!(TIME.get().unwrap().with(TimePickerState::hour), 3);
                assert_eq!(TIME.get().unwrap().with(TimePickerState::minute), 8);
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
