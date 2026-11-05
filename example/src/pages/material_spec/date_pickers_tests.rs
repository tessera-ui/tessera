use std::time::Duration;

use tessera_components::{
    boxed::boxed, column::column, modifier::ModifierExt as _, theme::material_theme,
};
use tessera_ui::{
    Dp, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier, PlacementModifierNode,
    PressKeyEventType, PxPosition, State,
    accesskit::{Action, ActionRequest, Node, Role, TreeId},
    modifier::ModifierCapabilityExt as _,
    remember, tessera,
};

#[tessera]
fn fixture() {
    material_theme().child(|| {
        super::date_figure(super::Figure::Docked, true, false);
    });
}

#[tessera]
fn modal_example_fixture(
    output: tessera_ui::testing::TestProbe<Option<(super::Sample, State<bool>)>>,
) {
    let source = super::Sample {
        single: remember(|| super::single(super::DatePickerDisplayMode::Input)),
        range: remember(|| super::range(super::DatePickerDisplayMode::Input)),
        range_mode: remember(|| false),
        dark: remember(|| false),
        status: remember(String::new),
    };
    let draft = super::Sample {
        single: remember(|| super::single(super::DatePickerDisplayMode::Picker)),
        range: remember(|| super::range(super::DatePickerDisplayMode::Picker)),
        range_mode: remember(|| false),
        dark: remember(|| false),
        status: remember(String::new),
    };
    let open = remember(|| false);
    let input = remember(|| true);
    output.set(Some((source, open)));
    material_theme().child(move || {
        tessera_components::dialog::dialog_provider()
            .is_open(open.get())
            .padding(Dp::ZERO)
            .on_close_request(move || open.set(false))
            .main_content(move || {
                column().children(move || {
                    super::example(source, "Input", open, input, true, draft);
                });
            })
            .dialog_content(move |dialog_scope| {
                super::themed_picker(draft, "Modal")
                    .dialog(dialog_scope)
                    .modal(open)
                    .commit_target(source);
            });
    });
}

fn choose_modal_day(renderer: &mut HeadlessRenderer, day: &str) {
    activate(renderer, "Switch to calendar");
    activate(renderer, day);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn date_dialog_example_commits_only_confirmed_selection() {
    let output = tessera_ui::testing::TestProbe::default();
    let observed = output.clone();
    let mut renderer = reference_renderer(Scene::Modal(observed));
    renderer.render_frames(4);
    let (source, open) = output.get().expect("mounted date dialog example");
    activate(&mut renderer, "Open Input date dialog");
    choose_modal_day(&mut renderer, "July 20, 2026");
    activate(&mut renderer, "Modal cancel date");
    assert!(!open.get());
    assert_eq!(
        source.single.with(super::DatePickerState::selected_date),
        Some(super::date(18))
    );

    renderer.render_frames(30);
    assert!(
        !renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Dialog)
    );
    activate(&mut renderer, "Open Input date dialog");
    choose_modal_day(&mut renderer, "July 21, 2026");
    let outside = PxPosition::from_f32_xy(10.0, 10.0);
    renderer.pointer_move(MOUSE_POINTER_ID, outside);
    renderer.pointer_press(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
    renderer.render_frames(4);
    assert!(!open.get());
    assert_eq!(
        source.single.with(super::DatePickerState::selected_date),
        Some(super::date(18))
    );

    renderer.render_frames(30);
    assert!(
        !renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Dialog)
    );
    activate(&mut renderer, "Open Input date dialog");
    choose_modal_day(&mut renderer, "July 22, 2026");
    activate(&mut renderer, "Modal confirm date");
    assert!(!open.get());
    assert_eq!(
        source.single.with(super::DatePickerState::selected_date),
        Some(super::date(22))
    );
    assert_eq!(
        crate::visible_text(&source.status.get()),
        "Confirmed date: 2026-07-22"
    );
}

