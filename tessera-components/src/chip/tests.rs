//! Chip layout, selection semantics, and live typography regressions.
//!
//! ## Usage
//!
//! Preserve compact control geometry and behavior as chip state changes.

use std::cell::Cell;

use tessera_ui::{
    Dp, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier, PressKeyEventType,
    PxPosition, State,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, Toggled, TreeId},
    remember, tessera,
};

use crate::{
    column::column,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    theme::{MaterialTheme, material_theme},
};

use super::{ChipVariant, chip};

#[derive(Clone, Copy, Default, PartialEq)]
struct Settings {
    leading: bool,
    trailing: bool,
    selected: bool,
    disabled: bool,
    large: bool,
    calls: usize,
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
                theme.typography.label_large.font_size = Dp(18.0);
                theme.typography.label_large.line_height = Some(Dp(28.0));
            }
            theme
        })
        .child(move || {
            column().children(move || {
                for (variant, label) in [
                    (ChipVariant::Assist, "Assist sample"),
                    (ChipVariant::Suggestion, "Suggestion sample"),
                    (ChipVariant::Filter, "Filter sample"),
                    (ChipVariant::Input, "Input sample"),
                ] {
                    let mut builder = chip()
                        .variant(variant)
                        .label(label)
                        .accessibility_label(format!("Control {label}"))
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            test_tag: Some(format!("Control {label}")),
                            ..Default::default()
                        }))
                        .selected(settings.get().selected)
                        .enabled(!settings.get().disabled)
                        .on_click(move || {
                            settings.with_mut(|value| {
                                value.selected = !value.selected;
                                value.calls += 1;
                            })
                        });
                    if settings.get().leading {
                        builder = builder.leading_icon(filled::HOME_SVG);
                    }
                    if settings.get().trailing {
                        builder = builder.trailing_icon(filled::CLOSE_SVG);
                    }
                    drop(builder);
                }
            });
        });
}

fn rect(renderer: &HeadlessRenderer, key: &str) -> [i64; 4] {
    let snapshot = renderer.snapshot();
    let mut pending = vec![&snapshot["root"]];
    while let Some(node) = pending.pop() {
        if node["accessibility"]["key"] == key || node["accessibility"]["label"] == key {
            return ["x", "y", "width", "height"]
                .map(|field| node["bounds"][field].as_i64().unwrap());
        }
        if let Some(children) = node["children"].as_array() {
            pending.extend(children);
        }
    }
    panic!("missing {key}");
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

fn check_chips() {
    for scale in [1.0, 2.0] {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 800,
                height: 600,
                scale_factor: scale,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(5);
        let settings = SETTINGS.get().unwrap();
        for (leading, trailing) in [(false, false), (true, false), (false, true), (true, true)] {
            settings.set(Settings {
                leading,
                trailing,
                ..Default::default()
            });
            renderer.render_frames(5);
            for (input, label) in [
                (false, "Assist sample"),
                (false, "Suggestion sample"),
                (false, "Filter sample"),
                (true, "Input sample"),
            ] {
                let outer = rect(&renderer, &format!("Control {label}"));
                let text = rect(&renderer, label);
                let start = if leading {
                    34
                } else if input {
                    12
                } else {
                    16
                };
                let end = if trailing {
                    34
                } else if input {
                    12
                } else {
                    16
                };
                assert_eq!(text[0] - outer[0], start * scale as i64, "{label} start");
                assert_eq!(
                    outer[0] + outer[2] - text[0] - text[2],
                    end * scale as i64,
                    "{label} end"
                );
            }
        }
        settings.set(Settings::default());
        renderer.render_frames(5);
        assert_eq!(rect(&renderer, "Filter sample")[3], 20 * scale as i64);
        settings.with_mut(|value| value.large = true);
        renderer.render_frames(5);
        assert_eq!(rect(&renderer, "Filter sample")[3], 28 * scale as i64);
        settings.with_mut(|value| value.large = false);
        renderer.render_frames(5);
        assert_eq!(rect(&renderer, "Filter sample")[3], 20 * scale as i64);
        let (id, filter) = node(&renderer, "Control Filter sample");
        assert_eq!(filter.role(), Role::CheckBox);
        assert_eq!(filter.toggled(), Some(Toggled::False));
        assert_eq!(
            node(&renderer, "Control Assist sample").1.role(),
            Role::Button
        );
        assert_eq!(node(&renderer, "Control Assist sample").1.toggled(), None);
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: id,
            data: None,
        }));
        renderer.render_frames(5);
        assert_eq!(settings.get().calls, 1);
        assert_eq!(
            node(&renderer, "Control Filter sample").1.toggled(),
            Some(Toggled::True)
        );
        assert_eq!(
            node(&renderer, "Control Input sample").1.toggled(),
            Some(Toggled::True)
        );
        settings.with_mut(|value| value.disabled = true);
        renderer.render_frames(5);
        let (_, filter) = node(&renderer, "Control Filter sample");
        assert!(filter.is_disabled());
        assert!(!filter.supports_action(Action::Click));
        let bounds = filter.bounds().unwrap();
        let point = PxPosition::new(
            (((bounds.x0 + bounds.x1) / 2.0) as i32).into(),
            (((bounds.y0 + bounds.y1) / 2.0) as i32).into(),
        );
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(5);
        assert_eq!(settings.get().calls, 1);
        settings.with_mut(|value| value.disabled = false);
        renderer.render_frames(5);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(5);
        assert_eq!(settings.get().calls, 2);
        assert_eq!(
            node(&renderer, "Control Filter sample").1.toggled(),
            Some(Toggled::False)
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn native_chips_preserve_edge_spacing_live_typography_selection_and_disabled_input() {
    crate::test_utils::run_isolated_test(
        "chip::tests::native_chips_preserve_edge_spacing_live_typography_selection_and_disabled_input",
        check_chips,
    );
}
