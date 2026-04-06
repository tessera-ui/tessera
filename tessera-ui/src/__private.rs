#![allow(missing_docs)]

use std::sync::Arc;

use indextree::NodeId;

use crate::{
    State,
    component_tree::{ComponentNode, NodeRole},
    layout::{DefaultLayoutPolicy, NoopRenderPolicy},
    modifier::Modifier,
    prop::ErasedComponentRunner,
    runtime::TesseraRuntime,
};

pub use crate::{
    layout::layout,
    prop::{Prop, make_component_runner},
    runtime::{
        CurrentComponentInstanceGuard, GroupGuard, NodeContextGuard, PathGroupGuard, PhaseGuard,
        RuntimePhase, push_current_component_instance_key, push_current_node, push_phase,
    },
};

#[cfg(feature = "shard")]
use crate::router::RouterController;
#[cfg(feature = "shard")]
use tessera_shard::{ShardState, ShardStateLifeCycle};

pub fn record_current_context_snapshot_for(instance_key: u64) {
    crate::context::record_current_context_snapshot_for(instance_key);
}

pub fn current_instance_logic_id() -> u64 {
    crate::runtime::current_instance_logic_id()
}

pub fn current_instance_key() -> u64 {
    crate::runtime::current_instance_key()
}

#[cfg(feature = "shard")]
pub fn current_router_controller() -> State<RouterController> {
    crate::router::current_router_controller()
}

#[cfg(feature = "shard")]
pub fn with_current_router_shard_state<T, F, R>(
    shard_id: &str,
    life_cycle: ShardStateLifeCycle,
    f: F,
) -> R
where
    T: Default + Send + Sync + 'static,
    F: FnOnce(ShardState<T>) -> R,
{
    crate::router::with_current_router_shard_state(shard_id, life_cycle, f)
}

pub fn register_component_node(fn_name: &str, _component_type_id: u64) -> NodeId {
    TesseraRuntime::with_mut(|runtime| {
        runtime.component_tree.add_node(ComponentNode {
            fn_name: fn_name.to_string(),
            role: NodeRole::Composition,
            instance_logic_id: 0,
            instance_key: 0,
            pointer_preview_handlers: Vec::new(),
            pointer_handlers: Vec::new(),
            pointer_final_handlers: Vec::new(),
            keyboard_preview_handlers: Vec::new(),
            keyboard_handlers: Vec::new(),
            ime_preview_handlers: Vec::new(),
            ime_handlers: Vec::new(),
            focus_requester_binding: None,
            focus_registration: None,
            focus_restorer_fallback: None,
            focus_traversal_policy: None,
            focus_changed_handler: None,
            focus_event_handler: None,
            focus_beyond_bounds_handler: None,
            focus_reveal_handler: None,
            modifier: Modifier::default(),
            layout_policy: Box::new(DefaultLayoutPolicy),
            render_policy: Box::new(NoopRenderPolicy),
            replay: None,
            props_unchanged_from_previous: false,
        })
    })
}

pub fn register_layout_node(fn_name: &str, _component_type_id: u64) -> NodeId {
    TesseraRuntime::with_mut(|runtime| {
        runtime.component_tree.add_node(ComponentNode {
            fn_name: fn_name.to_string(),
            role: NodeRole::Layout,
            instance_logic_id: 0,
            instance_key: 0,
            pointer_preview_handlers: Vec::new(),
            pointer_handlers: Vec::new(),
            pointer_final_handlers: Vec::new(),
            keyboard_preview_handlers: Vec::new(),
            keyboard_handlers: Vec::new(),
            ime_preview_handlers: Vec::new(),
            ime_handlers: Vec::new(),
            focus_requester_binding: None,
            focus_registration: None,
            focus_restorer_fallback: None,
            focus_traversal_policy: None,
            focus_changed_handler: None,
            focus_event_handler: None,
            focus_beyond_bounds_handler: None,
            focus_reveal_handler: None,
            modifier: Modifier::default(),
            layout_policy: Box::new(DefaultLayoutPolicy),
            render_policy: Box::new(NoopRenderPolicy),
            replay: None,
            props_unchanged_from_previous: false,
        })
    })
}

pub fn finish_component_node() {
    TesseraRuntime::with_mut(|runtime| {
        runtime.finalize_current_layout_policy_dirty();
        runtime.component_tree.pop_node();
    });
}

pub fn set_current_node_identity(instance_key: u64, instance_logic_id: u64) {
    TesseraRuntime::with_mut(|runtime| {
        runtime.set_current_node_identity(instance_key, instance_logic_id);
    });
}

pub fn set_current_component_replay<P>(runner: Arc<dyn ErasedComponentRunner>, props: &P) -> bool
where
    P: crate::prop::Prop,
{
    TesseraRuntime::with_mut(|runtime| runtime.set_current_component_replay(runner, props))
}
