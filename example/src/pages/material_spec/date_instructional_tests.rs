//! Date accessibility figure interaction regressions.
//!
//! ## Usage
//!
//! Exercise editing, selection, clearing, and focus in date figures.

use std::{cell::Cell, time::Duration};

use tessera_components::theme::material_theme;
use tessera_ui::{
    FocusDirection, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType,
    PxPosition,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, TreeId},
};

use super::*;

thread_local! {
    static KIND: Cell<Illustration> = const { Cell::new(Illustration::EntryMethods) };
    pub(super) static ENTRY_DRAFT: Cell<Option<State<String>>> = const { Cell::new(None) };
    pub(super) static ENTRY_CHANGES: Cell<usize> = const { Cell::new(0) };
    pub(super) static CLEAR_STATE: Cell<Option<State<DatePickerState>>> = const { Cell::new(None) };
    pub(super) static CLEAR_COMMITTED: Cell<Option<State<Option<CalendarDate>>>> = const { Cell::new(None) };
    pub(super) static GEOMETRY: Cell<Option<State<PreciseGeometry>>> = const { Cell::new(None) };
}

struct Pipelines;
impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn fixture() {
    super::super::super::media::load_fonts();
    material_theme().child(|| {
        illustration(KIND.get());
    });
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn responsive_formatting_figure_keeps_its_full_canvas_and_clickable_calendar() {
    KIND.set(Illustration::Formatting);
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 812,
            height: 800,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    let canvas = node(&renderer, "Deferred date formatting diagram", Role::Group)
        .1
        .bounds()
        .unwrap();
    assert!(
        (canvas.width() - 812.0).abs() < 0.001,
        "The native artboard must fill its actual viewport: {canvas:?}"
    );
    let calendar = node(&renderer, "Open Unformatted date calendar", Role::Button)
        .1
        .bounds()
        .unwrap();
    assert!(
        calendar.x0 >= canvas.x0 && calendar.x1 <= canvas.x1,
        "The calendar must remain inside its painted and interactive canvas: {calendar:?}, {canvas:?}"
    );
    input_article_target(
        &mut renderer,
        "Open Unformatted date calendar",
        Some(Role::Button),
        Some("\n"),
    );
    renderer.render_frames(8);
    assert!(
        exists(&renderer, "Optional date selection", Role::Dialog),
        "Opening the real scaled calendar must survive a same-batch Enter"
    );
}

fn renderer(kind: Illustration) -> HeadlessRenderer {
    KIND.set(kind);
    let width = if kind == Illustration::EntryMethods {
        1353
    } else {
        1352
    };
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width,
            height: 800,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(6);
    let figure = field_bounds("figure");
    assert_eq!((figure.width, figure.height), (width as f32, 600.0));
    if matches!(kind, Illustration::EntryMethods | Illustration::Labels) {
        let pixels = renderer.read_pixels().unwrap();
        let pixel = |x: usize, y: usize| {
            let i = (y * width as usize + x) * 4;
            &pixels[i..i + 4]
        };
        let background = pixel(100, 20);
        let (strokes, notch, clear_above) = if kind == Illustration::EntryMethods {
            ([(346, 233), (928, 233)], (380, 233), (380, 231))
        } else {
            ([(65, 180), (960, 127)], (880, 127), (880, 123))
        };
        let notch_background = pixel(clear_above.0, clear_above.1);
        for (x, y) in strokes {
            assert!(
                (0..3).any(
                    |channel| (pixel(x, y)[channel] as i16 - background[channel] as i16).abs() > 64
                ),
                "Reference-centered annotation stroke must paint at ({x}, {y}), actual {:?}, background {background:?}",
                pixel(x, y),
            );
        }
        assert!(
            (0..4).all(|channel| (pixel(notch.0, notch.1)[channel] as i16
                - notch_background[channel] as i16)
                .abs()
                <= 2),
            "The annotation must leave the floating-label notch clear, actual {:?}, background {notch_background:?}",
            pixel(notch.0, notch.1),
        );
    }
    if let Some(directory) = std::env::var_os("TESSERA_MENU_QA_DIR") {
        let path = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&path).unwrap();
        let name = match kind {
            Illustration::EntryMethods => "date-access-entry-native.png",
            Illustration::Formatting => "date-access-formatting-native.png",
            Illustration::Clear => "date-access-clear-native.png",
            Illustration::Labels => "date-access-labels-native.png",
        };
        std::fs::write(
            path.join(name.replace(".png", ".json")),
            renderer.snapshot().to_string(),
        )
        .unwrap();
        std::fs::write(
            path.join(name.replace(".png", "-published.txt")),
            format!("{:#?}", renderer.accessibility_snapshot()),
        )
        .unwrap();
        let pixels = renderer.read_pixels().unwrap();
        tessera_ui::headless::write_png_file(
            path.join(name),
            width,
            600,
            &pixels[..width as usize * 600 * 4],
        )
        .unwrap();
    }
    renderer
}

fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label) && node.role() == role)
        .unwrap_or_else(|| panic!("missing native {label} {role:?}"))
}

fn exists(renderer: &HeadlessRenderer, label: &str, role: Role) -> bool {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .iter()
        .any(|(_, node)| node.label() == Some(label) && node.role() == role)
}

fn act(renderer: &mut HeadlessRenderer, label: &str, role: Role, action: Action) {
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: node(renderer, label, role).0,
        data: None,
    }));
    renderer.render_frames(6);
}

fn click(renderer: &mut HeadlessRenderer, label: &str, role: Role) {
    let bounds = node(renderer, label, role).1.bounds().unwrap();
    let pointer = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, pointer);
    renderer.pointer_press(MOUSE_POINTER_ID, pointer, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, pointer, PressKeyEventType::Left);
    renderer.render_frames(8);
}

fn settle_background(renderer: &mut HeadlessRenderer, label: &str) {
    for _ in 0..90 {
        let tree = renderer.accessibility_snapshot().unwrap();
        if !tree
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Dialog)
            && exists(renderer, label, Role::TextInput)
        {
            return;
        }
        renderer.render_frames(1);
    }
    assert!(
        exists(renderer, label, Role::TextInput),
        "the real dialog exit must restore its editable background"
    );
    assert!(
        !renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Dialog)
    );
}

fn value(renderer: &HeadlessRenderer, label: &str) -> String {
    node(renderer, label, Role::TextInput)
        .1
        .value()
        .unwrap()
        .to_owned()
}

fn field_bounds(name: &str) -> tessera_ui::FloatRect {
    GEOMETRY
        .get()
        .unwrap()
        .with(|geometry| geometry[name].window_bounds())
}

fn assert_native_field(name: &str, x: f32, y: f32) {
    let bounds = field_bounds(name);
    assert_eq!(
        (bounds.x, bounds.y, bounds.width, bounds.height),
        (x, y, 684.0, 112.0)
    );
}

#[test]
#[ignore = "requires a native GPU adapter for date entry illustrations"]
fn illustrated_entry_keeps_separate_native_stops_and_confirms_calendar_selection() {
    let mut renderer = renderer(Illustration::EntryMethods);
    assert_native_field("field", 334.0, 244.0);
    assert_eq!(value(&renderer, "Entry methods date"), "08/17/2023");
    let editor = node(&renderer, "Entry methods date", Role::TextInput).0;
    let toggle = node(&renderer, "Open Entry methods date calendar", Role::Button).0;
    assert_ne!(editor, toggle);
    act(
        &mut renderer,
        "Entry methods date",
        Role::TextInput,
        Action::Focus,
    );
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, editor);
    FocusManager::current().request_move_focus(FocusDirection::Next);
    renderer.render_frames(4);
    assert_eq!(
        renderer.accessibility_snapshot().unwrap().focus,
        toggle,
        "the field focus group must not add a Tab stop between editor and calendar icon"
    );
    click(
        &mut renderer,
        "Open Entry methods date calendar",
        Role::Button,
    );
    assert!(exists(&renderer, "Optional date selection", Role::Dialog));
    click(&mut renderer, "August 18, 2023", Role::Button);
    click(
        &mut renderer,
        "Confirm optional date selection",
        Role::Button,
    );
    settle_background(&mut renderer, "Entry methods date");
    assert_eq!(value(&renderer, "Entry methods date"), "08/18/2023");
    assert_native_field("field", 334.0, 244.0);
    renderer.resize(676, 800);
    renderer.render_frames(6);
    let bounds = field_bounds("field");
    assert!((bounds.width - 684.0 * 676.0 / 1353.0).abs() < 0.01);
    click(
        &mut renderer,
        "Open Entry methods date calendar",
        Role::Button,
    );
    assert!(exists(&renderer, "Optional date selection", Role::Dialog));
    click(
        &mut renderer,
        "Cancel optional date selection",
        Role::Button,
    );
    settle_background(&mut renderer, "Entry methods date");
    assert_eq!(value(&renderer, "Entry methods date"), "08/18/2023");
}

