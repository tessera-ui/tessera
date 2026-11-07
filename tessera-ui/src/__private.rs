#![allow(missing_docs)]

use std::sync::Arc;

use indextree::NodeId;

use crate::{
    component_tree::{ComponentNode, NodeRole},
    runtime::TesseraRuntime,
};

pub use crate::{
    component_call::{
        ComponentCall, current_component_runner, register_component_call, with_pending_scope,
    },
    layout::layout,
    prop::{ErasedComponentRunner, Prop, make_component_runner},
    runtime::{
        CurrentComponentInstanceGuard, GroupGuard, NodeContextGuard, PathGroupGuard, PhaseGuard,
        RuntimePhase, push_current_component_instance_key, push_current_node, push_phase,
    },
    session::{local_runtime_resource, runtime_resource},
};

pub fn record_current_context_snapshot_for(instance_key: u64) {
    crate::context::record_current_context_snapshot_for(instance_key);
}

pub fn current_instance_logic_id() -> u64 {
    crate::runtime::current_instance_logic_id()
}

pub fn record_component_execution(name: &'static str, instance: u64) {
    crate::work::record(instance, crate::WorkPhase::Build, name);
}

pub fn current_instance_key() -> u64 {
    crate::runtime::current_instance_key()
}

pub fn current_phase() -> Option<RuntimePhase> {
    crate::runtime::current_phase()
}

pub fn current_runtime_session() -> Option<crate::WeakRuntimeSession> {
    crate::RuntimeSession::current().map(|session| session.downgrade())
}

pub fn current_replay_boundary_instance_key_from_scope() -> Option<u64> {
    crate::runtime::current_replay_boundary_instance_key_from_scope()
}

pub fn context_from_previous_snapshot_for_instance<T>(
    instance_key: u64,
) -> Option<crate::context::Context<T>>
where
    T: Send + Sync + 'static,
{
    crate::context::context_from_previous_snapshot_for_instance(instance_key)
}

pub fn register_component_node(fn_name: &str, _component_type_id: u64) -> NodeId {
    TesseraRuntime::with_mut(|runtime| {
        runtime
            .component_tree
            .add_node(ComponentNode::new(fn_name, NodeRole::Composition))
    })
}

pub fn register_layout_node(fn_name: &str, _component_type_id: u64) -> NodeId {
    TesseraRuntime::with_mut(|runtime| {
        runtime
            .component_tree
            .add_node(ComponentNode::new(fn_name, NodeRole::Layout))
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
