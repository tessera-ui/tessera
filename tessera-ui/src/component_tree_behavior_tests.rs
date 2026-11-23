use std::collections::BTreeMap;

use crate::{
    AccessibilityActionHandler, AccessibilityNode, ComputedData, Constraint, CursorEventContent,
    FocusDirection, FocusRequester, FocusScopeNode, LayoutPolicy, LayoutResult, Modifier,
    PointerEventPass, PointerInput, PointerInputModifierNode, PressKeyEventType, Px, PxPosition,
    SemanticsModifierNode, State, accesskit::Role, focus, layout::MeasureScope,
    modifier::FocusModifierExt as _, remember, tessera, testing::UiTest,
};

#[derive(Clone, Default, PartialEq)]
struct FixtureObservation {
    version: crate::testing::TestProbe<Option<State<u32>>>,
    leaf_version: crate::testing::TestProbe<Option<State<u32>>>,
    requesters: crate::testing::TestProbe<BTreeMap<usize, FocusRequester>>,
    scopes: crate::testing::TestProbe<Option<(FocusScopeNode, FocusScopeNode)>>,
    reverse: crate::testing::TestProbe<Option<State<bool>>>,
    hits: crate::testing::TestProbe<Vec<usize>>,
    armed: crate::testing::TestProbe<Option<State<bool>>>,
}

#[derive(Clone, PartialEq)]
struct Overlap;

impl LayoutPolicy for Overlap {
    fn measure(
        &self,
        scope: &mut MeasureScope<'_>,
    ) -> Result<LayoutResult, crate::MeasurementError> {
        let mut result = LayoutResult::new(ComputedData {
            width: Px(20),
            height: Px(20),
        });
        for child in scope.children() {
            child.measure(scope, &Constraint::exact(Px(20), Px(20)))?;
            result.place_child(child, PxPosition::ZERO);
        }
        Ok(result)
    }
}

#[derive(Clone, PartialEq)]
struct Tag {
    key: String,
    label: String,
}

impl SemanticsModifierNode for Tag {
    fn apply(&self, node: &mut AccessibilityNode, _: &mut Option<AccessibilityActionHandler>) {
        node.key = Some(self.key.clone());
        node.label = Some(self.label.clone());
        node.role = Some(Role::Button);
        node.focusable = true;
    }
}