#[tessera]
fn translation_fixture(translation_probe: tessera_ui::testing::TestProbe<Option<State<Dp>>>) {
    let translation = remember(|| Dp::ZERO);
    translation_probe.set(Some(translation));
    material_theme().child(move || {
        boxed()
            .modifier(
                Modifier::new()
                    .fill_max_width()
                    .push_placement(Translation(translation.get().to_pixels_f32())),
            )
            .children(|| {
                super::date_figure(super::Figure::Docked, true, false);
            });
    });
}

#[derive(Clone, Copy, PartialEq)]
struct Translation(f32);

impl PlacementModifierNode for Translation {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        position + PxPosition::from_f32_xy(0.0, self.0)
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn date_annotation_translation_keeps_raw_coordinates_without_rebuilding_local_callouts() {
    let translation_probe = tessera_ui::testing::TestProbe::default();
    let observed = translation_probe.clone();
    let mut renderer = HeadlessRenderer::new(
        Box::new(move || {
            translation_fixture(observed.clone());
        }),
        vec![Box::new(crate::TestPipelines)],
        HeadlessConfig {
            width: 800,
            height: 900,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(36);
    let before = field(&renderer).bounds().unwrap();
    let raw = field_coordinates(&renderer);
    let local = relative_geometry(&renderer);
    assert!(local.contains_key("figure") && local.contains_key("field"));
    assert!(
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| {
                node.role() == Role::Note && crate::has_label(&node, "Anatomy marker 2")
            }),
        "The native figure must publish its actual callouts before translation"
    );
    let builds = annotation_builds(&renderer);
    assert!(builds > 0, "The real figure must build its annotations");
    for translation in [48.0, 48.125, 96.625] {
        translation_probe.get().unwrap().set(Dp(translation));
        renderer.render_frames(6);
        let after = field(&renderer).bounds().unwrap();
        let moved = field_coordinates(&renderer);
        assert_eq!(after.y0, before.y0 + translation);
        assert_eq!(
            moved.window_bounds().y,
            raw.window_bounds().y + translation as f32
        );
        assert_eq!(relative_geometry(&renderer), local);
        assert_eq!(
            annotation_builds(&renderer),
            builds,
            "Outer translation must not rebuild local diagram callouts"
        );
    }
    activate(&mut renderer, "Choose month");
    renderer.render_frames(6);
    let changed = relative_geometry(&renderer);
    assert_ne!(
        changed["figure"].h, local["figure"].h,
        "The real month view changes the native artboard height"
    );
    assert!(
        annotation_builds(&renderer) > builds,
        "Real relative geometry changes must still update callouts"
    );
    assert!(
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| {
                node.role() == Role::ListBoxOption
                    && crate::has_label(&node, "August")
                    && node.is_selected() == Some(true)
            })
    );
}

#[tessera]
fn range_fixture(figure: super::Figure) {
    material_theme().child(move || {
        super::date_figure(figure, false, false);
    });
}

#[tessera]
fn article_fixture() {
    material_theme().child(super::date_pickers_spec);
}

#[tessera]
fn guideline_fixture(year: bool) {
    material_theme().child(move || {
        super::chapter_figure(
            super::Figure::Docked,
            if year {
                "ui-year-selection"
            } else {
                "ui-month-selection"
            }
            .into(),
            false,
        );
    });
}

#[tessera]
fn color_fixture(figure: super::Figure, menu: bool, year: bool) {
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                super::date_color_pair(figure, menu, year);
            });
    });
}

fn field(renderer: &HeadlessRenderer) -> Node {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::TextInput && crate::has_label(&node, "Event date"))
        .expect("date field")
        .1
}

