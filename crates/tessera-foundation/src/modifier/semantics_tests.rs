use tessera_ui::{
    Dp, Modifier,
    accesskit::Role,
    layout::{Stack, layout},
    tessera,
    testing::UiTest,
};

use crate::modifier::{ModifierExt as _, SemanticsArgs};

#[derive(Clone, Copy, PartialEq)]
enum Container {
    Tagged,
    Cleared,
    ClearedThenTagged,
}

fn tag() -> SemanticsArgs {
    SemanticsArgs {
        test_tag: Some("container".into()),
        ..Default::default()
    }
}

#[tessera]
fn fixture(container: Container) {
    let modifier = Modifier::new().size(Dp(80.0), Dp(40.0));
    let modifier = match container {
        Container::Tagged => modifier.semantics(tag()),
        Container::Cleared => modifier.clear_and_set_semantics(tag()),
        Container::ClearedThenTagged => modifier.clear_and_set_semantics(tag()).semantics(tag()),
    };
    layout(Stack).modifier(modifier).child(|| {
        layout(Stack).modifier(
            Modifier::new()
                .size(Dp(40.0), Dp(20.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::Button),
                    label: Some("Child".into()),
                    ..Default::default()
                }),
        );
    });
}

fn child_published(container: Container) -> bool {
    let mut ui = UiTest::new()
        .viewport(100, 60)
        .content(move || fixture(container));
    ui.frame();
    ui.inspect_accessibility()
        .expect("published semantics")
        .find("Child", Role::Button)
        .is_some()
}

#[test]
fn semantics_keep_descendants_and_only_clear_and_set_hides_them() {
    assert!(child_published(Container::Tagged));
    assert!(!child_published(Container::Cleared));
    assert!(!child_published(Container::ClearedThenTagged));
}
