use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
};

use crate::{
    AccessibilityActionHandler, AccessibilityNode, ComputedData, Constraint, CursorEventContent,
    FocusDirection, FocusManager, FocusModifierExt, FocusRequester, FocusScopeNode, GestureState,
    LayoutModifierNode, LayoutResult, Modifier, PointerChange, PointerEventPass, PointerInput,
    PointerInputModifierNode, PressKeyEventType, Px, PxPosition, SemanticsModifierNode, State,
    accesskit::Role,
    layout::{MeasureScope, layout},
    remember,
    runtime::TesseraRuntime,
    tessera,
    testing::{
        last_layout_frame_diagnostics, layout_test, pump_layout_test_session,
        pump_layout_test_session_with_input, start_layout_test_session,
    },
    time::Instant,
};

thread_local! {
    static VERSION: Cell<Option<State<u32>>> = const { Cell::new(None) };
    static LEAF_VERSION: Cell<Option<State<u32>>> = const { Cell::new(None) };
    static PARENT_BUILDS: Cell<usize> = const { Cell::new(0) };
    static REQUESTERS: RefCell<BTreeMap<usize, FocusRequester>> = const { RefCell::new(BTreeMap::new()) };
    static SCOPES: Cell<Option<(FocusScopeNode, FocusScopeNode)>> = const { Cell::new(None) };
    static REVERSE: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static HITS: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
    static RETAINED_BUILDS: Cell<usize> = const { Cell::new(0) };
}

#[derive(Clone, PartialEq)]
struct Overlap;

impl LayoutModifierNode for Overlap {
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

struct Pointer(Option<usize>);
impl PointerInputModifierNode for Pointer {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if input.pass != PointerEventPass::Main {
            return;
        }
        for change in input.pointer_changes.iter_mut() {
            if !change.is_consumed() && matches!(change.content, CursorEventContent::Pressed(_)) {
                if let Some(index) = self.0 {
                    HITS.with_borrow_mut(|hits| hits.push(index));
                }
                change.consume();
            }
        }
    }
}

#[tessera(crate)]
fn edited_leaf(version: u32) {
    layout().policy(Overlap).modifier(
        Modifier::new()
            .push_semantics(Tag {
                key: "edited".to_owned(),
                label: format!("Version {version}"),
            })
            .push_pointer_input(Pointer(None)),
    );
}

#[tessera(crate)]
fn retained_leaf(index: usize) {
    let requester = remember(FocusRequester::new).get();
    REQUESTERS.with_borrow_mut(|requesters| {
        requesters.insert(index, requester);
    });
    layout().policy(Overlap).modifier(
        Modifier::new()
            .push_semantics(Tag {
                key: format!("retained-{index}"),
                label: format!("Retained {index}"),
            })
            .push_pointer_input(Pointer(Some(index)))
            .focusable()
            .focus_requester(requester),
    );
}

#[tessera(crate)]
fn retained_branch(count: usize) {
    RETAINED_BUILDS.set(RETAINED_BUILDS.get() + 1);
    layout().policy(Overlap).child(move || {
        for index in 0..count {
            crate::key(index, move || {
                retained_leaf(index);
            });
        }
    });
}

#[tessera(crate)]
fn replay_fixture(count: usize) {
    let version = remember(|| 0_u32);
    VERSION.set(Some(version));
    let version = version.get();
    layout().policy(Overlap).child(move || {
        edited_leaf(version);
        retained_branch(count);
    });
}

fn retained_node_ids() -> BTreeMap<String, crate::NodeId> {
    TesseraRuntime::with(|runtime| {
        let mut result = BTreeMap::new();
        runtime.component_tree.metadatas().with_entries(|entries| {
            for (&id, metadata) in entries {
                if let Some(key) = metadata
                    .accessibility
                    .as_ref()
                    .and_then(|node| node.key.as_ref())
                    .filter(|key| key.starts_with("retained-"))
                {
                    result.insert(key.clone(), id);
                }
            }
        });
        result
    })
}

