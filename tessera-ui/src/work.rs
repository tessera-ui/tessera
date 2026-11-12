//! Executed UI work by frame and phase.
//!
//! ## Usage
//!
//! Inspect actual component, layout, and paint dispatches in test sessions.

/// A runtime execution phase.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WorkPhase {
    /// A component body executed.
    Build,
    /// Final content was measured.
    Measure,
    /// Intrinsic content was measured.
    IntrinsicMeasure,
    /// Cached measurements were used to place children.
    Place,
    /// A placement modifier transformed geometry.
    Transform,
    /// A draw policy or modifier recorded paint.
    Paint,
}

#[cfg(any(
    test,
    feature = "testing",
    all(feature = "headless", not(target_family = "wasm"))
))]
mod collector {
    use std::cell::RefCell;

    use rustc_hash::FxHashMap;

    use super::WorkPhase;

    /// One component, policy, or modifier execution.
    #[derive(Clone, Debug)]
    pub struct WorkExecution {
        /// Stable instance identity within the session.
        pub instance: u64,
        /// Dispatched phase.
        pub phase: WorkPhase,
        /// Component function name or concrete policy type name.
        pub target: &'static str,
    }

    /// Actual work in a completed frame.
    #[derive(Clone, Debug, Default)]
    pub struct FrameWork {
        /// Session that owns the observed instances.
        pub session: Option<crate::RuntimeSessionId>,
        executions: Vec<WorkExecution>,
    }

    impl FrameWork {
        /// Returns dispatches in execution order.
        pub fn executions(&self) -> &[WorkExecution] {
            &self.executions
        }

        /// Returns dispatches for one phase.
        pub fn phase(&self, phase: WorkPhase) -> impl Iterator<Item = &WorkExecution> {
            self.executions
                .iter()
                .filter(move |entry| entry.phase == phase)
        }

        /// Counts named component or policy dispatches in a phase.
        pub fn count(&self, target: &str, phase: WorkPhase) -> usize {
            self.phase(phase)
                .filter(|entry| entry.target == target)
                .count()
        }

        /// Counts dispatches of a concrete policy or modifier type.
        pub fn executed<T: 'static>(&self, phase: WorkPhase) -> usize {
            self.count(std::any::type_name::<T>(), phase)
        }

        /// Counts dispatches for one stable instance.
        pub fn for_instance(&self, instance: u64, phase: WorkPhase) -> usize {
            self.phase(phase)
                .filter(|entry| entry.instance == instance)
                .count()
        }
    }

    /// Cumulative work during a renderer or test session's lifetime.
    #[derive(Clone, Debug, Default)]
    pub struct WorkTotals {
        /// Session that owns the observed instances.
        pub session: Option<crate::RuntimeSessionId>,
        counts: FxHashMap<&'static str, [usize; 6]>,
    }

    impl WorkTotals {
        /// Counts named component or policy dispatches in a phase.
        pub fn count(&self, target: &str, phase: WorkPhase) -> usize {
            self.counts
                .get(target)
                .map_or(0, |counts| counts[phase as usize])
        }

        pub(crate) fn record(&mut self, frame: &FrameWork) {
            self.session = frame.session;
            for entry in frame.executions() {
                let count = &mut self.counts.entry(entry.target).or_default()[entry.phase as usize];
                *count = count.checked_add(1).expect("work count exhausted");
            }
        }
    }

    #[derive(Default)]
    struct Observer(RefCell<Option<FrameWork>>);

    pub(crate) fn begin() {
        crate::local_runtime_resource::<Observer>()
            .0
            .replace(Some(FrameWork {
                session: crate::RuntimeSession::current().map(|session| session.identity()),
                ..Default::default()
            }));
    }

    pub(crate) fn finish() -> FrameWork {
        crate::local_runtime_resource::<Observer>()
            .0
            .take()
            .unwrap_or_default()
    }

    pub(crate) fn record(instance: u64, phase: WorkPhase, target: &'static str) {
        if crate::RuntimeSession::current().is_none() {
            return;
        }
        if let Some(frame) = crate::local_runtime_resource::<Observer>()
            .0
            .borrow_mut()
            .as_mut()
        {
            frame.executions.push(WorkExecution {
                instance,
                phase,
                target,
            });
        }
    }

    pub(crate) fn record_policy<T>(phase: WorkPhase) {
        if crate::RuntimeSession::current().is_none() {
            return;
        }
        let instance = crate::execution_context::with_execution_context(|context| {
            context.instance_logic_id_stack.last().copied().unwrap_or(0)
        });
        record(instance, phase, std::any::type_name::<T>());
    }
}

#[cfg(any(
    test,
    feature = "testing",
    all(feature = "headless", not(target_family = "wasm"))
))]
pub use collector::{FrameWork, WorkExecution, WorkTotals};
#[cfg(any(
    test,
    feature = "testing",
    all(feature = "headless", not(target_family = "wasm"))
))]
pub(crate) use collector::{begin, finish, record, record_policy};

#[cfg(not(any(
    test,
    feature = "testing",
    all(feature = "headless", not(target_family = "wasm"))
)))]
pub(crate) fn record(_instance: u64, _phase: WorkPhase, _target: &'static str) {}

#[cfg(not(any(
    test,
    feature = "testing",
    all(feature = "headless", not(target_family = "wasm"))
)))]
pub(crate) fn record_policy<T>(_phase: WorkPhase) {}
