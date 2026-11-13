//! Snackbar layout and responsive rendering regressions.
//!
//! ## Usage
//!
//! Keep notification text and actions aligned at compact and constrained sizes.

use tessera_foundation::modifier::{ModifierExt as _, Padding, SemanticsArgs};
use tessera_ui::{
    Dp, Modifier, State,
    accesskit::{Node, Role},
    remember, tessera,
    testing::UiTest,
};

use crate::{column::column, md3::theme::material_theme};

use super::snackbar;

#[derive(Clone, Debug, PartialEq)]
struct Settings {
    width: f64,
    message: String,
    action: Option<String>,
    dismiss: bool,
    stacked: bool,
    padding: Option<Padding>,
}

#[derive(Clone, Copy, Default)]
struct FixtureHandles {
    settings: Option<State<Settings>>,
}

#[tessera]
fn fixture(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
    let settings = remember(|| Settings {
        width: 320.0,
        message: "Saved".into(),
        action: None,
        dismiss: false,
        stacked: false,
        padding: None,
    });
    observed.with_mut(|handles| handles.settings = Some(settings));
    material_theme().child(move || {
        column().children(move || {
            let s = settings.get();
            snackbar()
                .modifier(Modifier::new().width(Dp(s.width)).semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some("Notification".into()),
                    merge_descendants: true,
                    ..Default::default()
                }))
                .message(s.message)
                .action_label_optional(s.action)
                .with_dismiss_action(s.dismiss)
                .action_on_new_line(s.stacked)
                .content_padding_optional(s.padding);
        });
    });
}

fn node(renderer: &UiTest, label: &str, role: Role) -> Node {
    let snapshot = renderer
        .inspect_accessibility()
        .expect("accessibility tree");
    let (_, node) = snapshot
        .find(label, role)
        .unwrap_or_else(|| panic!("missing {role:?}: {label}"));
    node.clone()
}

fn check() {
    for scale in [1.0, 1.25, 1.5, 2.0] {
        let observed = tessera_ui::testing::TestProbe::default();
        let mut renderer = UiTest::new()
            .viewport(800, 600)
            .scale_factor(scale)
            .rendered(crate::TesseraComponents)
            .content(|| fixture(observed.clone()));
        renderer.frames(4);
        let settings = observed.get().settings.unwrap();
        let initial = settings.get();
        let baseline = renderer.read_pixels();
        for (message, text_height, height) in
            [("Saved", 20.0, 48.0), ("Saved\nto library", 40.0, 68.0)]
        {
            for (action, dismiss) in [(false, false), (true, false), (false, true), (true, true)] {
                settings.with_mut(|s| {
                    s.message = message.into();
                    s.action = action.then(|| "Undo".into());
                    s.dismiss = dismiss;
                    s.stacked = false;
                });
                renderer.frames(4);
                let bounds = node(&renderer, "Notification", Role::Group)
                    .bounds()
                    .unwrap();
                let text = node(&renderer, message, Role::Status).bounds().unwrap();
                assert_eq!(bounds.width(), 320.0 * scale);
                assert_eq!(bounds.height(), height * scale);
                assert_eq!(text.height(), text_height * scale);
                assert_eq!(text.x0 - bounds.x0, 16.0 * scale);
                assert_eq!(text.y0 - bounds.y0, 14.0 * scale);
                if action {
                    let control = node(&renderer, "Undo", Role::Button).bounds().unwrap();
                    assert!(control.height() >= 48.0 * scale);
                    assert!(
                        ((control.y0 + control.y1) / 2.0 - (bounds.y0 + bounds.y1) / 2.0).abs()
                            <= 0.5
                    );
                    assert!(text.x1 <= control.x0);
                    if !dismiss {
                        assert_eq!(bounds.x1 - control.x1, 8.0 * scale);
                    }
                }
                if dismiss {
                    let close = node(&renderer, "Dismiss", Role::Button).bounds().unwrap();
                    assert_eq!(close.x1, bounds.x1);
                    assert!(
                        ((close.y0 + close.y1) / 2.0 - (bounds.y0 + bounds.y1) / 2.0).abs() <= 0.5
                    );
                }
            }
        }
        settings.with_mut(|s| {
            s.width = 200.0;
            s.message = "Saved to your library with all changes".into();
        });
        renderer.frames(4);
        let text = node(&renderer, &settings.get().message, Role::Status)
            .bounds()
            .unwrap();
        let bounds = node(&renderer, "Notification", Role::Group)
            .bounds()
            .unwrap();
        assert!(text.height() > 20.0 * scale);
        assert_eq!(bounds.height(), text.height() + 14.0 * scale * 2.0);
        settings.with_mut(|s| {
            s.width = 320.0;
            s.message = "Saved".into();
            s.action = Some("Undo".into());
            s.stacked = true;
        });
        renderer.frames(4);
        let bounds = node(&renderer, "Notification", Role::Group)
            .bounds()
            .unwrap();
        let action = node(&renderer, "Undo", Role::Button).bounds().unwrap();
        let text = node(&renderer, "Saved", Role::Status).bounds().unwrap();
        assert_eq!(bounds.height(), 100.0 * scale);
        assert_eq!(text.y0 - bounds.y0, 14.0 * scale);
        assert_eq!(action.y0 - bounds.y0, 48.0 * scale);
        assert_eq!(bounds.y1 - action.y1, 4.0 * scale);
        settings.with_mut(|s| {
            s.width = 200.0;
            s.action = Some("Restore the previous changes".into());
        });
        renderer.frames(4);
        let bounds = node(&renderer, "Notification", Role::Group)
            .bounds()
            .unwrap();
        let action = node(&renderer, "Restore the previous changes", Role::Button)
            .bounds()
            .unwrap();
        assert!(action.x0 >= bounds.x0 && action.x1 <= bounds.x1);
        assert!(action.y0 >= text.y1);
        settings.set(initial.clone());
        settings.with_mut(|s| s.padding = Some(super::SnackbarDefaults::CONTENT_PADDING));
        renderer.frames(4);
        assert!(renderer.read_pixels() == baseline);
        settings.with_mut(|s| s.width = 0.0);
        renderer.frames(3);
        settings.set(initial);
        renderer.frames(5);
        assert!(renderer.read_pixels() == baseline);
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn snackbar_measured_wrapping_action_rows_and_zero_width_recover() {
    crate::test_utils::run_isolated_test(
        "md3::snackbar::geometry_tests::snackbar_measured_wrapping_action_rows_and_zero_width_recover",
        check,
    );
}