fn update_one_leaf(count: usize) -> (super::LayoutFrameDiagnostics, super::LayoutFrameDiagnostics) {
    RETAINED_BUILDS.set(0);
    let mut session = start_layout_test_session(
        layout_test(move || {
            replay_fixture(count);
        })
        .viewport_px(100, 100),
    );
    pump_layout_test_session(&mut session, 0);
    let initial = last_layout_frame_diagnostics().expect("Initial frame diagnostics");
    let retained = retained_node_ids();
    assert_eq!(retained.len(), count);
    assert_eq!(RETAINED_BUILDS.get(), 1);
    VERSION.get().expect("Version state").set(1);
    pump_layout_test_session(&mut session, 16_000_000);
    let updated = last_layout_frame_diagnostics().expect("Updated frame diagnostics");
    assert_eq!(
        RETAINED_BUILDS.get(),
        1,
        "The equal-prop sibling must really reuse its component subtree"
    );
    assert_eq!(
        retained_node_ids(),
        retained,
        "Retained control identities must survive the production replay"
    );
    let update = TesseraRuntime::with_mut(|runtime| {
        runtime
            .component_tree
            .accessibility_update(None, true)
            .expect("Accessibility snapshot")
    });
    assert!(
        update
            .nodes
            .iter()
            .any(|(_, node)| node.label() == Some("Version 1"))
    );
    assert_eq!(
        update
            .nodes
            .iter()
            .filter(|(_, node)| node
                .label()
                .is_some_and(|label| label.starts_with("Retained ")))
            .count(),
        count
    );
    (initial, updated)
}

#[test]
fn replay_consumer_work_does_not_grow_with_an_unchanged_retained_sibling() {
    let (small_initial, small_update) = update_one_leaf(4);
    let (large_initial, large_update) = update_one_leaf(64);
    assert!(
        large_initial.pointer_nodes_updated > small_initial.pointer_nodes_updated,
        "The first publication must visit the larger actual fixture"
    );
    assert!(large_initial.accessibility_nodes_updated > small_initial.accessibility_nodes_updated);
    assert!(small_update.pointer_nodes_updated > 0);
    assert!(small_update.accessibility_nodes_updated > 0);
    assert!(
        large_update.pointer_nodes_updated <= small_update.pointer_nodes_updated,
        "Updating the same leaf must not republish unrelated pointer descendants: small={}, large={}",
        small_update.pointer_nodes_updated,
        large_update.pointer_nodes_updated
    );
    assert!(
        large_update.accessibility_nodes_updated <= small_update.accessibility_nodes_updated,
        "Updating the same leaf must not republish unrelated semantic descendants: small={}, large={}",
        small_update.accessibility_nodes_updated,
        large_update.accessibility_nodes_updated
    );
}

#[tessera(crate)]
fn independently_dirty_leaf(version: State<u32>, parent: State<u32>, backwards_write: bool) {
    let version = version.get();
    if backwards_write && version == 1 {
        parent.set(2);
    }
    layout().policy(Overlap).modifier(
        Modifier::new()
            .push_semantics(Tag {
                key: "retained-dirty-leaf".to_owned(),
                label: format!("Leaf {version}"),
            })
            .push_pointer_input(Pointer(Some(42))),
    );
}

#[tessera(crate)]
fn retained_dirty_branch(version: State<u32>, parent: State<u32>, backwards_write: bool) {
    RETAINED_BUILDS.set(RETAINED_BUILDS.get() + 1);
    layout().policy(Overlap).child(move || {
        independently_dirty_leaf(version, parent, backwards_write);
    });
}

#[tessera(crate)]
fn nested_dirty_fixture(write_during_replay: bool, backwards_write: bool) {
    PARENT_BUILDS.set(PARENT_BUILDS.get() + 1);
    let parent = remember(|| 0_u32);
    let leaf = remember(|| 0_u32);
    VERSION.set(Some(parent));
    LEAF_VERSION.set(Some(leaf));
    let version = parent.get();
    if write_during_replay && version > 0 {
        leaf.set(version);
    }
    layout().policy(Overlap).child(move || {
        edited_leaf(version);
        retained_dirty_branch(leaf, parent, backwards_write);
    });
}