struct Pointer(Option<usize>, crate::testing::TestProbe<Vec<usize>>);
impl PointerInputModifierNode for Pointer {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if input.pass != PointerEventPass::Main {
            return;
        }
        for change in input.pointer_changes.iter_mut() {
            if !change.is_consumed() && matches!(change.content, CursorEventContent::Pressed(_)) {
                if let Some(index) = self.0 {
                    self.1.with_mut(|hits| hits.push(index));
                }
                change.consume();
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct RevisionedOverlap(u32);

impl LayoutPolicy for RevisionedOverlap {
    fn measure(
        &self,
        scope: &mut MeasureScope<'_>,
    ) -> Result<LayoutResult, crate::MeasurementError> {
        Overlap.measure(scope)
    }
}

#[tessera]
fn versioned_control(version: u32, probe: FixtureObservation) {
    crate::layout::layout(RevisionedOverlap(version)).modifier(
        Modifier::new()
            .push_semantics(Tag {
                key: "edited".to_owned(),
                label: format!("Version {version}"),
            })
            .push_pointer_input(Pointer(None, probe.hits.clone())),
    );
}

#[tessera]
fn sibling_control(index: usize, probe: FixtureObservation) {
    let requester = remember(FocusRequester::new).get();
    probe.requesters.with_mut(|requesters| {
        requesters.insert(index, requester);
    });
    crate::layout::layout(Overlap).modifier(
        Modifier::new()
            .push_semantics(Tag {
                key: format!("sibling-{index}"),
                label: format!("Sibling {index}"),
            })
            .push_pointer_input(Pointer(Some(index), probe.hits.clone()))
            .focusable()
            .focus_requester(requester),
    );
}

#[tessera]
fn sibling_controls(probe: FixtureObservation) {
    crate::layout::layout(Overlap).child(move || {
        let probe = probe.clone();
        for index in 0..2 {
            let probe = probe.clone();
            crate::key(index, move || {
                sibling_control(index, probe.clone());
            });
        }
    });
}

#[tessera]
fn label_fixture(probe: FixtureObservation) {
    let version = remember(|| 0_u32);
    probe.version.set(Some(version));
    let version = version.get();
    crate::layout::layout(Overlap).child(move || {
        let probe = probe.clone();
        versioned_control(version, probe.clone());
        sibling_controls(probe.clone());
    });
}

#[test]
fn updating_a_label_preserves_sibling_accessibility_and_focus() {
    let probe = FixtureObservation::default();
    let mut session = UiTest::new()
        .content(|| label_fixture(probe.clone()))
        .viewport(100, 100);
    session.frame();
    let (sibling, _) = session.accessibility_node("Sibling 1");
    assert!(session.accessibility_action(sibling, crate::accesskit::Action::Focus));
    session.frames(1);
    assert_eq!(session.accessibility_snapshot().unwrap().focus, sibling);
    probe.version.get().expect("Version state").set(1);
    session.advance(std::time::Duration::from_millis(16));
    assert_eq!(
        semantic_reading_order(&session),
        ["Version 1", "Sibling 0", "Sibling 1"]
    );
    let (sibling, _) = session.accessibility_node("Sibling 1");
    assert_eq!(session.accessibility_snapshot().unwrap().focus, sibling);
}

#[tessera]
fn child_control(
    version: State<u32>,
    parent: State<u32>,
    child_updates_parent: bool,
    probe: FixtureObservation,
) {
    let version = version.get();
    if child_updates_parent && version == 1 {
        parent.set(2);
    }
    crate::layout::layout(Overlap).modifier(
        Modifier::new()
            .push_semantics(Tag {
                key: "nested-child".to_owned(),
                label: format!("Leaf {version}"),
            })
            .push_pointer_input(Pointer(Some(42), probe.hits.clone())),
    );
}

#[tessera]
fn nested_child(
    version: State<u32>,
    parent: State<u32>,
    child_updates_parent: bool,
    probe: FixtureObservation,
) {
    crate::layout::layout(Overlap).child(move || {
        let probe = probe.clone();
        child_control(version, parent, child_updates_parent, probe.clone());
    });
}

#[tessera]
fn nested_state_fixture(
    parent_updates_child: bool,
    child_updates_parent: bool,
    probe: FixtureObservation,
) {
    let parent = remember(|| 0_u32);
    let leaf = remember(|| 0_u32);
    probe.version.set(Some(parent));
    probe.leaf_version.set(Some(leaf));
    let version = parent.get();
    if parent_updates_child && version > 0 {
        leaf.set(version);
    }
    crate::layout::layout(Overlap).child(move || {
        let probe = probe.clone();
        versioned_control(version, probe.clone());
        nested_child(leaf, parent, child_updates_parent, probe.clone());
    });
}

fn check_nested_state_updates(parent_updates_child: bool, child_updates_parent: bool) {
    let probe = FixtureObservation::default();

    let mut session = UiTest::new()
        .content(|| nested_state_fixture(parent_updates_child, child_updates_parent, probe.clone()))
        .viewport(100, 100);
    session.frame();
    probe
        .version
        .get()
        .expect("test fixture entry exists")
        .set(1);
    if !parent_updates_child {
        probe
            .leaf_version
            .get()
            .expect("test fixture entry exists")
            .set(1);
    }
    session.advance(std::time::Duration::from_millis(16));
    let labels = semantic_reading_order(&session);
    assert!(labels.iter().any(|label| label == "Version 1"));
    assert!(labels.iter().any(|label| label == "Leaf 1"));
    let position = PxPosition::from_f32_xy(10.0, 10.0);
    session.pointer_press(crate::MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    session.frames(1);
    assert_eq!(probe.hits.with(Clone::clone), [42]);
    if child_updates_parent {
        let labels = semantic_reading_order(&session);
        assert!(labels.iter().any(|label| label == "Version 2"));
        assert!(labels.iter().any(|label| label == "Leaf 2"));
    }
}

#[test]
fn nested_controls_reflect_independent_state_updates() {
    check_nested_state_updates(false, false);
}

#[test]
fn child_label_reflects_state_set_by_its_parent() {
    check_nested_state_updates(true, false);
}

#[test]
fn child_state_changes_are_visible_to_the_parent_on_the_next_frame() {
    check_nested_state_updates(true, true);
}

#[tessera]
fn ordered_branch(index: usize, probe: FixtureObservation) {
    crate::layout::layout(Overlap).child(move || {
        let probe = probe.clone();
        sibling_control(index, probe.clone());
    });
}

#[tessera]
fn ordered_fixture(probe: FixtureObservation) {
    let reverse = remember(|| false);
    probe.reverse.set(Some(reverse));
    let reverse = reverse.get();
    crate::layout::layout(Overlap).child(move || {
        let probe = probe.clone();
        for index in if reverse { [1_usize, 0] } else { [0_usize, 1] } {
            let probe = probe.clone();
            crate::key(index, move || {
                ordered_branch(index, probe.clone());
            });
        }
    });
}

#[derive(Clone, PartialEq)]
struct PlacementOrder(State<bool>);

impl LayoutPolicy for PlacementOrder {
    fn measure(
        &self,
        scope: &mut MeasureScope<'_>,
    ) -> Result<LayoutResult, crate::MeasurementError> {
        let mut result = LayoutResult::new(ComputedData {
            width: Px(20),
            height: Px(20),
        });
        let children = scope.children();
        for &child in &children {
            child.measure(scope, &Constraint::exact(Px(20), Px(20)))?;
        }
        for index in if self.0.get() { [1, 0] } else { [0, 1] } {
            result.place_child(children[index], PxPosition::ZERO);
        }
        Ok(result)
    }
}

#[tessera]
fn placement_order_fixture(probe: FixtureObservation) {
    let reverse = remember(|| false);
    probe.reverse.set(Some(reverse));
    crate::layout::layout(PlacementOrder(reverse)).child(move || {
        let probe = probe.clone();
        ordered_branch(0, probe.clone());
        ordered_branch(1, probe.clone());
    });
}

#[test]
fn focus_traversal_follows_placement_order() {
    let probe = FixtureObservation::default();
    let mut session = UiTest::new()
        .content(|| placement_order_fixture(probe.clone()))
        .viewport(100, 100);
    session.frame();
    let (first, second) = probe.requesters.with(|items| (items[&0], items[&1]));
    assert!(focus::move_focus(FocusDirection::Next));
    assert!(first.is_focused());
    session.advance(std::time::Duration::from_millis(8));
    probe
        .reverse
        .get()
        .expect("test fixture entry exists")
        .set(true);
    session.advance(std::time::Duration::from_millis(8));
    focus::clear_focus(true);
    assert!(focus::move_focus(FocusDirection::Next));
    assert!(second.is_focused());
}

fn semantic_reading_order(session: &crate::testing::UiTest) -> Vec<String> {
    session
        .inspect_accessibility()
        .expect("Accessibility snapshot")
        .reading_order()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect()
}

#[test]
fn reordering_updates_pointer_targets_reading_order_and_focus_traversal() {
    let probe = FixtureObservation::default();
    let mut session = UiTest::new()
        .content(|| ordered_fixture(probe.clone()))
        .viewport(100, 100);
    session.frame();
    assert_eq!(semantic_reading_order(&session), ["Sibling 0", "Sibling 1"]);
    let (first, second) = probe.requesters.with(|items| (items[&0], items[&1]));
    assert!(focus::move_focus(FocusDirection::Next));
    assert!(first.is_focused());
    session.advance(std::time::Duration::from_millis(8));
    probe.reverse.get().expect("Reverse state").set(true);
    session.advance(std::time::Duration::from_millis(8));
    assert_eq!(semantic_reading_order(&session), ["Sibling 1", "Sibling 0"]);
    focus::clear_focus(true);
    assert!(focus::move_focus(FocusDirection::Next));
    assert!(
        second.is_focused(),
        "Traversal follows the new sibling order"
    );
    let position = PxPosition::from_f32_xy(10.0, 10.0);
    session.pointer_press(crate::MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    session.frames(1);
    assert_eq!(
        probe.hits.with(Clone::clone),
        [0],
        "The new last sibling must be the topmost pointer consumer"
    );
}

#[tessera]
fn optional_pointer_fixture(probe: FixtureObservation) {
    let armed = remember(|| true);
    probe.armed.set(Some(armed));
    let modifier = Modifier::new().push_semantics(Tag {
        key: "optional-pointer".into(),
        label: "Pointer target".into(),
    });
    let modifier = if armed.get() {
        modifier.push_pointer_input(Pointer(Some(0), probe.hits))
    } else {
        modifier
    };
    crate::layout::layout(Overlap).modifier(modifier);
}

#[tessera]
fn overlap_root(probe: FixtureObservation) {
    crate::layout::layout(Overlap).child(move || optional_pointer_fixture(probe.clone()));
}

#[test]
fn a_control_receives_presses_only_while_its_handler_is_attached() {
    let probe = FixtureObservation::default();
    let mut session = UiTest::new()
        .content(|| {
            let probe = probe.clone();
            overlap_root(probe);
        })
        .viewport(100, 100);
    session.frame();
    let position = PxPosition::from_f32_xy(10.0, 10.0);
    let button = PressKeyEventType::Left;
    for (enabled, hits) in [(true, 1), (false, 1), (true, 2)] {
        probe
            .armed
            .get()
            .expect("test fixture entry exists")
            .set(enabled);
        session.pointer_press(crate::MOUSE_POINTER_ID, position, button);
        session.pointer_release(crate::MOUSE_POINTER_ID, position, button);
        session.frames(1);
        assert_eq!(semantic_reading_order(&session), ["Pointer target"]);
        assert_eq!(probe.hits.with(Vec::len), hits);
    }
}

#[derive(Clone, PartialEq)]
struct Hint(String);
impl SemanticsModifierNode for Hint {
    fn apply(&self, node: &mut AccessibilityNode, _: &mut Option<AccessibilityActionHandler>) {
        node.role = Some(Role::Group);
        node.tooltip = Some(self.0.clone());
        node.merge_descendants = true;
    }
}

#[tessera]
fn conditional_description(probe: FixtureObservation) {
    let visible = remember(|| false);
    probe.reverse.set(Some(visible));
    let modifier = if visible.get() {
        Modifier::new()
            .push_semantics(Hint("Help text".into()))
            .push_semantics(Tag {
                key: "conditional-description".into(),
                label: "Help host".into(),
            })
    } else {
        Modifier::new()
    };
    crate::layout::layout(Overlap).modifier(modifier);
}

#[test]
fn toggling_help_adds_and_removes_its_accessible_tooltip() {
    let probe = FixtureObservation::default();
    let mut session = UiTest::new().content(|| conditional_description(probe.clone()));
    session.frame();
    assert!(semantic_reading_order(&session).is_empty());
    let visible = probe.reverse.get().expect("Visibility state");
    visible.set(true);
    session.advance(std::time::Duration::from_millis(16));
    assert_eq!(
        session.accessibility_node("Help host").1.tooltip(),
        Some("Help text")
    );
    visible.set(false);
    session.advance(std::time::Duration::from_millis(16));
    assert!(semantic_reading_order(&session).is_empty());
}

#[tessera]
fn scoped_fixture(probe: FixtureObservation) {
    let changed = remember(|| false);
    probe.reverse.set(Some(changed));
    let changed = changed.get();
    let scopes = remember(|| (FocusScopeNode::new(), FocusScopeNode::new())).get();
    probe.scopes.set(Some(scopes));
    crate::layout::layout(Overlap)
        .modifier(
            Modifier::new()
                .focus_scope(Some(if changed { scopes.1 } else { scopes.0 }))
                .push_semantics(Hint(
                    if changed {
                        "Updated hint"
                    } else {
                        "First hint"
                    }
                    .to_owned(),
                )),
        )
        .child(move || {
            let probe = probe.clone();
            ordered_branch(0, probe.clone());
        });
}

#[test]
fn a_control_uses_its_updated_focus_scope_and_accessible_description() {
    let probe = FixtureObservation::default();
    let mut session = UiTest::new()
        .content(|| scoped_fixture(probe.clone()))
        .viewport(100, 100);
    session.frame();
    let requester = probe.requesters.with(|items| items[&0]);
    let (old_scope, new_scope) = probe.scopes.get().expect("Remembered focus scopes");
    requester.request_focus();
    assert!(old_scope.has_focus());
    assert!(!new_scope.has_focus());
    session.advance(std::time::Duration::from_millis(8));
    assert_eq!(
        session.accessibility_node("Sibling 0").1.description(),
        Some("First hint")
    );
    probe.reverse.get().expect("Changed state").set(true);
    session.advance(std::time::Duration::from_millis(8));
    requester.request_focus();
    assert!(requester.is_focused());
    assert!(
        new_scope.has_focus(),
        "Retained target must attach to the replacement scope"
    );
    assert!(!old_scope.has_focus());
    assert_eq!(
        session.accessibility_node("Sibling 0").1.description(),
        Some("Updated hint"),
        "Sibling description follows its current focus scope"
    );
}

#[tessera]
fn optional_leaf(visible: State<bool>) {
    if visible.get() {
        crate::layout::layout(Overlap).modifier(Modifier::new().push_semantics(Tag {
            key: "optional".into(),
            label: "Optional".into(),
        }));
    }
}

#[tessera]
fn optional_leaf_host(probe: FixtureObservation) {
    let visible = remember(|| true);
    probe.armed.set(Some(visible));
    crate::layout::layout(Overlap).child(move || optional_leaf(visible));
}

#[test]
fn a_call_that_stops_emitting_its_node_still_observes_its_state() {
    let probe = FixtureObservation::default();
    let mut session = UiTest::new()
        .content(|| optional_leaf_host(probe.clone()))
        .viewport(100, 100);
    session.frame();
    let visible = probe.armed.get().expect("visibility state");
    for shown in [false, true, false, true] {
        visible.set(shown);
        session.advance(std::time::Duration::from_millis(8));
        assert_eq!(
            semantic_reading_order(&session),
            if shown {
                vec!["Optional"]
            } else {
                Vec::<&str>::new()
            },
        );
    }
}

#[tessera]
fn tagged_leaf(label: String) {
    crate::layout::layout(Overlap).modifier(Modifier::new().push_semantics(Tag {
        key: label.clone(),
        label,
    }));
}

#[tessera]
fn leading_content_host(probe: FixtureObservation) {
    let version = remember(|| 0_u32);
    probe.version.set(Some(version));
    crate::layout::layout(Overlap).child(move || {
        tagged_leaf("Before".to_owned());
        crate::layout::layout(RevisionedOverlap(version.get())).child(|| {
            tagged_leaf("Inside".to_owned());
        });
    });
}

#[test]
fn updating_nested_content_preserves_reading_order() {
    let probe = FixtureObservation::default();
    let mut session = UiTest::new()
        .content(|| leading_content_host(probe.clone()))
        .viewport(100, 100);
    session.frame();
    let version = probe.version.get().expect("version state");
    for revision in 1..3 {
        version.set(revision);
        session.advance(std::time::Duration::from_millis(8));
        assert_eq!(semantic_reading_order(&session), ["Before", "Inside"]);
    }
}
