//! List row typography, selection, and input regressions.
//!
//! ## Usage
//!
//! Keep list content consistent across layout and state changes.

use std::cell::Cell;

use tessera_ui::{
    Color, Dp, FocusManager, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier,
    PressKeyEventType, PxPosition, RenderSlot, State,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, TreeId},
    remember, tessera, use_context,
};

use crate::{
    alignment::CrossAxisAlignment,
    column::column,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    spacer::spacer,
    text::text,
    theme::{ContentColor, MaterialTheme, material_theme},
};

use super::list_item;

#[derive(Clone, Copy, PartialEq)]
struct Settings {
    width: Dp,
    large: bool,
    enabled: bool,
    selected: bool,
    calls: usize,
    custom: bool,
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
}

fn tagged(tag: &str) -> Modifier {
    Modifier::new().semantics(SemanticsArgs {
        test_tag: Some(tag.into()),
        ..Default::default()
    })
}

#[tessera]
fn slot(name: String) {
    let color = use_context::<ContentColor>()
        .expect("slot color")
        .get()
        .current;
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            spacer().modifier(
                tagged(&format!("{name}-ink"))
                    .size(Dp(8.0), Dp(8.0))
                    .background(color),
            );
            text().modifier(tagged(&name)).content(name.clone());
        });
}

#[tessera]
fn fixture() {
    let settings = remember(|| Settings {
        width: Dp(240.0),
        large: false,
        enabled: true,
        selected: false,
        calls: 0,
        custom: false,
    });
    SETTINGS.set(Some(settings));
    material_theme()
        .theme(move || {
            let mut theme = MaterialTheme::default();
            theme.color_scheme.surface = Color::WHITE;
            theme.color_scheme.secondary_container = Color::BLUE;
            theme.color_scheme.on_secondary_container = Color::WHITE;
            theme.color_scheme.on_surface = Color::RED;
            theme.color_scheme.on_surface_variant = if settings.get().large {
                Color::BLUE
            } else {
                Color::GREEN
            };
            if settings.get().large {
                theme.typography.title_medium.font_size = Dp(18.0);
                theme.typography.title_medium.line_height = Some(Dp(28.0));
                theme.typography.label_small.font_size = Dp(14.0);
                theme.typography.label_small.line_height = Some(Dp(20.0));
                theme.typography.body_large.font_size = Dp(22.0);
                theme.typography.body_large.line_height = Some(Dp(32.0));
                theme.typography.body_medium.font_size = Dp(18.0);
                theme.typography.body_medium.line_height = Some(Dp(28.0));
            }
            theme
        })
        .child(move || {
            column().children(move || {
                let (content, overline, supporting) = if settings.get().custom {
                    (
                        Some(RenderSlot::new(|| {
                            slot("Main".into());
                        })),
                        Some(RenderSlot::new(|| {
                            slot("Over".into());
                        })),
                        Some(RenderSlot::new(|| {
                            slot("Support".into());
                        })),
                    )
                } else {
                    (None, None, None)
                };
                list_item()
                    .modifier(tagged("item").width(settings.get().width))
                    .headline("H")
                    .content_optional(content)
                    .overline_content_optional(overline)
                    .supporting_content_optional(supporting)
                    .accessibility_label("List entry")
                    .enabled(settings.get().enabled)
                    .selected(settings.get().selected)
                    .leading(|| {
                        slot("Lead".into());
                    })
                    .trailing(|| {
                        slot("Meta".into());
                    })
                    .on_click(move || {
                        settings.with_mut(|settings| {
                            settings.calls += 1;
                            settings.selected = !settings.selected;
                        })
                    });
            });
        });
}

fn rect(renderer: &HeadlessRenderer, tag: &str) -> [i64; 4] {
    let snapshot = renderer.snapshot();
    let mut pending = vec![&snapshot["root"]];
    while let Some(node) = pending.pop() {
        if node["accessibility"]["key"] == tag || node["accessibility"]["label"] == tag {
            return ["x", "y", "width", "height"].map(|key| node["bounds"][key].as_i64().unwrap());
        }
        if let Some(children) = node["children"].as_array() {
            pending.extend(children);
        }
    }
    panic!("missing {tag}");
}

fn entry(renderer: &HeadlessRenderer) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::ListItem && node.label() == Some("List entry"))
        .unwrap()
}

fn click(renderer: &mut HeadlessRenderer) {
    let [x, y, width, height] = rect(renderer, "item");
    let point_y = if SETTINGS.get().unwrap().get().custom {
        y + height - 4
    } else {
        y + height / 2
    };
    let point = PxPosition::from_f32_xy((x + width / 2) as f32, point_y as f32);
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(4);
    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(630.0, 390.0));
    renderer.render_frames(60);
}

fn assert_ink(renderer: &HeadlessRenderer, tag: &str, expected: [u8; 4]) {
    let [x, y, width, height] = rect(renderer, tag);
    let pixels = renderer.read_pixels().unwrap();
    let offset = (((y + height / 2) as u32 * renderer.physical_size().width
        + (x + width / 2) as u32)
        * 4) as usize;
    assert_eq!(&pixels[offset..offset + 4], &expected);
}