fn check_independent_nested_replay(write_during_replay: bool, backwards_write: bool) {
    RETAINED_BUILDS.set(0);
    PARENT_BUILDS.set(0);
    HITS.with_borrow_mut(Vec::clear);
    let mut session = start_layout_test_session(
        layout_test(move || {
            nested_dirty_fixture(write_during_replay, backwards_write);
        })
        .viewport_px(100, 100),
    );
    pump_layout_test_session(&mut session, 0);
    let retained = TesseraRuntime::with(|runtime| {
        let key = runtime
            .component_tree
            .tree()
            .iter()
            .find(|node| !node.is_removed() && node.get().fn_name == "retained_dirty_branch")
            .unwrap()
            .get()
            .instance_key;
        (
            key,
            runtime
                .component_tree
                .find_node_id_by_instance_key(key)
                .unwrap(),
        )
    });
    assert_eq!(RETAINED_BUILDS.get(), 1);
    VERSION.get().unwrap().set(1);
    if !write_during_replay {
        LEAF_VERSION.get().unwrap().set(1);
    }
    pump_layout_test_session(&mut session, 16_000_000);
    assert_eq!(
        RETAINED_BUILDS.get(),
        1,
        "A dirty descendant must replay independently of its equal-prop ancestor"
    );
    assert_eq!(
        TesseraRuntime::with(|runtime| {
            runtime
                .component_tree
                .find_node_id_by_instance_key(retained.0)
        }),
        Some(retained.1)
    );
    let labels = semantic_reading_order();
    assert!(labels.iter().any(|label| label == "Version 1"));
    assert!(labels.iter().any(|label| label == "Leaf 1"));
    assert_eq!(PARENT_BUILDS.get(), 2);
    let position = PxPosition::from_f32_xy(10.0, 10.0);
    let press = PointerChange::new(
        Instant::now(),
        crate::MOUSE_POINTER_ID,
        CursorEventContent::Pressed(PressKeyEventType::Left),
        GestureState::TapCandidate,
    )
    .with_position(position);
    pump_layout_test_session_with_input(&mut session, 32_000_000, vec![press], Some(position));
    assert_eq!(HITS.with_borrow(Clone::clone), [42]);
    if backwards_write {
        let labels = semantic_reading_order();
        assert!(labels.iter().any(|label| label == "Version 2"));
        assert!(labels.iter().any(|label| label == "Leaf 2"));
        assert_eq!(PARENT_BUILDS.get(), 3);
        assert_eq!(RETAINED_BUILDS.get(), 1);
    }
}

#[test]
fn nested_dirty_leaf_replays_without_rebuilding_its_equal_prop_ancestor() {
    check_independent_nested_replay(false, false);
}

#[test]
fn invalidation_created_during_parent_replay_reaches_a_retained_descendant() {
    check_independent_nested_replay(true, false);
}

#[test]
fn a_later_replay_round_defers_writes_to_an_already_executed_boundary() {
    check_independent_nested_replay(true, true);
}

thread_local! {
    static CYCLIC_REPLAY_ARMED: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static CYCLIC_REPLAY_BUILDS: Cell<[usize; 2]> = const { Cell::new([0; 2]) };
}

#[tessera(crate)]
fn cyclic_replay_peer(index: usize, own: State<u32>, other: State<u32>, armed: State<bool>) {
    let mut builds = CYCLIC_REPLAY_BUILDS.get();
    builds[index] += 1;
    CYCLIC_REPLAY_BUILDS.set(builds);
    let value = own.get();
    if armed.get() {
        other.set(value + 1);
    }
    layout()
        .policy(Overlap)
        .modifier(Modifier::new().push_semantics(Tag {
            key: format!("cyclic-{index}"),
            label: format!("Cycle {index}"),
        }));
}

#[tessera(crate)]
fn cyclic_replay_fixture() {
    let left = remember(|| 0_u32);
    let right = remember(|| 0_u32);
    let armed = remember(|| false);
    CYCLIC_REPLAY_ARMED.set(Some(armed));
    layout().policy(Overlap).child(move || {
        cyclic_replay_peer(0, left, right, armed);
        cyclic_replay_peer(1, right, left, armed);
    });
}

