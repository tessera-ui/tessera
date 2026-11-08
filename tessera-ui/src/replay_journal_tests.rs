use std::collections::BTreeMap;

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
        layout_test, pump_layout_test_session, pump_layout_test_session_with_input,
        start_layout_test_session,
    },
    time::Instant,
};

#[derive(Clone, Default, PartialEq)]
struct ReplayProbe {
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

impl LayoutModifierNode for RevisionedOverlap {
    fn measure(
        &self,
        scope: &mut MeasureScope<'_>,
    ) -> Result<LayoutResult, crate::MeasurementError> {
        Overlap.measure(scope)
    }
}

#[tessera(crate)]
fn edited_leaf(version: u32, probe: ReplayProbe) {
    layout().policy(RevisionedOverlap(version)).modifier(
        Modifier::new()
            .push_semantics(Tag {
                key: "edited".to_owned(),
                label: format!("Version {version}"),
            })
            .push_pointer_input(Pointer(None, probe.hits.clone())),
    );
}

#[tessera(crate)]
fn retained_leaf(index: usize, probe: ReplayProbe) {
    let requester = remember(FocusRequester::new).get();
    probe.requesters.with_mut(|requesters| {
        requesters.insert(index, requester);
    });
    layout().policy(Overlap).modifier(
        Modifier::new()
            .push_semantics(Tag {
                key: format!("retained-{index}"),
                label: format!("Retained {index}"),
            })
            .push_pointer_input(Pointer(Some(index), probe.hits.clone()))
            .focusable()
            .focus_requester(requester),
    );
}

#[tessera(crate)]
fn retained_branch(count: usize, probe: ReplayProbe) {
    layout().policy(Overlap).child(move || {
        let probe = probe.clone();
        for index in 0..count {
            let probe = probe.clone();
            crate::key(index, move || {
                retained_leaf(index, probe.clone());
            });
        }
    });
}

#[tessera(crate)]
fn replay_fixture(count: usize, probe: ReplayProbe) {
    let version = remember(|| 0_u32);
    probe.version.set(Some(version));
    let version = version.get();
    layout().policy(Overlap).child(move || {
        let probe = probe.clone();
        edited_leaf(version, probe.clone());
        retained_branch(count, probe.clone());
    });
}

fn retained_node_ids(
    session: &crate::testing::LayoutTestSession<impl Fn()>,
) -> BTreeMap<String, crate::NodeId> {
    session
        .inspect_components()
        .nodes()
        .iter()
        .filter_map(|node| {
            let key = node.semantics()?.key.as_ref()?.clone();
            key.starts_with("retained-").then(|| (key, node.node_id()))
        })
        .collect()
}

fn update_one_leaf(count: usize) -> (super::LayoutFrameDiagnostics, super::LayoutFrameDiagnostics) {
    let probe = ReplayProbe::default();

    let mut session = start_layout_test_session(
        layout_test({
            let probe = probe.clone();
            move || replay_fixture(count, probe.clone())
        })
        .viewport_px(100, 100),
    );
    pump_layout_test_session(&mut session, 0);
    let initial = session.diagnostics().expect("Initial frame diagnostics");
    let edited_node = |session: &crate::testing::LayoutTestSession<_>| {
        session
            .inspect_components()
            .nodes()
            .iter()
            .find(|node| {
                node.semantics()
                    .and_then(|semantics| semantics.key.as_deref())
                    == Some("edited")
            })
            .expect("edited layout")
            .node_id()
    };
    let edited = edited_node(&session);
    let retained = retained_node_ids(&session);
    assert_eq!(retained.len(), count);
    assert_eq!(
        session
            .work_totals()
            .count("retained_branch", crate::WorkPhase::Build),
        1
    );
    probe.version.get().expect("Version state").set(1);
    pump_layout_test_session(&mut session, 16_000_000);
    let updated = session.diagnostics().expect("Updated frame diagnostics");
    assert_eq!(updated.cache_miss_dirty_self, 1);
    assert_eq!(updated.cache_store_count, 0);
    assert_eq!(updated.geometry_nodes_updated, 0);
    assert_eq!(
        edited_node(&session),
        edited,
        "replay must update the existing layout leaf"
    );
    assert_eq!(
        session
            .work_totals()
            .count("retained_branch", crate::WorkPhase::Build),
        1,
        "The equal-prop sibling must really reuse its component subtree"
    );
    assert_eq!(
        retained_node_ids(&session),
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
fn independently_dirty_leaf(
    version: State<u32>,
    parent: State<u32>,
    backwards_write: bool,
    probe: ReplayProbe,
) {
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
            .push_pointer_input(Pointer(Some(42), probe.hits.clone())),
    );
}

#[tessera(crate)]
fn retained_dirty_branch(
    version: State<u32>,
    parent: State<u32>,
    backwards_write: bool,
    probe: ReplayProbe,
) {
    layout().policy(Overlap).child(move || {
        let probe = probe.clone();
        independently_dirty_leaf(version, parent, backwards_write, probe.clone());
    });
}

#[tessera(crate)]
fn nested_dirty_fixture(write_during_replay: bool, backwards_write: bool, probe: ReplayProbe) {
    let parent = remember(|| 0_u32);
    let leaf = remember(|| 0_u32);
    probe.version.set(Some(parent));
    probe.leaf_version.set(Some(leaf));
    let version = parent.get();
    if write_during_replay && version > 0 {
        leaf.set(version);
    }
    layout().policy(Overlap).child(move || {
        let probe = probe.clone();
        edited_leaf(version, probe.clone());
        retained_dirty_branch(leaf, parent, backwards_write, probe.clone());
    });
}

fn check_independent_nested_replay(write_during_replay: bool, backwards_write: bool) {
    let probe = ReplayProbe::default();

    probe.hits.with_mut(Vec::clear);
    let mut session = start_layout_test_session(
        layout_test({
            let probe = probe.clone();
            move || nested_dirty_fixture(write_during_replay, backwards_write, probe.clone())
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
    assert_eq!(
        session
            .work_totals()
            .count("retained_dirty_branch", crate::WorkPhase::Build),
        1
    );
    probe.version.get().unwrap().set(1);
    if !write_during_replay {
        probe.leaf_version.get().unwrap().set(1);
    }
    pump_layout_test_session(&mut session, 16_000_000);
    assert_eq!(
        session
            .work_totals()
            .count("retained_dirty_branch", crate::WorkPhase::Build),
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
    let labels = semantic_reading_order(&session);
    assert!(labels.iter().any(|label| label == "Version 1"));
    assert!(labels.iter().any(|label| label == "Leaf 1"));
    assert_eq!(
        session
            .work_totals()
            .count("nested_dirty_fixture", crate::WorkPhase::Build),
        2
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
    assert_eq!(probe.hits.with(Clone::clone), [42]);
    if backwards_write {
        let labels = semantic_reading_order(&session);
        assert!(labels.iter().any(|label| label == "Version 2"));
        assert!(labels.iter().any(|label| label == "Leaf 2"));
        assert_eq!(
            session
                .work_totals()
                .count("nested_dirty_fixture", crate::WorkPhase::Build),
            3
        );
        assert_eq!(
            session
                .work_totals()
                .count("retained_dirty_branch", crate::WorkPhase::Build),
            1
        );
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

#[test]
fn a_stale_invalidation_does_not_block_live_replay() {
    let probe = ReplayProbe::default();
    let mut session = start_layout_test_session(
        layout_test(|| replay_fixture(1, probe.clone())).viewport_px(100, 100),
    );
    pump_layout_test_session(&mut session, 0);
    let retained = retained_node_ids(&session);
    assert!(!retained.is_empty());

    crate::runtime::record_replay_boundary_invalidation_for_instance_key(u64::MAX);
    probe.version.get().unwrap().set(1);
    pump_layout_test_session(&mut session, 16_666_667);

    assert!(
        semantic_reading_order(&session)
            .iter()
            .any(|label| label == "Version 1")
    );
    assert_eq!(retained_node_ids(&session), retained);
}

#[tessera(crate)]
fn cyclic_replay_peer(index: usize, own: State<u32>, other: State<u32>, armed: State<bool>) {
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
fn cyclic_replay_fixture(probe: ReplayProbe) {
    let left = remember(|| 0_u32);
    let right = remember(|| 0_u32);
    let armed = remember(|| false);
    probe.armed.set(Some(armed));
    layout().policy(Overlap).child(move || {
        cyclic_replay_peer(0, left, right, armed);
        cyclic_replay_peer(1, right, left, armed);
    });
}

#[test]
fn cyclic_writers_execute_once_per_frame_and_keep_their_next_frame_work() {
    let probe = ReplayProbe::default();
    let mut session = start_layout_test_session(
        layout_test(|| cyclic_replay_fixture(probe.clone())).viewport_px(100, 100),
    );
    pump_layout_test_session(&mut session, 0);
    let peers: Vec<_> = session
        .work()
        .executions()
        .iter()
        .filter(|entry| {
            entry.phase == crate::WorkPhase::Build && entry.target == "cyclic_replay_peer"
        })
        .map(|entry| entry.instance)
        .collect();
    assert_eq!(peers.len(), 2);
    assert_ne!(peers[0], peers[1]);
    probe.armed.get().unwrap().set(true);
    for frame in 1..=3 {
        pump_layout_test_session(&mut session, frame * 16_666_667);
        assert_eq!(
            session
                .work()
                .count("cyclic_replay_peer", crate::WorkPhase::Build),
            2
        );
        for &peer in &peers {
            assert_eq!(
                session.work().for_instance(peer, crate::WorkPhase::Build),
                1
            );
        }
        assert!(crate::runtime::has_pending_build_invalidations());
        let labels = semantic_reading_order(&session);
        assert!(labels.iter().any(|label| label == "Cycle 0"));
        assert!(labels.iter().any(|label| label == "Cycle 1"));
    }
}

#[tessera(crate)]
fn ordered_branch(index: usize, probe: ReplayProbe) {
    layout().policy(Overlap).child(move || {
        let probe = probe.clone();
        retained_leaf(index, probe.clone());
    });
}

#[tessera(crate)]
fn ordered_fixture(probe: ReplayProbe) {
    let reverse = remember(|| false);
    probe.reverse.set(Some(reverse));
    let reverse = reverse.get();
    layout().policy(Overlap).child(move || {
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
fn placement_order_fixture(probe: ReplayProbe) {
    let reverse = remember(|| false);
    probe.reverse.set(Some(reverse));
    layout().policy(PlacementOrder(reverse)).child(move || {
        let probe = probe.clone();
        ordered_branch(0, probe.clone());
        ordered_branch(1, probe.clone());
    });
}

#[test]
fn placement_order_changes_update_retained_descendant_focus_traversal_without_recomposition() {
    let probe = ReplayProbe::default();

    probe.requesters.with_mut(BTreeMap::clear);
    let mut session = start_layout_test_session(
        layout_test(|| placement_order_fixture(probe.clone())).viewport_px(100, 100),
    );
    pump_layout_test_session(&mut session, 0);
    let (first, second) = probe.requesters.with(|items| (items[&0], items[&1]));
    assert!(FocusManager::current().move_focus(FocusDirection::Next));
    assert!(first.is_focused());
    pump_layout_test_session(&mut session, 8_000_000);
    let before = session
        .work_totals()
        .count("ordered_branch", crate::WorkPhase::Build);
    probe.reverse.get().unwrap().set(true);
    pump_layout_test_session(&mut session, 16_000_000);
    assert_eq!(
        session
            .work_totals()
            .count("ordered_branch", crate::WorkPhase::Build),
        before
    );
    FocusManager::current().clear_focus(true);
    assert!(FocusManager::current().move_focus(FocusDirection::Next));
    assert!(second.is_focused());
}

fn semantic_reading_order(session: &crate::testing::LayoutTestSession<impl Fn()>) -> Vec<String> {
    session
        .inspect_accessibility()
        .expect("Accessibility snapshot")
        .reading_order()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect()
}

#[test]
fn replay_reorders_retained_pointer_targets_semantics_and_focus_traversal() {
    let probe = ReplayProbe::default();

    probe.requesters.with_mut(BTreeMap::clear);
    probe.hits.with_mut(Vec::clear);
    let mut session = start_layout_test_session(
        layout_test(|| {
            ordered_fixture(probe.clone());
        })
        .viewport_px(100, 100),
    );
    pump_layout_test_session(&mut session, 0);
    assert_eq!(
        session
            .work_totals()
            .count("ordered_branch", crate::WorkPhase::Build),
        2
    );
    assert_eq!(
        semantic_reading_order(&session),
        ["Retained 0", "Retained 1"]
    );
    let (first, second) = probe.requesters.with(|items| (items[&0], items[&1]));
    assert!(FocusManager::current().move_focus(FocusDirection::Next));
    assert!(first.is_focused());
    // Focus changes invalidate their host replay boundary independently of
    // reorder.
    pump_layout_test_session(&mut session, 8_000_000);
    let retained = retained_node_ids(&session);
    let snapshot = session.inspect_components();
    let retained_instances =
        ["retained-0", "retained-1"].map(|tag| snapshot.tagged(tag).instance_id());
    let stabilized_builds = session
        .work_totals()
        .count("ordered_branch", crate::WorkPhase::Build);
    probe.reverse.get().expect("Reverse state").set(true);
    pump_layout_test_session(&mut session, 16_000_000);
    assert_eq!(
        retained_instances.map(|instance| session
            .work()
            .for_instance(instance, crate::WorkPhase::Measure)),
        [0, 0],
        "Reordering retained children with unchanged constraints must reuse their measurements"
    );
    assert_eq!(
        session
            .work_totals()
            .count("ordered_branch", crate::WorkPhase::Build),
        stabilized_builds,
        "Both branch bodies must really be retained during reorder"
    );
    assert_eq!(retained_node_ids(&session), retained);
    assert_eq!(
        semantic_reading_order(&session),
        ["Retained 1", "Retained 0"]
    );
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
        probe.hits.with(Clone::clone),
        [0],
        "The new last sibling must be the topmost pointer consumer"
    );
}

#[tessera(crate)]
fn optional_pointer_fixture(probe: ReplayProbe) {
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
    layout().policy(Overlap).modifier(modifier);
}

#[test]
fn retained_layout_leaves_pointer_index_when_its_last_handler_is_removed() {
    let probe = ReplayProbe::default();
    let mut session = start_layout_test_session(
        layout_test(|| {
            let probe = probe.clone();
            layout()
                .policy(Overlap)
                .child(move || optional_pointer_fixture(probe.clone()));
        })
        .viewport_px(100, 100),
    );
    pump_layout_test_session(&mut session, 0);
    let layout_id = |session: &crate::testing::LayoutTestSession<_>| {
        session
            .inspect_components()
            .tagged("optional-pointer")
            .node_id()
    };
    let retained = layout_id(&session);
    let position = PxPosition::from_f32_xy(10.0, 10.0);
    let button = PressKeyEventType::Left;
    for (frame, enabled, hits) in [(1, true, 1), (2, false, 1), (3, true, 2)] {
        probe.armed.get().unwrap().set(enabled);
        let timestamp = Instant::now();
        let changes = [
            CursorEventContent::Pressed(button),
            CursorEventContent::Released(button),
        ]
        .map(|event| {
            PointerChange::new(
                timestamp,
                crate::MOUSE_POINTER_ID,
                event,
                GestureState::TapCandidate,
            )
            .with_position(position)
        });
        pump_layout_test_session_with_input(
            &mut session,
            frame * 16_000_000,
            changes.into(),
            Some(position),
        );
        assert_eq!(layout_id(&session), retained);
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

#[tessera(crate)]
fn conditional_description(probe: ReplayProbe) {
    let visible = remember(|| true);
    probe.reverse.set(Some(visible));
    if visible.get() {
        Modifier::new()
            .push_semantics(Hint("Help text".into()))
            .attach();
    }
    layout().policy(Overlap);
}

#[test]
fn rebuilding_a_node_without_semantics_discards_its_previous_description() {
    let probe = ReplayProbe::default();
    let mut session = start_layout_test_session(layout_test(|| {
        conditional_description(probe.clone());
    }));
    let description = |session: &crate::testing::LayoutTestSession<_>| {
        let components = session.inspect_components();
        let host = components
            .nodes()
            .iter()
            .find(|node| node.name() == "conditional_description")
            .expect("Description host");
        (
            host.node_id(),
            host.semantics().and_then(|node| node.tooltip.clone()),
        )
    };
    pump_layout_test_session(&mut session, 0);
    let (host, initial) = description(&session);
    assert_eq!(initial.as_deref(), Some("Help text"));
    probe.reverse.get().expect("Visibility state").set(false);
    pump_layout_test_session(&mut session, 16_000_000);
    assert_eq!(description(&session), (host, None));
}

#[tessera(crate)]
fn scoped_fixture(probe: ReplayProbe) {
    let changed = remember(|| false);
    probe.reverse.set(Some(changed));
    let changed = changed.get();
    let scopes = remember(|| (FocusScopeNode::new(), FocusScopeNode::new())).get();
    probe.scopes.set(Some(scopes));
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
        .child(move || {
            let probe = probe.clone();
            ordered_branch(0, probe.clone());
        });
}

#[test]
fn replay_refreshes_retained_descendants_when_semantic_and_focus_ancestry_changes() {
    let probe = ReplayProbe::default();

    probe.requesters.with_mut(BTreeMap::clear);
    let mut session = start_layout_test_session(
        layout_test(|| {
            scoped_fixture(probe.clone());
        })
        .viewport_px(100, 100),
    );
    pump_layout_test_session(&mut session, 0);
    let requester = probe.requesters.with(|items| items[&0]);
    let (old_scope, new_scope) = probe.scopes.get().expect("Remembered focus scopes");
    requester.request_focus();
    assert!(old_scope.has_focus());
    assert!(!new_scope.has_focus());
    // Drain the focus host invalidation before changing its ancestor context.
    pump_layout_test_session(&mut session, 8_000_000);
    let scope_nodes = |session: &crate::testing::LayoutTestSession<_>| {
        let components = session.inspect_components();
        let host = components
            .nodes()
            .iter()
            .find(|node| node.name() == "scoped_fixture")
            .expect("Scope host");
        let layout = components
            .nodes()
            .iter()
            .find(|node| {
                node.name() == "layout"
                    && node
                        .semantics()
                        .is_some_and(|semantics| semantics.tooltip.is_some())
            })
            .expect("Scope layout");
        (host.node_id(), layout.node_id())
    };
    let scope = scope_nodes(&session);
    let retained = retained_node_ids(&session);
    let stabilized_builds = session
        .work_totals()
        .count("ordered_branch", crate::WorkPhase::Build);
    probe.reverse.get().expect("Changed state").set(true);
    pump_layout_test_session(&mut session, 16_000_000);
    assert_eq!(scope_nodes(&session), scope);
    assert_eq!(
        session
            .work_totals()
            .count("ordered_branch", crate::WorkPhase::Build),
        stabilized_builds,
        "The descendant must really reuse its component subtree during ancestry change"
    );
    assert_eq!(retained_node_ids(&session), retained);
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