fn activate(renderer: &mut HeadlessRenderer, label: &str) {
    let target_node = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| crate::has_label(&node, label) && node.supports_action(Action::Click))
        .unwrap_or_else(|| panic!("missing actionable {label}"))
        .0;
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node,
        data: None,
    }));
    renderer.render_frames(3);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn date_figures_keep_docked_anchors_and_commit_only_complete_ranges() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TestPipelines)],
        HeadlessConfig {
            width: 800,
            height: 800,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(4);
    let before = field(&renderer).bounds().unwrap();
    let visual = field_coordinates(&renderer);
    assert_eq!(visual.local_size.height.0, 56);
    let pixels = renderer.read_pixels().unwrap();
    let snapshot = renderer.accessibility_snapshot().unwrap();
    for label in ["Anatomy marker 2", "Anatomy marker 3"] {
        let marker = snapshot
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Note && crate::has_label(&node, label))
            .unwrap_or_else(|| panic!("missing {label}"));
        let bounds = marker.1.bounds().unwrap();
        let mut ink = 0;
        for y in bounds.y0 as usize..bounds.y1 as usize {
            for x in bounds.x0 as usize..bounds.x1 as usize {
                ink += usize::from(
                    pixels[(y * 800 + x) * 4..][..3]
                        .iter()
                        .all(|channel| *channel < 225),
                );
            }
        }
        assert!(
            ink > 20,
            "{label} must paint its outline and number: {bounds:?}, ink={ink}"
        );
    }
    let mut ink = 0;
    for y in before.y0 as usize..before.y1 as usize {
        for x in before.x0 as usize..(before.x0 + 128.0).min(before.x1) as usize {
            let pixel = &pixels[(y * 800 + x) * 4..][..3];
            ink += usize::from(pixel.iter().all(|channel| *channel < 128));
        }
    }
    assert!(
        ink > 20,
        "the selected date must be painted inside the field"
    );
    activate(&mut renderer, "Toggle event date calendar");
    assert_eq!(field(&renderer).bounds(), Some(before));
    assert!(
        !renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| crate::has_label(&node, "Docked calendar"))
    );
    activate(&mut renderer, "Toggle event date calendar");
    assert_eq!(field(&renderer).bounds(), Some(before));
    activate(&mut renderer, "August 19, 2025");
    assert_eq!(field(&renderer).value(), Some("08/19/2025"));
    activate(&mut renderer, "August 26, 2025");
    let snapshot = renderer.accessibility_snapshot().unwrap();
    let selected = snapshot
        .nodes
        .iter()
        .find(|(_, node)| {
            crate::has_label(&node, "August 26, 2025") && node.supports_action(Action::Click)
        })
        .unwrap()
        .1
        .bounds()
        .unwrap();
    let annotation = snapshot
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Note && crate::has_label(&node, "Anatomy marker 10"))
        .unwrap()
        .1
        .bounds()
        .unwrap();
    assert_eq!(
        (annotation.y0 + annotation.y1) / 2.0,
        (selected.y0 + selected.y1) / 2.0
    );
    activate(&mut renderer, "OK");
    assert_eq!(field(&renderer).bounds(), Some(before));
    assert_eq!(field(&renderer).value(), Some("08/26/2025"));
    activate(&mut renderer, "Toggle event date calendar");
    activate(&mut renderer, "August 20, 2025");
    assert_eq!(field(&renderer).value(), Some("08/20/2025"));
    activate(&mut renderer, "Cancel");
    assert_eq!(field(&renderer).bounds(), Some(before));
    assert_eq!(field(&renderer).value(), Some("08/26/2025"));
    renderer.resize(640, 800);
    renderer.render_frames(8);
    let narrow = field(&renderer).bounds().unwrap();
    let container = field_coordinates(&renderer).window_bounds();
    assert_eq!(container.width, 195.0);
    assert_eq!(container.height, 35.0);
    assert!(
        narrow.x0 >= f64::from(container.x)
            && narrow.y0 >= f64::from(container.y)
            && narrow.x1 <= f64::from(container.x + container.width)
            && narrow.y1 <= f64::from(container.y + container.height),
        "the actual editor must remain inside its scaled native field"
    );
    activate(&mut renderer, "Toggle event date calendar");
    let selected = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| {
            crate::has_label(&node, "August 19, 2025") && node.supports_action(Action::Click)
        })
        .unwrap()
        .1
        .bounds()
        .unwrap();
    let pointer = PxPosition::from_f32_xy(
        ((selected.x0 + selected.x1) / 2.0) as f32,
        ((selected.y0 + selected.y1) / 2.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, pointer);
    renderer.pointer_press(MOUSE_POINTER_ID, pointer, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, pointer, PressKeyEventType::Left);
    renderer.render_frames(4);
    assert_eq!(field(&renderer).value(), Some("08/19/2025"));
    assert_eq!(field(&renderer).bounds(), Some(narrow));
    activate(&mut renderer, "OK");
    activate(&mut renderer, "Toggle event date calendar");
    assert_eq!(field(&renderer).value(), Some("08/19/2025"));
    assert_eq!(field(&renderer).bounds(), Some(narrow));
    drop(renderer);
    for figure in [
        super::Figure::Range,
        super::Figure::RangeInput,
        super::Figure::FullScreen,
    ] {
        check_range_confirmation_and_cancellation(figure);
    }
    check_single_date_input_confirmation();
}