#[test]
fn cyclic_writers_execute_once_per_frame_and_keep_their_next_frame_work() {
    CYCLIC_REPLAY_BUILDS.set([0; 2]);
    let mut session =
        start_layout_test_session(layout_test(|| cyclic_replay_fixture()).viewport_px(100, 100));
    pump_layout_test_session(&mut session, 0);
    assert_eq!(CYCLIC_REPLAY_BUILDS.get(), [1, 1]);
    CYCLIC_REPLAY_ARMED.get().unwrap().set(true);
    for frame in 1..=3 {
        pump_layout_test_session(&mut session, frame * 16_666_667);
        assert_eq!(CYCLIC_REPLAY_BUILDS.get(), [frame as usize + 1; 2]);
        assert!(crate::runtime::has_pending_build_invalidations());
        let labels = semantic_reading_order();
        assert!(labels.iter().any(|label| label == "Cycle 0"));
        assert!(labels.iter().any(|label| label == "Cycle 1"));
    }
}

#[tessera(crate)]
fn ordered_branch(index: usize) {
    RETAINED_BUILDS.set(RETAINED_BUILDS.get() + 1);
    layout().policy(Overlap).child(move || {
        retained_leaf(index);
    });
}

#[tessera(crate)]
fn ordered_fixture() {
    let reverse = remember(|| false);
    REVERSE.set(Some(reverse));
    let reverse = reverse.get();
    layout().policy(Overlap).child(move || {
        for index in if reverse { [1_usize, 0] } else { [0_usize, 1] } {
            crate::key(index, move || {
                ordered_branch(index);
            });
        }
    });
}

#[derive(Clone, PartialEq)]
struct PlacementOrder(State<bool>);

