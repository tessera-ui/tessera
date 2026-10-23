//! Segmented selection, responsive geometry, and live typography regressions.
//!
//! ## Usage
//!
//! Keep connected choices accessible and usable as state and bounds change.

use std::cell::Cell;

use tessera_ui::{
    Dp, FocusDirection, FocusManager, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier,
    PressKeyEventType, PxPosition, State,
    accesskit::{Action, ActionRequest, Node, NodeId, Orientation, Role, Toggled, TreeId},
    remember, tessera,
};

use crate::{
    button::button,
    column::column,
    modifier::{ModifierExt as _, SemanticsArgs},
    text::text,
    theme::{MaterialTheme, material_theme},
};

use super::{
    SegmentedButtonDefaults, multi_choice_segmented_button_row, segmented_button,
    single_choice_segmented_button_row,
};

#[derive(Clone, Copy, PartialEq)]
struct Settings {
    selected: usize,
    checked: [bool; 2],
    disabled: bool,
    large: bool,
    width: Dp,
    natural: bool,
    calls: usize,
    height: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            selected: 1,
            checked: [false, true],
            disabled: false,
            large: false,
            width: Dp(360.0),
            natural: true,
            calls: 0,
            height: 40.0,
        }
    }
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let settings = remember(Settings::default);
    SETTINGS.set(Some(settings));
    material_theme()
        .theme(move || {
            let mut theme = MaterialTheme::default();
            if settings.get().large {
                theme.typography.label_large.line_height = Some(Dp(28.0));
            }
            theme
        })
        .child(move || {
            column().children(move || {
                single_choice_segmented_button_row()
                    .modifier(
                        (if settings.get().natural {
                            Modifier::new()
                        } else {
                            Modifier::new().width(settings.get().width)
                        })
                        .semantics(SemanticsArgs {
                            label: Some("View options".to_owned()),
                            description: Some("Calendar period".to_owned()),
                            ..Default::default()
                        }),
                    )
                    .content(move || {
                        for (index, label) in ["Day", "Week", "Month"].into_iter().enumerate() {
                            segmented_button()
                                .label(label)
                                .min_height(Dp(settings.get().height))
                                .content_padding(crate::modifier::Padding::symmetric(
                                    Dp(12.0),
                                    Dp(((settings.get().height - 20.0) * 0.5).clamp(0.0, 8.0)),
                                ))
                                .accessibility_label(format!("Single {label}"))
                                .selected(settings.get().selected == index)
                                .enabled(index != 1 || !settings.get().disabled)
                                .shape(SegmentedButtonDefaults::item_shape(
                                    index,
                                    3,
                                    SegmentedButtonDefaults::shape(),
                                ))
                                .on_click(move || {
                                    settings.with_mut(|value| {
                                        value.selected = index;
                                        value.calls += 1;
                                    })
                                });
                        }
                    });
                multi_choice_segmented_button_row().content(move || {
                    for (index, label) in ["Bold", "Italic"].into_iter().enumerate() {
                        segmented_button()
                            .label(label)
                            .accessibility_label(format!("Multi {label}"))
                            .selected(settings.get().checked[index])
                            .enabled(index != 1 || !settings.get().disabled)
                            .on_click(move || {
                                settings.with_mut(|value| {
                                    value.checked[index] = !value.checked[index];
                                    value.calls += 1;
                                })
                            });
                    }
                });
                segmented_button()
                    .accessibility_label("Indicator")
                    .selected(settings.get().selected == 1)
                    .on_click(|| {});
                button()
                    .accessibility_label("After")
                    .on_click(|| {})
                    .child(|| {
                        text().content("Continue");
                    });
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

fn action(renderer: &mut HeadlessRenderer, label: &str, action: Action) -> bool {
    renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: node(renderer, label).0,
        data: None,
    })
}

fn traverse(renderer: &mut HeadlessRenderer, direction: FocusDirection, label: &str) {
    FocusManager::current().request_move_focus(direction);
    renderer.render_frames(5);
    assert_eq!(
        renderer.accessibility_snapshot().unwrap().focus,
        node(renderer, label).0,
        "{direction:?} to {label}; nodes: {:?}",
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .map(|(id, node)| (*id, node.role(), node.label()))
            .collect::<Vec<_>>()
    );
}

fn text_rect(renderer: &HeadlessRenderer, label: &str) -> [i64; 4] {
    let snapshot = renderer.snapshot();
    let mut pending = vec![&snapshot["root"]];
    while let Some(node) = pending.pop() {
        if node["accessibility"]["label"].as_str() == Some(label) {
            return ["x", "y", "width", "height"]
                .map(|field| node["bounds"][field].as_i64().unwrap());
        }
        if let Some(children) = node["children"].as_array() {
            pending.extend(children);
        }
    }
    panic!("missing {label}");
}

fn text_height(renderer: &HeadlessRenderer, label: &str) -> i64 {
    text_rect(renderer, label)[3]
}

fn indicator_ink(renderer: &HeadlessRenderer) -> usize {
    let bounds = node(renderer, "Indicator").1.bounds().unwrap();
    let center_x = ((bounds.x0 + bounds.x1) / 2.0) as u32;
    let center_y = ((bounds.y0 + bounds.y1) / 2.0) as u32;
    let pixels = renderer.read_pixels().unwrap();
    let width = renderer.physical_size().width;
    let mut count = 0;
    for y in center_y - 9..center_y + 9 {
        for x in center_x - 9..center_x + 9 {
            let offset = ((y * width + x) * 4) as usize;
            if pixels[offset + 3] > 128
                && pixels[offset..offset + 3].iter().all(|value| *value < 128)
            {
                count += 1;
            }
        }
    }
    count
}

fn check_segments(samples: u32) {
    for scale in [1.0, 1.25, 1.5, 2.0] {
        check_segments_at_density(samples, scale);
    }
}

fn check_segments_at_density(samples: u32, scale: f64) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 900,
            height: 600,
            scale_factor: scale,
            sample_count: samples,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    let settings = SETTINGS.get().unwrap();
    let natural_width = node(&renderer, "Single Month").1.bounds().unwrap().x1;
    let day = node(&renderer, "Single Day").1.bounds().unwrap();
    let label = text_rect(&renderer, "Day");
    assert!(
        (label[0] as f64 + label[2] as f64 / 2.0 - (day.x0 + day.x1) / 2.0).abs() <= 1.0,
        "labels without icons stay centered"
    );
    assert!(
        indicator_ink(&renderer) > 5,
        "selected segments need visible checkmarks"
    );
    let group = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::RadioGroup)
        .unwrap()
        .1;
    assert_eq!(group.children().len(), 3);
    assert_eq!(group.label(), Some("View options"));
    assert_eq!(group.description(), Some("Calendar period"));
    assert_eq!(group.orientation(), Some(Orientation::Horizontal));
    for (label, checked) in [
        ("Single Day", false),
        ("Single Week", true),
        ("Single Month", false),
    ] {
        let item = node(&renderer, label).1;
        assert_eq!(item.role(), Role::RadioButton);
        assert_eq!(
            item.toggled(),
            Some(if checked {
                Toggled::True
            } else {
                Toggled::False
            })
        );
        assert!(item.supports_action(Action::Click));
    }
    assert_eq!(node(&renderer, "Multi Italic").1.role(), Role::CheckBox);
    assert_eq!(
        node(&renderer, "Multi Italic").1.toggled(),
        Some(Toggled::True)
    );
    traverse(&mut renderer, FocusDirection::Next, "Single Week");
    assert_eq!(
        settings.get().calls,
        0,
        "Tab entry must not activate a segment"
    );
    traverse(&mut renderer, FocusDirection::Right, "Single Month");
    assert_eq!(settings.get().selected, 2);
    assert_eq!(
        node(&renderer, "Single Month").1.bounds().unwrap().x1,
        natural_width,
        "selection preserves natural group width"
    );
    assert_eq!(settings.get().calls, 1);
    traverse(&mut renderer, FocusDirection::Next, "Multi Bold");
    assert_eq!(settings.get().checked, [false, true]);
    traverse(&mut renderer, FocusDirection::Right, "Multi Italic");
    assert_eq!(
        settings.get().checked,
        [false, true],
        "multi-choice navigation preserves selection"
    );
    traverse(&mut renderer, FocusDirection::Next, "Indicator");
    traverse(&mut renderer, FocusDirection::Next, "After");
    assert!(action(&mut renderer, "Multi Bold", Action::Click));
    renderer.render_frames(5);
    assert_eq!(settings.get().checked, [true, true]);
    assert!(action(&mut renderer, "Single Day", Action::Focus));
    renderer.render_frames(5);
    assert_eq!(settings.get().selected, 2, "explicit focus must not select");
    let bounds = node(&renderer, "Single Day").1.bounds().unwrap();
    let center = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    renderer.pointer_press(MOUSE_POINTER_ID, center, PressKeyEventType::Left);
    renderer.render_frames(5);
    assert_eq!(
        settings.get().selected,
        2,
        "pointer press must wait for completion"
    );
    renderer.pointer_release(MOUSE_POINTER_ID, center, PressKeyEventType::Left);
    renderer.render_frames(5);
    assert_eq!(settings.get().selected, 0);
    assert_eq!(
        node(&renderer, "Single Month").1.bounds().unwrap().x1,
        natural_width,
        "deselection preserves natural group width"
    );
    assert_eq!(
        indicator_ink(&renderer),
        0,
        "deselection removes the default checkmark"
    );
    settings.with_mut(|value| {
        value.selected = 1;
        value.disabled = true;
    });
    renderer.render_frames(5);
    let disabled = node(&renderer, "Single Week").1;
    assert!(disabled.is_disabled());
    assert_eq!(disabled.toggled(), Some(Toggled::True));
    assert!(!disabled.supports_action(Action::Click));
    assert!(!action(&mut renderer, "Single Week", Action::Click));
    assert!(action(&mut renderer, "Single Day", Action::Focus));
    renderer.render_frames(5);
    traverse(&mut renderer, FocusDirection::Right, "Single Month");
    assert_eq!(settings.get().selected, 2);
    assert_eq!(text_height(&renderer, "Month"), (20.0 * scale) as i64);
    settings.with_mut(|value| value.large = true);
    renderer.render_frames(5);
    assert_eq!(text_height(&renderer, "Month"), (28.0 * scale) as i64);
    settings.with_mut(|value| {
        value.large = false;
        value.disabled = false;
        value.natural = false;
    });
    for width in [181.0, 0.0, 360.0, 181.0, 360.0] {
        settings.with_mut(|value| value.width = Dp(width));
        renderer.render_frames(5);
        let day = node(&renderer, "Single Day").1.bounds().unwrap();
        let week = node(&renderer, "Single Week").1.bounds().unwrap();
        let month = node(&renderer, "Single Month").1.bounds().unwrap();
        assert!(
            month.x1 == (width * scale).floor(),
            "segments overflow {width}: {month:?}"
        );
        if width > 0.0 {
            assert_eq!(week.x0, day.x1 - scale.floor());
            assert_eq!(month.x0, week.x1 - scale.floor());
            assert!(((day.x1 - day.x0) - (month.x1 - month.x0)).abs() <= 1.0);
        }
    }
    assert_eq!(text_height(&renderer, "Month"), (20.0 * scale) as i64);
    for height in [40.0, 36.0, 32.0, 28.0] {
        settings.with_mut(|settings| {
            settings.height = height;
            settings.selected = 0;
            settings.width = Dp(360.0);
            settings.natural = false;
        });
        renderer.render_frames(5);
        let target = node(&renderer, "Single Day").1.bounds().unwrap();
        assert_eq!(
            target.height(),
            (48.0 * scale).floor(),
            "density preserves the interactive height"
        );
        let pixels = renderer.read_pixels().unwrap();
        let width = renderer.physical_size().width as usize;
        let background = &pixels[..4];
        let x = ((target.x0 + target.x1) * 0.5) as usize;
        let occupied = (target.y0 as usize..target.y1 as usize)
            .filter(|y| &pixels[(*y * width + x) * 4..(*y * width + x) * 4 + 4] != background)
            .collect::<Vec<_>>();
        let painted = (occupied.last().unwrap() - occupied.first().unwrap() + 1) as f64;
        assert!(
            (painted - height * scale).abs() <= 1.0,
            "painted height {painted} for density height {height} at {scale}"
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn native_segment_selection_focus_live_typography_and_narrow_recovery_msaa1() {
    crate::test_utils::run_isolated_test(
        "segmented_buttons::tests::native_segment_selection_focus_live_typography_and_narrow_recovery_msaa1",
        || check_segments(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn native_segment_selection_focus_live_typography_and_narrow_recovery_msaa4() {
    crate::test_utils::run_isolated_test(
        "segmented_buttons::tests::native_segment_selection_focus_live_typography_and_narrow_recovery_msaa4",
        || check_segments(4),
    );
}
