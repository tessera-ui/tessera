//! Chip layout, selection semantics, and live typography regressions.
//!
//! ## Usage
//!
//! Preserve compact control geometry and behavior as chip state changes.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Dp, MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition, State,
    accesskit::{Action, Role, Toggled},
    remember, tessera,
    testing::UiTest,
};

use crate::{
    column::column,
    md3::{
        material_icons::filled,
        theme::{MaterialTheme, material_theme},
    },
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

#[tessera]
fn fixture(probe: tessera_ui::testing::TestProbe<Option<State<Settings>>>) {
    let settings = remember(Settings::default);
    probe.set(Some(settings));
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
                    let builder = chip()
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
                        builder.leading_icon(filled::HOME_SVG);
                    }
                    if settings.get().trailing {
                        builder.trailing_icon(filled::CLOSE_SVG);
                    }
                }
            });
        });
}

pub(super) fn rect(renderer: &UiTest, key: &str) -> [i64; 4] {
    let snapshot = renderer.inspect_components();
    let node = snapshot
        .find_tag(key)
        .unwrap_or_else(|| snapshot.labeled(key));
    let bounds = node.bounds().expect("placed test target");
    [bounds.x, bounds.y, bounds.width, bounds.height].map(|value| value as i64)
}
fn check_chips() {
    for scale in [1.0, 2.0] {
        let probe = tessera_ui::testing::TestProbe::default();
        let mut renderer = UiTest::new()
            .viewport(800, 600)
            .scale_factor(scale)
            .rendered(crate::TesseraComponents)
            .content(|| fixture(probe.clone()));
        renderer.frames(5);
        let settings = probe.get().unwrap();
        for (leading, trailing) in [(false, false), (true, false), (false, true), (true, true)] {
            settings.set(Settings {
                leading,
                trailing,
                ..Default::default()
            });
            renderer.frames(5);
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
        renderer.frames(5);
        assert_eq!(rect(&renderer, "Filter sample")[3], 20 * scale as i64);
        settings.with_mut(|value| value.large = true);
        renderer.frames(5);
        assert_eq!(rect(&renderer, "Filter sample")[3], 28 * scale as i64);
        settings.with_mut(|value| value.large = false);
        renderer.frames(5);
        assert_eq!(rect(&renderer, "Filter sample")[3], 20 * scale as i64);
        let (id, filter) = renderer.accessibility_node("Control Filter sample");
        assert_eq!(filter.role(), Role::CheckBox);
        assert_eq!(filter.toggled(), Some(Toggled::False));
        assert_eq!(
            renderer
                .accessibility_node("Control Assist sample")
                .1
                .role(),
            Role::Button
        );
        assert_eq!(
            renderer
                .accessibility_node("Control Assist sample")
                .1
                .toggled(),
            None
        );
        assert!(renderer.accessibility_action(id, Action::Click));
        renderer.frames(5);
        assert_eq!(settings.get().calls, 1);
        assert_eq!(
            renderer
                .accessibility_node("Control Filter sample")
                .1
                .toggled(),
            Some(Toggled::True)
        );
        assert_eq!(
            renderer
                .accessibility_node("Control Input sample")
                .1
                .toggled(),
            Some(Toggled::True)
        );
        settings.with_mut(|value| value.disabled = true);
        renderer.frames(5);
        let (_, filter) = renderer.accessibility_node("Control Filter sample");
        assert!(filter.is_disabled());
        assert!(!filter.supports_action(Action::Click));
        let bounds = filter.bounds().unwrap();
        let point = PxPosition::new(
            (((bounds.x0 + bounds.x1) / 2.0) as i32).into(),
            (((bounds.y0 + bounds.y1) / 2.0) as i32).into(),
        );
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.frames(5);
        assert_eq!(settings.get().calls, 1);
        settings.with_mut(|value| value.disabled = false);
        renderer.frames(5);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.frames(5);
        assert_eq!(settings.get().calls, 2);
        assert_eq!(
            renderer
                .accessibility_node("Control Filter sample")
                .1
                .toggled(),
            Some(Toggled::False)
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn native_chips_preserve_edge_spacing_live_typography_selection_and_disabled_input() {
    crate::test_utils::run_isolated_test(
        "md3::chip::tests::native_chips_preserve_edge_spacing_live_typography_selection_and_disabled_input",
        check_chips,
    );
}