#[test]
#[ignore = "requires a native GPU adapter for deferred date formatting"]
fn illustrated_date_input_formats_on_submission_or_focus_exit_and_preserves_invalid_edits() {
    let mut renderer = renderer(Illustration::Formatting);
    assert_native_field("raw", 334.0, 124.0);
    assert_native_field("formatted", 334.0, 344.0);
    assert_eq!(value(&renderer, "Unformatted date"), "08172023");
    assert_eq!(value(&renderer, "Formatted date"), "08/17/2023");
    act(
        &mut renderer,
        "Unformatted date",
        Role::TextInput,
        Action::Focus,
    );
    renderer.inject_text("\n");
    renderer.render_frames(6);
    assert_eq!(value(&renderer, "Unformatted date"), "08/17/2023");
    assert_eq!(value(&renderer, "Formatted date"), "08/17/2023");
    renderer.inject_text("9");
    renderer.render_frames(6);
    let invalid = value(&renderer, "Unformatted date");
    assert_ne!(invalid, "08/17/2023");
    assert_eq!(value(&renderer, "Formatted date"), "08/17/2023");
    renderer.inject_text("\n");
    renderer.render_frames(6);
    assert_eq!(value(&renderer, "Unformatted date"), invalid);
    assert_eq!(value(&renderer, "Formatted date"), "08/17/2023");
    assert_native_field("raw", 334.0, 124.0);
    assert_native_field("formatted", 334.0, 344.0);
    assert_eq!(
        node(&renderer, "Unformatted date", Role::TextInput)
            .1
            .description(),
        Some("Enter a valid date (MM/DD/YYYY)")
    );
    drop(renderer);
    let mut renderer = self::renderer(Illustration::Formatting);
    act(
        &mut renderer,
        "Unformatted date",
        Role::TextInput,
        Action::Focus,
    );
    assert_eq!(value(&renderer, "Unformatted date"), "08172023");
    act(
        &mut renderer,
        "Formatted date",
        Role::TextInput,
        Action::Focus,
    );
    assert_eq!(value(&renderer, "Unformatted date"), "08/17/2023");
}