fn check_single_date_input_confirmation() {
    for (value, valid) in [("02/30/2025", false), ("02/28/2025", true)] {
        let figure = super::Figure::Input;
        let mut renderer = HeadlessRenderer::new(
            Box::new(move || {
                range_fixture(figure);
            }),
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width: 800,
                height: 800,
                sample_count: 1,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(4);
        let input = renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| node.role() == Role::TextInput)
            .expect("native date input")
            .0;
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action: Action::Focus,
            target_tree: TreeId::ROOT,
            target_node: input,
            data: None,
        }));
        renderer.render_frames(3);
        renderer.inject_text(value);
        renderer.render_frames(4);
        assert_eq!(
            range_action(&renderer, "Reference date confirm").is_disabled(),
            !valid,
            "only a complete selectable date may be confirmed: {value}"
        );
        if valid {
            activate(&mut renderer, "Reference date confirm");
        } else {
            activate(&mut renderer, "Reference date cancel");
        }
        let expected = if valid {
            "Date selection confirmed"
        } else {
            "Date selection cancelled"
        };
        assert!(
            renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| crate::has_label(&node, expected))
        );
    }
}

fn check_range_confirmation_and_cancellation(figure: super::Figure) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(move || {
            range_fixture(figure);
        }),
        vec![Box::new(crate::TestPipelines)],
        HeadlessConfig {
            width: 800,
            height: 900,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(4);
    if figure == super::Figure::RangeInput {
        activate(&mut renderer, "Switch to range calendar");
    }
    let (confirm, cancel) = if figure == super::Figure::FullScreen {
        ("Save", "Close full-screen date selection")
    } else {
        ("Reference range confirm", "Reference range cancel")
    };
    assert_range_endpoints(&renderer, 17, 23);
    activate(&mut renderer, "August 19, 2025");
    let confirmation = range_action(&renderer, confirm);
    assert!(
        confirmation.is_disabled(),
        "{confirm} must reject an incomplete range"
    );
    activate(&mut renderer, cancel);
    activate(&mut renderer, "Reopen reference date picker");
    assert_range_endpoints(&renderer, 17, 23);
    activate(&mut renderer, "August 19, 2025");
    activate(&mut renderer, "August 21, 2025");
    assert!(!range_action(&renderer, confirm).is_disabled());
    activate(&mut renderer, confirm);
    activate(&mut renderer, "Reopen reference date picker");
    assert_range_endpoints(&renderer, 19, 21);
    activate(&mut renderer, "August 20, 2025");
    activate(&mut renderer, "August 22, 2025");
    activate(&mut renderer, cancel);
    activate(&mut renderer, "Reopen reference date picker");
    assert_range_endpoints(&renderer, 19, 21);
}

fn range_action(renderer: &HeadlessRenderer, label: &str) -> Node {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::Button && crate::has_label(&node, label))
        .unwrap_or_else(|| panic!("missing range action {label}"))
        .1
}

