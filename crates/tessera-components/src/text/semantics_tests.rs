//! Text accessibility metadata through the native renderer.
//!
//! ## Usage
//!
//! Preserve caller semantics while text and explicit labels change.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Modifier, State,
    accesskit::{Node, NodeId, Role},
    headless::{HeadlessConfig, HeadlessRenderer},
    remember, tessera,
    testing::TestProbe,
};

use crate::{column::column, text::text};

struct PlainLabel(&'static str);

impl From<PlainLabel> for String {
    fn from(label: PlainLabel) -> Self {
        label.0.into()
    }
}

#[tessera]
fn fixture(probe: Option<TestProbe<Option<State<bool>>>>) {
    let changed = remember(|| false);
    if let Some(probe) = probe {
        probe.set(Some(changed));
    }
    column().children(move || {
        text(if changed.get() {
            "Updated heading"
        } else {
            "Visible heading"
        })
        .modifier(Modifier::new().semantics(SemanticsArgs {
            test_tag: Some("heading".into()),
            role: Some(Role::Heading),
            heading_level: Some(2),
            label: Some("Caller heading".into()),
            description: Some("Caller description".into()),
            tooltip: Some("Heading tooltip".into()),
            ..Default::default()
        }))
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
        text("Selected option").modifier(Modifier::new().semantics(SemanticsArgs {
            test_tag: Some("option".into()),
            role: Some(Role::ListBoxOption),
            selected: Some(true),
            disabled: true,
            ..Default::default()
        }));
        text(PlainLabel(if changed.get() {
            "Changed text"
        } else {
            "Plain text"
        }))
        .modifier(Modifier::new().semantics(SemanticsArgs {
            test_tag: Some("plain".into()),
            ..Default::default()
        }));
        text("Hidden text").modifier(Modifier::new().semantics(SemanticsArgs {
            test_tag: Some("hidden".into()),
            hidden: true,
            ..Default::default()
        }));
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
    let probe = TestProbe::<Option<State<bool>>>::default();
    let observed = probe.clone();
    let mut renderer = HeadlessRenderer::new(
        Box::new(move || {
            fixture().probe(probe.clone());
        }),
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
        observed.get().unwrap().set(changed);
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
