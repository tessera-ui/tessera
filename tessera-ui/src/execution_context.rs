use std::{cell::RefCell, ptr::NonNull, sync::Arc};

use rustc_hash::FxHashSet as HashSet;

use crate::{
    NodeId,
    context::ContextMap,
    focus::FocusOwner,
    runtime::{CompositionRuntime, RuntimePhase, TesseraRuntime},
};

#[derive(Default)]
pub(crate) struct ExecutionContext {
    pub(crate) current_runtime_stack: Vec<NonNull<TesseraRuntime>>,
    pub(crate) current_composition_runtime_stack: Vec<NonNull<CompositionRuntime>>,
    pub(crate) current_focus_owner_stack: Vec<NonNull<FocusOwner>>,
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
}

impl ExecutionContext {
    pub(crate) fn new() -> Self {
        Self {
            context_stack: vec![ContextMap::new()],
            ..Self::default()
        }
    }
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

pub(crate) fn with_context_stack<R>(f: impl FnOnce(&Vec<ContextMap>) -> R) -> R {
    with_execution_context(|context| f(&context.context_stack))
}

pub(crate) fn with_context_stack_mut<R>(f: impl FnOnce(&mut Vec<ContextMap>) -> R) -> R {
    with_execution_context_mut(|context| f(&mut context.context_stack))
}

pub(crate) fn with_focus_owner_stack<R>(f: impl FnOnce(&Vec<NonNull<FocusOwner>>) -> R) -> R {
    with_execution_context(|context| f(&context.current_focus_owner_stack))
}

pub(crate) fn with_focus_owner_stack_mut<R>(
    f: impl FnOnce(&mut Vec<NonNull<FocusOwner>>) -> R,
) -> R {
    with_execution_context_mut(|context| f(&mut context.current_focus_owner_stack))
}

#[derive(Clone, Copy, Default)]
pub(crate) struct OrderFrame {
    pub(crate) remember: u64,
    pub(crate) functor: u64,
    pub(crate) context: u64,
    pub(crate) instance: u64,
    pub(crate) frame_receiver: u64,
}

#[derive(Clone, Copy)]
pub(crate) enum OrderCounterKind {
    Remember,
    Functor,
    Context,
    FrameReceiver,
}

pub(crate) fn push_order_frame() {
    with_execution_context_mut(|context| context.order_frame_stack.push(OrderFrame::default()));
}

pub(crate) fn pop_order_frame(underflow_message: &str) {
    with_execution_context_mut(|context| {
        let popped = context.order_frame_stack.pop();
        debug_assert!(popped.is_some(), "{underflow_message}");
    });
}

pub(crate) fn next_order_counter(kind: OrderCounterKind, empty_message: &str) -> u64 {
    with_execution_context_mut(|context| {
        debug_assert!(!context.order_frame_stack.is_empty(), "{empty_message}");
        let frame = context.order_frame_stack.last_mut().expect(empty_message);
        match kind {
            OrderCounterKind::Remember => {
                let counter = frame.remember;
                frame.remember = frame.remember.wrapping_add(1);
                counter
            }
            OrderCounterKind::Functor => {
                let counter = frame.functor;
                frame.functor = frame.functor.wrapping_add(1);
                counter
            }
            OrderCounterKind::Context => {
                let counter = frame.context;
                frame.context = frame.context.wrapping_add(1);
                counter
            }
            OrderCounterKind::FrameReceiver => {
                let counter = frame.frame_receiver;
                frame.frame_receiver = frame.frame_receiver.wrapping_add(1);
                counter
            }
        }
    })
}

pub(crate) fn next_child_instance_call_index() -> u64 {
    with_execution_context_mut(|context| {
        let Some(frame) = context.order_frame_stack.last_mut() else {
            return 0;
        };
        let index = frame.instance;
        frame.instance = frame.instance.wrapping_add(1);
        index
    })
}