#[test]
#[ignore = "requires a native GPU adapter for calendar clear actions"]
fn illustrated_clear_uses_native_day_selection_and_only_visible_focus_targets() {
    let mut renderer = renderer(Illustration::Clear);
    assert_eq!(
        node(&renderer, "August 17, 2020", Role::Button)
            .1
            .is_selected(),
        Some(true)
    );
    for (_, node) in renderer.accessibility_snapshot().unwrap().nodes {
        if node.role() == Role::Button {
            let bounds = node.bounds().unwrap();
            assert!(bounds.width() > 0.0 && bounds.height() > 0.0);
            assert!(bounds.x0 >= 316.0 && bounds.x1 <= 1036.0);
            assert!(bounds.y0 >= 0.0 && bounds.y1 <= 504.0);
            assert!(
                !["Previous month", "Next month", "Choose year"]
                    .contains(&node.label().unwrap_or_default())
            );
        }
    }
    assert!(!exists(&renderer, "August 8, 2020", Role::Button));
    click(&mut renderer, "Optional clear Clear", Role::Button);
    assert_eq!(
        node(&renderer, "August 17, 2020", Role::Button)
            .1
            .is_selected(),
        Some(false)
    );
    click(&mut renderer, "Optional clear Cancel", Role::Button);
    assert_eq!(
        node(&renderer, "August 17, 2020", Role::Button)
            .1
            .is_selected(),
        Some(true)
    );
    click(&mut renderer, "Optional clear Clear", Role::Button);
    click(&mut renderer, "Optional clear OK", Role::Button);
    click(&mut renderer, "August 18, 2020", Role::Button);
    assert_eq!(
        node(&renderer, "August 18, 2020", Role::Button)
            .1
            .is_selected(),
        Some(true)
    );
    click(&mut renderer, "Optional clear Cancel", Role::Button);
    assert_eq!(
        node(&renderer, "August 18, 2020", Role::Button)
            .1
            .is_selected(),
        Some(false)
    );
    act(&mut renderer, "August 9, 2020", Role::Button, Action::Focus);
    for _ in 0..26 {
        FocusManager::current().request_move_focus(FocusDirection::Next);
        renderer.render_frames(2);
        let tree = renderer.accessibility_snapshot().unwrap();
        let focus = tree.focus;
        let bounds = tree
            .nodes
            .iter()
            .find(|(id, _)| *id == focus)
            .unwrap()
            .1
            .bounds()
            .unwrap();
        assert!(bounds.width() > 0.0 && bounds.height() > 0.0);
        assert!(bounds.y0 >= 0.0 && bounds.y1 <= 504.0);
    }
    renderer.resize(676, 500);
    renderer.render_frames(6);
    click(&mut renderer, "August 31, 2020", Role::Button);
    assert_eq!(
        node(&renderer, "August 31, 2020", Role::Button)
            .1
            .is_selected(),
        Some(true)
    );
    click(&mut renderer, "Optional clear Clear", Role::Button);
    assert_eq!(
        node(&renderer, "August 31, 2020", Role::Button)
            .1
            .is_selected(),
        Some(false)
    );
}

#[test]
#[ignore = "requires a native GPU adapter for accessible date labeling"]
fn labeled_date_composite_keeps_editing_and_native_month_chooser_functional() {
    let mut renderer = renderer(Illustration::Labels);
    assert_native_field("field", 790.0, 140.0);
    assert_eq!(value(&renderer, "Event date"), "2023/08/17");
    let composite = node(&renderer, "Event date, YYYY/MM/DD", Role::ComboBox).1;
    assert_eq!(composite.is_expanded(), Some(false));
    assert_eq!(composite.has_popup(), Some(HasPopup::Dialog));
    assert_eq!(
        node(&renderer, "Event date", Role::TextInput)
            .1
            .description(),
        Some("YYYY/MM/DD")
    );
    click(&mut renderer, "Previous labeled month", Role::Button);
    node(&renderer, "Jul", Role::Label);
    click(&mut renderer, "Choose labeled month", Role::Button);
    assert!(exists(
        &renderer,
        "Labeled date calendar chooser",
        Role::Dialog
    ));
    assert!(exists(&renderer, "Choose month", Role::ListBox));
    click(&mut renderer, "Choose month", Role::Button);
    click(&mut renderer, "Cancel", Role::Button);
    settle_background(&mut renderer, "Event date");
    assert_eq!(value(&renderer, "Event date"), "2023/08/17");
    act(
        &mut renderer,
        "Event date, YYYY/MM/DD",
        Role::ComboBox,
        Action::Click,
    );
    assert!(exists(
        &renderer,
        "Labeled date calendar chooser",
        Role::Dialog
    ));
    click(&mut renderer, "Cancel", Role::Button);
    settle_background(&mut renderer, "Event date");
    assert_eq!(
        node(&renderer, "Event date, YYYY/MM/DD", Role::ComboBox)
            .1
            .is_expanded(),
        Some(false)
    );
    assert!(exists(&renderer, "Previous labeled month", Role::Button));
    act(&mut renderer, "Event date", Role::TextInput, Action::Focus);
    renderer.inject_text("9");
    renderer.render_frames(4);
    let invalid = value(&renderer, "Event date");
    assert_ne!(invalid, "2023/08/17");
    renderer.inject_text("\n");
    renderer.render_frames(4);
    assert_eq!(value(&renderer, "Event date"), invalid);
}