fn assert_range_endpoints(renderer: &HeadlessRenderer, start: u8, end: u8) {
    let snapshot = renderer.accessibility_snapshot().unwrap();
    for (day, description) in [(start, "Start date"), (end, "End date")] {
        let label = format!("August {day}, 2025");
        assert!(
            snapshot.nodes.iter().any(|(_, node)| {
                crate::has_label(&node, label.as_str())
                    && crate::text_matches(node.description(), description)
            }),
            "{label} must remain the committed {description}"
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn date_article_switches_real_chapters_and_unmounts_inactive_pages() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(article_fixture),
        vec![Box::new(crate::TestPipelines)],
        HeadlessConfig {
            width: 900,
            height: 8000,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(4);
    for (page, present, absent) in [
        ("Overview", "Differences from M2", "Tokens & specs"),
        ("Guidelines", "Usage", "Differences from M2"),
        ("Accessibility", "Date entry methods", "Usage"),
        ("Specs", "Tokens & specs", "Date entry methods"),
    ] {
        activate(&mut renderer, page);
        let snapshot = renderer.accessibility_snapshot().unwrap();
        assert!(snapshot.nodes.iter().any(|(_, node)| {
            node.role() == Role::Tab
                && crate::has_label(&node, page)
                && node.is_selected() == Some(true)
        }));
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::Heading && crate::has_label(&node, present)),
            "missing {present} on {page}"
        );
        assert!(
            !snapshot
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::Heading && crate::has_label(&node, absent)),
            "inactive {absent} remained on {page}"
        );
        if page == "Overview" {
            let variants = snapshot
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Group && crate::has_label(&node, "Date picker variants")
                })
                .expect("native date picker overview figure")
                .1
                .bounds()
                .unwrap();
            assert!(
                (variants.height() - variants.width() * 376.0 / 1024.0).abs() <= 1.0,
                "the three-column overview must retain its reference canvas ratio"
            );
            let mut previous = None;
            for label in [
                "1 Docked date picker",
                "2 Modal date picker",
                "3 Modal date input",
            ] {
                let marker = snapshot
                    .nodes
                    .iter()
                    .find(|(_, node)| node.role() == Role::Note && crate::has_label(&node, label))
                    .unwrap_or_else(|| panic!("missing overview variant {label}"))
                    .1
                    .bounds()
                    .unwrap();
                if let Some(previous) = previous {
                    let previous: tessera_ui::accesskit::Rect = previous;
                    assert!(marker.x0 > previous.x1);
                    assert!((marker.y0 - previous.y0).abs() <= 1.0);
                }
                previous = Some(marker);
            }
            assert!(snapshot.nodes.iter().any(|(_, node)| {
                node.role() == Role::TextInput
                    && crate::has_label(&node, "Overview docked date")
                    && node.value() == Some("08/17/2025")
            }));
            for label in ["Docked calendar", "Select date"] {
                assert!(snapshot.nodes.iter().any(|(_, node)| {
                    node.role() == Role::Group && crate::has_label(&node, label)
                }));
            }
            assert!(snapshot.nodes.iter().any(|(_, node)| {
                node.role() == Role::TextInput && crate::has_label(&node, "Start date")
            }));
        }
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn date_reference_guidelines_and_color_pairs_mount_native_views() {
    for year in [false, true] {
        let mut renderer = reference_renderer(Scene::Guideline(year));
        renderer.render_frames(4);
        let label = if year { "2025" } else { "August" };
        assert!(
            renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| {
                    node.role() == Role::ListBoxOption
                        && crate::has_label(&node, label)
                        && node.is_selected() == Some(true)
                }),
            "the {label} menu must be open in its navigation guideline"
        );
    }
    for (figure, menu, year, title, marker) in [
        (
            super::Figure::Docked,
            false,
            false,
            "Docked date picker color roles",
            "Color role 11: On primary",
        ),
        (
            super::Figure::Docked,
            true,
            false,
            "Docked date picker menu color roles",
            "Color role 7: On surface",
        ),
        (
            super::Figure::Day,
            false,
            false,
            "Modal date picker color roles",
            "Color role 7: Primary",
        ),
        (
            super::Figure::Day,
            false,
            true,
            "Modal date picker year color roles",
            "Color role 9: On surface variant",
        ),
        (
            super::Figure::FullScreen,
            false,
            false,
            "Modal date range picker color roles",
            "Color role 8: Primary",
        ),
        (
            super::Figure::Input,
            false,
            false,
            "Modal date input color roles",
            "Color role 7: Outline variant",
        ),
    ] {
        let mut renderer = reference_renderer(Scene::Color(figure, menu, year));
        renderer.render_frames(4);
        let snapshot = renderer.accessibility_snapshot().unwrap();
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|(_, node)| { node.role() == Role::Group && crate::has_label(&node, title) })
        );
        assert_eq!(
            snapshot
                .nodes
                .iter()
                .filter(|(_, node)| {
                    node.role() == Role::Note && crate::has_label(&node, marker)
                })
                .count(),
            2,
            "both themes must paint {marker} in {title}"
        );
        if figure == super::Figure::Day {
            let containers: Vec<_> = snapshot
                .nodes
                .iter()
                .filter(|(_, node)| {
                    node.role() == Role::Note
                        && crate::has_label(&node, "Color role 3: Surface container high")
                })
                .collect();
            assert_eq!(containers.len(), 2);
            for (_, container) in containers {
                let bounds = container.bounds().unwrap();
                assert_eq!(
                    bounds.height(),
                    31.25,
                    "{title} must retain the full scaled marker"
                );
                assert!(
                    bounds.y0 >= 0.0,
                    "{title} must keep its marker on the canvas"
                );
            }
        }
        let pixels = renderer.read_pixels().unwrap();
        let light = &pixels[(20 * 1000 + 100) * 4..][..3];
        let dark = &pixels[(20 * 1000 + 600) * 4..][..3];
        assert_ne!(
            light, dark,
            "{title} must render separate light and dark themes"
        );
    }
}