impl LayoutModifierNode for PlacementOrder {
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

#[tessera(crate)]
fn placement_order_fixture() {
    let reverse = remember(|| false);
    REVERSE.set(Some(reverse));
    layout().policy(PlacementOrder(reverse)).child(|| {
        ordered_branch(0);
        ordered_branch(1);
    });
}

#[test]
fn placement_order_changes_update_retained_descendant_focus_traversal_without_recomposition() {
    RETAINED_BUILDS.set(0);
    REQUESTERS.with_borrow_mut(BTreeMap::clear);
    let mut session =
        start_layout_test_session(layout_test(placement_order_fixture).viewport_px(100, 100));
    pump_layout_test_session(&mut session, 0);
    let (first, second) = REQUESTERS.with_borrow(|items| (items[&0], items[&1]));
    assert!(FocusManager::current().move_focus(FocusDirection::Next));
    assert!(first.is_focused());
    pump_layout_test_session(&mut session, 8_000_000);
    let before = RETAINED_BUILDS.get();
    REVERSE.get().unwrap().set(true);
    pump_layout_test_session(&mut session, 16_000_000);
    assert_eq!(RETAINED_BUILDS.get(), before);
    FocusManager::current().clear_focus(true);
    assert!(FocusManager::current().move_focus(FocusDirection::Next));
    assert!(second.is_focused());
}

fn semantic_reading_order() -> Vec<String> {
    let update = TesseraRuntime::with_mut(|runtime| {
        runtime
            .component_tree
            .accessibility_update(None, true)
            .expect("Accessibility snapshot")
    });
    let nodes = update
        .nodes
        .iter()
        .map(|(id, node)| (*id, node))
        .collect::<BTreeMap<_, _>>();
    fn collect(
        id: crate::accesskit::NodeId,
        nodes: &BTreeMap<crate::accesskit::NodeId, &crate::accesskit::Node>,
        labels: &mut Vec<String>,
    ) {
        let node = nodes[&id];
        if let Some(label) = node.label() {
            labels.push(label.to_owned());
        }
        for &child in node.children() {
            collect(child, nodes, labels);
        }
    }
    let mut labels = Vec::new();
    collect(
        update.tree.expect("Published root").root,
        &nodes,
        &mut labels,
    );
    labels
}

#[test]
fn replay_reorders_retained_pointer_targets_semantics_and_focus_traversal() {
    RETAINED_BUILDS.set(0);
    REQUESTERS.with_borrow_mut(BTreeMap::clear);
    HITS.with_borrow_mut(Vec::clear);
    let mut session = start_layout_test_session(
        layout_test(|| {
            ordered_fixture();
        })
        .viewport_px(100, 100),
    );
    pump_layout_test_session(&mut session, 0);
    assert_eq!(RETAINED_BUILDS.get(), 2);
    assert_eq!(semantic_reading_order(), ["Retained 0", "Retained 1"]);
    let (first, second) = REQUESTERS.with_borrow(|items| (items[&0], items[&1]));
    assert!(FocusManager::current().move_focus(FocusDirection::Next));
    assert!(first.is_focused());
    // Focus changes invalidate their host replay boundary independently of
    // reorder.
    pump_layout_test_session(&mut session, 8_000_000);
    let retained = retained_node_ids();
    let stabilized_builds = RETAINED_BUILDS.get();
    REVERSE.get().expect("Reverse state").set(true);
    pump_layout_test_session(&mut session, 16_000_000);
    assert_eq!(
        RETAINED_BUILDS.get(),
        stabilized_builds,
        "Both branch bodies must really be retained during reorder"
    );
    assert_eq!(retained_node_ids(), retained);
    assert_eq!(semantic_reading_order(), ["Retained 1", "Retained 0"]);
    FocusManager::current().clear_focus(true);
    assert!(FocusManager::current().move_focus(FocusDirection::Next));
    assert!(
        second.is_focused(),
        "Traversal must follow the new retained sibling order"
    );
    let position = PxPosition::from_f32_xy(10.0, 10.0);
    let press = PointerChange::new(
        Instant::now(),
        crate::MOUSE_POINTER_ID,
        CursorEventContent::Pressed(PressKeyEventType::Left),
        GestureState::TapCandidate,
    )
    .with_position(position);
    pump_layout_test_session_with_input(&mut session, 32_000_000, vec![press], Some(position));
    assert_eq!(
        HITS.with_borrow(Clone::clone),
        [0],
        "The new last sibling must be the topmost pointer consumer"
    );
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

#[tessera(crate)]
fn scoped_fixture() {
    let changed = remember(|| false);
    REVERSE.set(Some(changed));
    let changed = changed.get();
    let scopes = remember(|| (FocusScopeNode::new(), FocusScopeNode::new())).get();
    SCOPES.set(Some(scopes));
    layout()
        .policy(Overlap)
        .modifier(
            Modifier::new()
                .focus_scope_with(if changed { scopes.1 } else { scopes.0 })
                .push_semantics(Hint(
                    if changed {
                        "Updated hint"
                    } else {
                        "First hint"
                    }
                    .to_owned(),
                )),
        )
        .child(|| {
            ordered_branch(0);
        });
}

#[test]
fn replay_refreshes_retained_descendants_when_semantic_and_focus_ancestry_changes() {
    RETAINED_BUILDS.set(0);
    REQUESTERS.with_borrow_mut(BTreeMap::clear);
    let mut session = start_layout_test_session(
        layout_test(|| {
            scoped_fixture();
        })
        .viewport_px(100, 100),
    );
    pump_layout_test_session(&mut session, 0);
    let requester = REQUESTERS.with_borrow(|items| items[&0]);
    let (old_scope, new_scope) = SCOPES.get().expect("Remembered focus scopes");
    requester.request_focus();
    assert!(old_scope.has_focus());
    assert!(!new_scope.has_focus());
    // Drain the focus host invalidation before changing its ancestor context.
    pump_layout_test_session(&mut session, 8_000_000);
    let retained = retained_node_ids();
    let stabilized_builds = RETAINED_BUILDS.get();
    REVERSE.get().expect("Changed state").set(true);
    pump_layout_test_session(&mut session, 16_000_000);
    assert_eq!(
        RETAINED_BUILDS.get(),
        stabilized_builds,
        "The descendant must really reuse its component subtree during ancestry change"
    );
    assert_eq!(retained_node_ids(), retained);
    requester.request_focus();
    assert!(requester.is_focused());
    assert!(
        new_scope.has_focus(),
        "Retained target must attach to the replacement scope"
    );
    assert!(!old_scope.has_focus());
    let update = TesseraRuntime::with_mut(|runtime| {
        runtime
            .component_tree
            .accessibility_update(None, true)
            .expect("Accessibility snapshot")
    });
    let leaf = update
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some("Retained 0"))
        .expect("Retained semantic control");
    assert_eq!(
        leaf.1.description(),
        Some("Updated hint"),
        "Retained semantics must refresh inherited tooltip context"
    );
}