fn article_target(renderer: &HeadlessRenderer, label: &str, role: Option<Role>) -> Node {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| {
            node.label() == Some(label)
                && role.is_none_or(|role| node.role() == role)
                && !node.is_hidden()
                && node.bounds().is_some()
        })
        .unwrap_or_else(|| {
            let published = renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .into_iter()
                .filter_map(|(_, node)| {
                    node.label()
                        .map(|label| (label.to_owned(), node.role(), node.bounds()))
                })
                .collect::<Vec<_>>();
            panic!("missing real article target {label} {role:?}: {published:?}")
        })
        .1
}

fn click_article_target(renderer: &mut HeadlessRenderer, label: &str, role: Option<Role>) {
    input_article_target(renderer, label, role, None);
}

fn reveal_article_target(renderer: &mut HeadlessRenderer, label: &str, role: Option<Role>) -> Node {
    let viewport = renderer.physical_size();
    for _ in 0..24 {
        let target = renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| {
                node.label() == Some(label)
                    && role.is_none_or(|role| node.role() == role)
                    && !node.is_hidden()
                    && node.bounds().is_some()
            });
        let Some((_, target)) = target else {
            renderer.pointer_scroll(
                MOUSE_POINTER_ID,
                PxPosition::from_f32_xy(viewport.width as f32 - 30.0, viewport.height as f32 * 0.5),
                0.0,
                if matches!(role, Some(Role::Link | Role::Tab)) {
                    1200.0
                } else {
                    -1200.0
                },
            );
            renderer.render_frames(40);
            continue;
        };
        let bounds = target.bounds().unwrap();
        let center = (bounds.y0 + bounds.y1) * 0.5;
        if center > 80.0 && center < f64::from(viewport.height) - 24.0 {
            return target;
        }
        renderer.pointer_scroll(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(viewport.width as f32 - 30.0, viewport.height as f32 * 0.5),
            0.0,
            (f64::from(viewport.height) * 0.5 - center).clamp(-1200.0, 1200.0) as f32,
        );
        renderer.render_frames(20);
    }
    panic!("real scroll did not reveal article target {label}");
}

fn input_article_target(
    renderer: &mut HeadlessRenderer,
    label: &str,
    role: Option<Role>,
    queued_ime: Option<&str>,
) {
    let target = reveal_article_target(renderer, label, role);
    let bounds = target.bounds().unwrap();
    let pointer = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, pointer);
    renderer.pointer_press(MOUSE_POINTER_ID, pointer, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, pointer, PressKeyEventType::Left);
    if let Some(text) = queued_ime {
        renderer.inject_text(text);
    }
    renderer.render_frames(8);
}

fn assert_pointer_editor_focus(renderer: &HeadlessRenderer, editor: &str, context: &str) {
    let tree = renderer.accessibility_snapshot().unwrap();
    let id = node(renderer, editor, Role::TextInput).0;
    assert_eq!(
        tree.focus,
        id,
        "{context}: physical pointer must focus the actual editor {editor}; focused node: {:?}",
        tree.nodes
            .iter()
            .find(|(candidate, _)| *candidate == tree.focus)
    );
}

fn select_editor_line_with_pointer(renderer: &mut HeadlessRenderer, editor: &str) {
    for _ in 0..3 {
        input_article_target(renderer, editor, Some(Role::TextInput), None);
        assert_pointer_editor_focus(renderer, editor, "native triple-click selection");
    }
}