fn check(samples: u32) {
    for (scale, width, height, lead_height, trail_height) in [
        (1.0, 240, 56, 24, 16),
        (1.25, 300, 70, 30, 20),
        (1.5, 360, 84, 36, 24),
        (2.0, 480, 112, 48, 32),
    ] {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 640,
                height: 400,
                scale_factor: scale,
                sample_count: samples,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(8);
        let settings = SETTINGS.get().unwrap();
        assert_eq!(rect(&renderer, "item"), [0, 0, width, height]);
        assert_eq!(rect(&renderer, "Lead")[3], lead_height);
        assert_eq!(rect(&renderer, "Meta")[3], trail_height);
        assert_ink(&renderer, "Lead-ink", [0, 255, 0, 255]);
        assert_ink(&renderer, "Meta-ink", [0, 255, 0, 255]);
        let id = entry(&renderer).0;
        assert_eq!(entry(&renderer).1.is_selected(), Some(false));
        settings.with_mut(|settings| settings.custom = true);
        renderer.render_frames(30);
        assert_eq!(entry(&renderer).0, id);
        assert_eq!(rect(&renderer, "Main")[3], (24.0 * scale) as i64);
        assert_eq!(rect(&renderer, "Over")[3], (16.0 * scale) as i64);
        assert_eq!(rect(&renderer, "Support")[3], (20.0 * scale) as i64);
        assert_ink(&renderer, "Main-ink", [255, 0, 0, 255]);
        assert_ink(&renderer, "Over-ink", [0, 255, 0, 255]);
        assert_ink(&renderer, "Support-ink", [0, 255, 0, 255]);
        settings.with_mut(|settings| settings.large = true);
        renderer.render_frames(30);
        assert_eq!(rect(&renderer, "Main")[3], (32.0 * scale) as i64);
        assert_eq!(rect(&renderer, "Over")[3], (20.0 * scale) as i64);
        assert_eq!(rect(&renderer, "Support")[3], (28.0 * scale) as i64);
        assert_ink(&renderer, "Over-ink", [0, 0, 255, 255]);
        assert_ink(&renderer, "Support-ink", [0, 0, 255, 255]);
        click(&mut renderer);
        assert_eq!(settings.get().calls, 1);
        assert!(
            settings.get().selected,
            "custom slots must preserve row activation after reflow"
        );
        for tag in ["Main-ink", "Over-ink", "Support-ink"] {
            assert_ink(&renderer, tag, [255; 4]);
        }
        settings.with_mut(|settings| {
            settings.custom = false;
            settings.large = false;
            settings.selected = false;
            settings.calls = 0;
        });
        FocusManager::current().clear_focus(true);
        renderer.render_frames(30);
        assert_eq!(rect(&renderer, "item"), [0, 0, width, height]);
        settings.with_mut(|settings| settings.large = true);
        renderer.render_frames(30);
        assert_eq!(rect(&renderer, "Lead")[3], (28.0 * scale) as i64);
        assert_eq!(rect(&renderer, "Meta")[3], (20.0 * scale) as i64);
        assert_ink(&renderer, "Lead-ink", [0, 0, 255, 255]);
        assert_ink(&renderer, "Meta-ink", [0, 0, 255, 255]);
        settings.with_mut(|settings| settings.large = false);
        renderer.render_frames(8);
        click(&mut renderer);
        assert_eq!(settings.get().calls, 1);
        assert_eq!(entry(&renderer).0, id);
        assert_eq!(entry(&renderer).1.is_selected(), Some(true));
        assert_ink(&renderer, "Lead-ink", [255; 4]);
        settings.with_mut(|settings| settings.enabled = false);
        renderer.render_frames(8);
        assert!(entry(&renderer).1.is_disabled());
        assert!(!entry(&renderer).1.supports_action(Action::Click));
        assert!(!entry(&renderer).1.supports_action(Action::Focus));
        click(&mut renderer);
        assert_eq!(settings.get().calls, 1);
        settings.with_mut(|settings| {
            settings.enabled = true;
            settings.width = Dp::ZERO;
        });
        renderer.render_frames(8);
        assert_eq!(rect(&renderer, "item")[2], 0);
        settings.with_mut(|settings| settings.width = Dp(240.0));
        renderer.render_frames(8);
        assert_eq!(rect(&renderer, "item"), [0, 0, width, height]);
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: id,
            data: None
        }));
        renderer.render_frames(8);
        assert_eq!(settings.get().calls, 2);
        assert_eq!(entry(&renderer).1.is_selected(), Some(false));
        renderer.resize(320, 400);
        renderer.render_frames(8);
        assert!(rect(&renderer, "item")[2] <= 320);
        renderer.resize(640, 400);
        renderer.render_frames(8);
        assert_eq!(rect(&renderer, "item"), [0, 0, width, height]);
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn list_item_context_updates_selection_disabled_input_and_resize_msaa1() {
    crate::test_utils::run_isolated_test(
        "list_item::native_tests::list_item_context_updates_selection_disabled_input_and_resize_msaa1",
        || check(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn list_item_context_updates_selection_disabled_input_and_resize_msaa4() {
    crate::test_utils::run_isolated_test(
        "list_item::native_tests::list_item_context_updates_selection_disabled_input_and_resize_msaa4",
        || check(4),
    );
}
