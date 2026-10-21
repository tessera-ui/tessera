//! Text accessibility metadata through the native renderer.
//!
//! ## Usage
//!
//! Preserve caller semantics while text and explicit labels change.

use std::cell::Cell;

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, Modifier, State,
    accesskit::{Node, NodeId, Role},
    remember, tessera,
};

use crate::{
    column::column,
    modifier::{ModifierExt as _, SemanticsArgs},
    text::text,
};

thread_local! {
    static CHANGED: Cell<Option<State<bool>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let changed = remember(|| false);
    CHANGED.set(Some(changed));
    column().children(move || {
        text()
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some("heading".into()),
                role: Some(Role::Heading),
                heading_level: Some(2),
                label: Some("Caller heading".into()),
                description: Some("Caller description".into()),
                tooltip: Some("Heading tooltip".into()),
                ..Default::default()
            }))
            .content(if changed.get() {
                "Updated heading"
            } else {
                "Visible heading"
            })
            .accessibility_label_optional(if changed.get() {
                Some("Explicit heading".into())
            } else {
                None
            })
            .accessibility_description_optional(if changed.get() {
                Some("Explicit description".into())
            } else {
                None
            });
        text()
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some("option".into()),
                role: Some(Role::ListBoxOption),
                selected: Some(true),
                disabled: true,
                ..Default::default()
            }))
            .content("Selected option");
        text()
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some("plain".into()),
                ..Default::default()
            }))
            .content(if changed.get() {
                "Changed text"
            } else {
                "Plain text"
            });
        text()
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some("hidden".into()),
                hidden: true,
                ..Default::default()
            }))
            .content("Hidden text");
    });
}

fn named(renderer: &HeadlessRenderer, label: &str) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing {label}"))
}

fn check(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 480,
            height: 240,
            sample_count: samples,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(3);
    let heading_id = named(&renderer, "Caller heading").0;
    let plain_id = named(&renderer, "Plain text").0;
    let initial_pixels = renderer.read_pixels().unwrap();
    for changed in [false, true, false] {
        CHANGED.get().unwrap().set(changed);
        renderer.render_frames(3);
        let (id, heading) = named(
            &renderer,
            if changed {
                "Explicit heading"
            } else {
                "Caller heading"
            },
        );
        assert_eq!(id, heading_id);
        assert_eq!(heading.role(), Role::Heading);
        assert_eq!(heading.level(), Some(2));
        assert_eq!(
            heading.description(),
            Some(if changed {
                "Explicit description"
            } else {
                "Caller description"
            })
        );
        assert_eq!(heading.tooltip(), Some("Heading tooltip"));
        assert_eq!(
            named(
                &renderer,
                if changed {
                    "Changed text"
                } else {
                    "Plain text"
                }
            )
            .0,
            plain_id
        );
        assert_eq!(
            named(
                &renderer,
                if changed {
                    "Changed text"
                } else {
                    "Plain text"
                }
            )
            .1
            .role(),
            Role::Label
        );
        let option = named(&renderer, "Selected option").1;
        assert_eq!(option.role(), Role::ListBoxOption);
        assert_eq!(option.is_selected(), Some(true));
        assert!(option.is_disabled());
        assert!(named(&renderer, "Hidden text").1.is_hidden());
    }
    assert_eq!(renderer.read_pixels().unwrap(), initial_pixels);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn caller_text_semantics_survive_defaults_and_explicit_label_updates_msaa1() {
    crate::test_utils::run_isolated_test(
        "text::semantics_tests::caller_text_semantics_survive_defaults_and_explicit_label_updates_msaa1",
        || check(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn caller_text_semantics_survive_defaults_and_explicit_label_updates_msaa4() {
    crate::test_utils::run_isolated_test(
        "text::semantics_tests::caller_text_semantics_survive_defaults_and_explicit_label_updates_msaa4",
        || check(4),
    );
}