fn probe_article_editor_pointer_and_queued_commits(renderer: &mut HeadlessRenderer) {
    let editor = "Entry methods date";
    input_article_target(renderer, editor, Some(Role::TextInput), None);
    assert_pointer_editor_focus(renderer, editor, "published text bounds center");
    click_article_target(renderer, "Date entry methods", Some(Role::Link));
    let figure = article_target(renderer, "Date entry methods diagram", Some(Role::Group))
        .bounds()
        .unwrap();
    let blank = PxPosition::from_f32_xy(
        (figure.x0 + figure.width() * 800.0 / 1353.0) as f32,
        (figure.y0 + figure.height() * 300.0 / 600.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, blank);
    renderer.pointer_press(MOUSE_POINTER_ID, blank, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, blank, PressKeyEventType::Left);
    renderer.render_frames(8);
    assert_pointer_editor_focus(
        renderer,
        editor,
        "blank body inside the visible outlined field",
    );
    select_editor_line_with_pointer(renderer, editor);
    let changes_before = ENTRY_CHANGES.get();
    let mut prefix = String::new();
    for (index, character) in "08/18/2023".chars().enumerate() {
        renderer.inject_text(&character.to_string());
        renderer.render_frames(1);
        prefix.push(character);
        assert_eq!(
            ENTRY_DRAFT.get().unwrap().get(),
            prefix,
            "the actual edit callback must commit every character during its input frame"
        );
        assert_eq!(ENTRY_CHANGES.get(), changes_before + index + 1);
        assert_pointer_editor_focus(renderer, editor, "during each individual IME commit");
        renderer.render_frames(1);
        assert_eq!(
            value(renderer, editor),
            prefix,
            "the next publication frame must expose the actual edited controller value"
        );
        assert_pointer_editor_focus(renderer, editor, "after each individual IME commit");
    }
    select_editor_line_with_pointer(renderer, editor);
    let changes_before = ENTRY_CHANGES.get();
    for character in "08/19/2023".chars() {
        renderer.inject_text(&character.to_string());
    }
    renderer.render_frames(1);
    assert_eq!(ENTRY_DRAFT.get().unwrap().get(), "08/19/2023");
    assert_eq!(ENTRY_CHANGES.get(), changes_before + 10);
    renderer.render_frames(1);
    assert_eq!(
        value(renderer, editor),
        "08/19/2023",
        "all individual character commits queued before one frame must be retained in order"
    );
    assert_pointer_editor_focus(
        renderer,
        editor,
        "after one-frame queued individual characters",
    );
    select_editor_line_with_pointer(renderer, editor);
    let changes_before = ENTRY_CHANGES.get();
    for chunk in ["08", "/17", "/20", "23"] {
        renderer.inject_text(chunk);
    }
    renderer.render_frames(1);
    assert_eq!(ENTRY_DRAFT.get().unwrap().get(), "08/17/2023");
    assert_eq!(ENTRY_CHANGES.get(), changes_before + 4);
    renderer.render_frames(1);
    assert_eq!(
        value(renderer, editor),
        "08/17/2023",
        "all queued IME chunks must update the same editor in order within one frame"
    );
    assert_pointer_editor_focus(renderer, editor, "after one-frame queued IME chunks");
}

fn assert_article_modal(
    renderer: &mut HeadlessRenderer,
    label: &str,
    action: &str,
    before: &[u8],
    screenshot: &str,
) {
    renderer.render_frames(30);
    let viewport = renderer.physical_size();
    let tree = renderer.accessibility_snapshot().unwrap();
    let (dialog_id, dialog_node) = tree
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Dialog && node.label() == Some(label))
        .unwrap_or_else(|| {
            if let Some(directory) = std::env::var_os("TESSERA_MENU_QA_DIR") {
                std::fs::write(
                    std::path::PathBuf::from(&directory).join(format!("missing-{screenshot}.json")),
                    renderer.snapshot().to_string(),
                )
                .unwrap();
                tessera_ui::headless::write_png_file(
                    std::path::PathBuf::from(directory).join(format!("missing-{screenshot}")),
                    renderer.physical_size().width,
                    renderer.physical_size().height,
                    &renderer.read_pixels().unwrap(),
                )
                .unwrap();
            }
            let published = tree
                .nodes
                .iter()
                .filter_map(|(_, node)| {
                    node.label()
                        .map(|label| (label, node.role(), node.bounds()))
                })
                .collect::<Vec<_>>();
            panic!("actual hosted dialog missing for {screenshot}; published={published:?}");
        });
    let mut descendants = vec![*dialog_id];
    let mut focused_inside = false;
    while let Some(id) = descendants.pop() {
        if id == tree.focus {
            focused_inside = true;
            break;
        }
        descendants.extend_from_slice(
            tree.nodes
                .iter()
                .find(|(node_id, _)| *node_id == id)
                .expect("published dialog descendant")
                .1
                .children(),
        );
    }
    assert!(
        focused_inside,
        "opening with a same-batch queued IME commit must keep actual focus inside the modal"
    );
    let dialog = dialog_node.bounds().unwrap();
    if label == "Optional date selection" {
        assert_eq!(
            (dialog.width(), dialog.height()),
            (360.0, 572.0),
            "the native date dialog must retain its size while providing full action targets"
        );
    }
    assert!(
        dialog.x0 >= 0.0
            && dialog.y0 >= 0.0
            && dialog.x1 <= f64::from(viewport.width)
            && dialog.y1 <= f64::from(viewport.height),
        "article modal must fit the window after scrolling: {dialog:?}, viewport{viewport:?}"
    );
    let action = article_target(renderer, action, Some(Role::Button));
    let action_bounds = action.bounds().unwrap();
    assert!(
        action_bounds.x0 >= dialog.x0
            && action_bounds.x1 <= dialog.x1
            && action_bounds.y0 >= dialog.y0
            && action_bounds.y1 <= dialog.y1,
        "the actual modal action must remain visible: {action_bounds:?} inside{dialog:?}"
    );
    assert!(
        action_bounds.width() >= 48.0 && action_bounds.height() >= 48.0,
        "actual modal action needs a native 48px target: {action_bounds:?} inside {dialog:?}"
    );
    let pixels = renderer.read_pixels().unwrap();
    let outside_figure = (200 * viewport.width as usize + 4) * 4;
    assert!(
        (0..3).any(|channel| before[outside_figure + channel] as i16
            - pixels[outside_figure + channel] as i16
            > 24),
        "the native scrim must cover article pixels outside the instructional figure"
    );
    assert!(!exists(renderer, "Entry methods date", Role::TextInput));
    assert!(!exists(renderer, "Event date", Role::TextInput));
    if let Some(directory) = std::env::var_os("TESSERA_MENU_QA_DIR") {
        tessera_ui::headless::write_png_file(
            std::path::PathBuf::from(directory).join(screenshot),
            viewport.width,
            viewport.height,
            &pixels,
        )
        .unwrap();
    }
}