#[derive(Clone)]
enum Scene {
    Modal(tessera_ui::testing::TestProbe<Option<(super::Sample, State<bool>)>>),
    Guideline(bool),
    Color(super::Figure, bool, bool),
}

fn reference_renderer(scene: Scene) -> HeadlessRenderer {
    HeadlessRenderer::new(
        Box::new(move || match scene.clone() {
            Scene::Modal(output) => {
                modal_example_fixture(output);
            }
            Scene::Guideline(year) => {
                guideline_fixture(year);
            }
            Scene::Color(figure, menu, year) => {
                color_fixture(figure, menu, year);
            }
        }),
        vec![Box::new(crate::TestPipelines)],
        HeadlessConfig {
            width: 1000,
            height: 1000,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap()
}

fn field_coordinates(renderer: &HeadlessRenderer) -> tessera_ui::LayoutCoordinates {
    renderer
        .inspect_components()
        .tagged("docked-date-field")
        .layout_coordinates()
        .expect("placed native date field")
}

fn relative_geometry(
    renderer: &HeadlessRenderer,
) -> std::collections::BTreeMap<&'static str, super::Rect> {
    let snapshot = renderer.inspect_components();
    let mut coordinates = std::collections::BTreeMap::new();
    for name in ["figure", "field", "container", "calendar"] {
        let tag = if name == "field" {
            "docked-date-field".to_owned()
        } else {
            format!("observed-{name}")
        };
        if let Some(node) = snapshot.find_tag(&tag)
            && let Some(value) = node.layout_coordinates()
        {
            coordinates.insert(name, value);
        }
    }
    super::annotations::relative_precise_geometry(&coordinates)
}

fn annotation_builds(renderer: &HeadlessRenderer) -> usize {
    renderer
        .work_totals()
        .count("date_annotations", tessera_ui::WorkPhase::Build)
        + renderer
            .work_totals()
            .count("date_color_annotations", tessera_ui::WorkPhase::Build)
}
