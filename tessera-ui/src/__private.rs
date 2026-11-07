#![allow(missing_docs)]

pub use crate::{
    component_call::{ComponentCall, register_component_call, with_pending_scope},
    layout::layout,
    prop::Prop,
    runtime::{
        CurrentComponentInstanceGuard, GroupGuard, NodeContextGuard, PathGroupGuard, PhaseGuard,
        RuntimePhase, begin_component_node, push_current_component_instance_key, push_current_node,
        push_phase,
    },
    session::{local_runtime_resource, runtime_resource},
};

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
