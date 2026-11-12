use std::{ptr::NonNull, sync::Arc};

use rustc_hash::FxHashSet as HashSet;
use smallvec::SmallVec;

use crate::{
    NodeId,
    context::ContextMap,
    focus::FocusOwner,
    runtime::{GroupPath, RuntimePhase},
    session::with_active_execution_context,
};

#[derive(Default)]
pub(crate) struct ExecutionContext {
    pub(crate) node_context_stack: Vec<NodeId>,
    pub(crate) group_path: GroupPath,
    pub(crate) instance_logic_id_stack: Vec<u64>,
    pub(crate) phase_stack: Vec<RuntimePhase>,
    pub(crate) order_frame_stack: Vec<OrderFrame>,
    pub(crate) instance_key_override: Option<u64>,
    pub(crate) measured_node_instance_key_stack: Vec<u64>,
    pub(crate) current_component_instance_stack: Vec<u64>,
    pub(crate) next_node_instance_logic_id_override: Option<u64>,
    pub(crate) build_dirty_instance_keys_stack: Vec<Arc<HashSet<u64>>>,
    pub(crate) context_map: ContextMap,
    pub(crate) current_focus_owner_stack: Vec<NonNull<FocusOwner>>,
    pub(crate) pointer_state_reader: Option<u64>,
}

#[derive(Clone, Default)]
pub(crate) struct OrderFrame {
    pub(crate) remember: u64,
    pub(crate) functor: u64,
    pub(crate) context: u64,
    pub(crate) instance: u64,
    pub(crate) frame_receiver: u64,
    pub(crate) modifier_attachment: u64,
    pub(crate) occurrence_salt: u64,
    pub(crate) group_occurrences: SmallVec<[(u64, u64); 4]>,
}

pub(crate) fn with_execution_context<R>(f: impl FnOnce(&ExecutionContext) -> R) -> R {
    with_active_execution_context(|context| f(&context.borrow()))
}

pub(crate) fn with_execution_context_mut<R>(f: impl FnOnce(&mut ExecutionContext) -> R) -> R {
    with_active_execution_context(|context| f(&mut context.borrow_mut()))
}

#[cfg(test)]
pub(crate) fn reset_execution_context() {
    with_execution_context_mut(|context| *context = ExecutionContext::default());
}
