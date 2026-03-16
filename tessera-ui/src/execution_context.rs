use std::{cell::RefCell, ptr::NonNull, sync::Arc};

use rustc_hash::FxHashSet as HashSet;

use crate::{NodeId, context::ContextMap, focus::FocusOwner, runtime::RuntimePhase};

#[derive(Default)]
pub(crate) struct ExecutionContext {
    pub(crate) node_context_stack: Vec<NodeId>,
    pub(crate) group_path_stack: Vec<u64>,
    pub(crate) instance_logic_id_stack: Vec<u64>,
    pub(crate) phase_stack: Vec<RuntimePhase>,
    pub(crate) order_frame_stack: Vec<OrderFrame>,
    pub(crate) instance_key_stack: Vec<u64>,
    pub(crate) current_component_instance_stack: Vec<u64>,
    pub(crate) next_node_instance_logic_id_override: Option<u64>,
    pub(crate) build_dirty_instance_keys_stack: Vec<Arc<HashSet<u64>>>,
    pub(crate) context_stack: Vec<ContextMap>,
    pub(crate) current_focus_owner_stack: Vec<NonNull<FocusOwner>>,
}

impl ExecutionContext {
    pub(crate) fn new() -> Self {
        Self {
            context_stack: vec![ContextMap::new()],
            ..Self::default()
        }
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct OrderFrame {
    pub(crate) remember: u64,
    pub(crate) functor: u64,
    pub(crate) context: u64,
    pub(crate) instance: u64,
    pub(crate) frame_receiver: u64,
}

thread_local! {
    static EXECUTION_CONTEXT: RefCell<ExecutionContext> = RefCell::new(ExecutionContext::new());
}

pub(crate) fn with_execution_context<R>(f: impl FnOnce(&ExecutionContext) -> R) -> R {
    EXECUTION_CONTEXT.with(|context| f(&context.borrow()))
}

pub(crate) fn with_execution_context_mut<R>(f: impl FnOnce(&mut ExecutionContext) -> R) -> R {
    EXECUTION_CONTEXT.with(|context| f(&mut context.borrow_mut()))
}

#[cfg(test)]
pub(crate) fn reset_execution_context() {
    with_execution_context_mut(|context| *context = ExecutionContext::new());
}