#[test]
#[ignore = "requires a native GPU adapter for actual app article modal routing"]
fn real_date_article_routes_instructional_dialogs_above_scrolled_content_and_keeps_actions_visible()
{
    let mut renderer = HeadlessRenderer::new(
        Box::new(crate::app::app),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 907,
            height: 871,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    click_article_target(&mut renderer, "Material Components", None);
    for _ in 0..90 {
        let tree = renderer.accessibility_snapshot().unwrap();
        if tree.nodes.iter().any(|(_, node)| {
            node.label() == Some("Open Date pickers specifications") && !node.is_hidden()
        }) && !tree
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Dialog)
        {
            break;
        }
        renderer.render_frames(1);
    }
    assert!(
        !renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Dialog),
        "the real navigation drawer must finish its modal exit before interacting with the directory"
    );
    click_article_target(&mut renderer, "Open Date pickers specifications", None);
    click_article_target(&mut renderer, "Accessibility", Some(Role::Tab));
    assert_eq!(
        article_target(&renderer, "Accessibility", Some(Role::Tab)).is_selected(),
        Some(true)
    );
    for (section, editor, icon, screenshot) in [
        (
            "Date entry methods",
            "Entry methods date",
            "Open Entry methods date calendar",
            "date-article-entry-modal.png",
        ),
        (
            "Accessible date input",
            "Unformatted date",
            "Open Unformatted date calendar",
            "date-article-formatting-modal.png",
        ),
    ] {
        click_article_target(&mut renderer, section, Some(Role::Link));
        if editor == "Entry methods date" {
            probe_article_editor_pointer_and_queued_commits(&mut renderer);
        }
        let original = value(&renderer, editor);
        assert_eq!(
            original,
            if editor == "Unformatted date" {
                "08172023"
            } else {
                "08/17/2023"
            },
            "the actual route must start with its authored date entry example"
        );
        let before = renderer.read_pixels().unwrap();
        if editor == "Unformatted date" {
            let canvas = article_target(
                &renderer,
                "Deferred date formatting diagram",
                Some(Role::Group),
            )
            .bounds()
            .unwrap();
            assert!(
                (canvas.width() - 812.0).abs() < 0.001,
                "The cold formatting canvas must preserve canonical width before input: {canvas:?}"
            );
        }
        input_article_target(&mut renderer, icon, Some(Role::Button), Some("\n"));
        assert_article_modal(
            &mut renderer,
            "Optional date selection",
            "Confirm optional date selection",
            &before,
            screenshot,
        );
        click(
            &mut renderer,
            "Cancel optional date selection",
            Role::Button,
        );
        settle_background(&mut renderer, editor);
        assert_eq!(
            value(&renderer, editor),
            "08/17/2023",
            "calendar focus exit may format the valid date but must preserve August 17 after Cancel; initial value: {original}"
        );
        click_article_target(&mut renderer, icon, Some(Role::Button));
        assert_article_modal(
            &mut renderer,
            "Optional date selection",
            "Confirm optional date selection",
            &before,
            screenshot,
        );
        let selected = node(&renderer, "August 17, 2023", Role::Button).1;
        assert_eq!(selected.is_selected(), Some(true));
        click(&mut renderer, "August 18, 2023", Role::Button);
        click(
            &mut renderer,
            "Confirm optional date selection",
            Role::Button,
        );
        settle_background(&mut renderer, editor);
        assert_eq!(value(&renderer, editor), "08/18/2023");
        assert_eq!(
            reveal_article_target(&mut renderer, "Accessibility", Some(Role::Tab)).is_selected(),
            Some(true)
        );
    }
    click_article_target(&mut renderer, "Labeling elements", Some(Role::Link));
    let original = value(&renderer, "Event date");
    let before = renderer.read_pixels().unwrap();
    click_article_target(&mut renderer, "Choose labeled month", Some(Role::Button));
    click(&mut renderer, "Choose month", Role::Button);
    assert_article_modal(
        &mut renderer,
        "Labeled date calendar chooser",
        "Cancel",
        &before,
        "date-article-labels-modal.png",
    );
    let wide = node(&renderer, "Labeled date calendar chooser", Role::Dialog)
        .1
        .bounds()
        .unwrap();
    renderer.resize(520, 871);
    renderer.render_frames(8);
    let narrow = node(&renderer, "Labeled date calendar chooser", Role::Dialog)
        .1
        .bounds()
        .unwrap();
    assert_eq!(
        narrow.width(),
        wide.width(),
        "native dialog must not inherit illustration scale"
    );
    assert!(narrow.x0 >= 0.0 && narrow.x1 <= 520.0 && narrow.y0 >= 0.0 && narrow.y1 <= 871.0);
    let cancel = node(&renderer, "Cancel", Role::Button).1.bounds().unwrap();
    assert!(
        cancel.x0 >= narrow.x0
            && cancel.x1 <= narrow.x1
            && cancel.y0 >= narrow.y0
            && cancel.y1 <= narrow.y1
    );
    click(&mut renderer, "Cancel", Role::Button);
    settle_background(&mut renderer, "Event date");
    assert_eq!(value(&renderer, "Event date"), original);
}
