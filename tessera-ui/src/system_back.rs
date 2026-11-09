//! Application handling for Android system back navigation.

use std::{
    collections::BTreeMap,
    sync::{Arc, Weak},
};

use parking_lot::Mutex;

use crate::{CallbackWith, remember};

/// The screen edge that started a predictive back gesture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackSwipeEdge {
    /// Left edge of the screen.
    Left,
    /// Right edge of the screen.
    Right,
}

/// A phase of system back navigation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SystemBackEvent {
    /// A predictive gesture started.
    Started(BackSwipeEdge),
    /// The gesture moved. The progress runs from zero to one.
    Progress {
        /// Fraction of the gesture completed.
        progress: f32,
        /// Edge where the gesture started.
        edge: BackSwipeEdge,
    },
    /// The gesture was cancelled. Restore the original view.
    Cancelled,
    /// Perform the back action. A button press can commit without a preview.
    Committed,
}

type BackCallback = CallbackWith<SystemBackEvent>;

/// Marks an off-screen destination preview that must not handle system back.
#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct BackPreviewScope(pub bool);

#[derive(Default)]
struct Registry {
    next_id: u64,
    callbacks: BTreeMap<u64, BackCallback>,
    #[cfg(any(target_os = "android", test, feature = "testing"))]
    gesture: Option<Option<u64>>,
}

impl Registry {
    #[cfg(any(target_os = "android", test, feature = "testing"))]
    fn active(&self) -> Option<u64> {
        self.callbacks.last_key_value().map(|(&id, _)| id)
    }

    #[cfg(any(target_os = "android", test, feature = "testing"))]
    fn callback(&mut self, event: SystemBackEvent) -> Option<BackCallback> {
        let id = match event {
            SystemBackEvent::Started(_) => {
                let active = self.active();
                self.gesture = Some(active);
                active
            }
            SystemBackEvent::Progress { .. } => self.gesture.flatten(),
            SystemBackEvent::Cancelled => self.gesture.take().flatten(),
            SystemBackEvent::Committed => self.gesture.take().unwrap_or_else(|| self.active()),
        }?;
        self.callbacks.get(&id).copied()
    }
}

type Handlers = Mutex<Registry>;

struct Registration {
    handlers: Weak<Handlers>,
    id: u64,
}

impl Drop for Registration {
    fn drop(&mut self) {
        if let Some(handlers) = self.handlers.upgrade() {
            handlers.lock().callbacks.remove(&self.id);
        }
    }
}

/// Registers an application back handler while this component is mounted.
///
/// On Android, the most recently mounted enabled handler receives system back.
/// Enable it only while its destination or overlay can go back. With no enabled
/// handler, Android performs its normal back action. A predictive gesture keeps
/// its starting handler until it commits or is cancelled.
#[crate::tessera]
pub fn system_back_handler(enabled: bool, on_back: Option<CallbackWith<SystemBackEvent>>) {
    if crate::use_context::<BackPreviewScope>().is_some_and(|scope| scope.get().0) {
        return;
    }
    let handlers = crate::runtime_resource::<Handlers>();
    let registration = remember(|| {
        let mut registry = handlers.lock();
        let id = registry.next_id;
        registry.next_id += 1;
        Registration {
            handlers: Arc::downgrade(&handlers),
            id,
        }
    });
    let id = registration.with(|registration| registration.id);
    let mut registry = handlers.lock();
    if let Some(callback) = on_back.filter(|_| enabled) {
        registry.callbacks.insert(id, callback);
    } else {
        registry.callbacks.remove(&id);
    }
}

#[cfg(target_os = "android")]
pub(crate) fn enabled() -> bool {
    crate::runtime_resource::<Handlers>()
        .lock()
        .active()
        .is_some()
}

#[cfg(any(target_os = "android", test, feature = "testing"))]
pub(crate) fn dispatch(event: SystemBackEvent) {
    let callback = crate::runtime_resource::<Handlers>().lock().callback(event);
    if let Some(callback) = callback {
        callback.call(event);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        State, remember,
        testing::{TestProbe, layout_test, pump_layout_test_session, start_layout_test_session},
    };

    use super::*;

    #[crate::tessera]
    fn fixture(
        overlay: TestProbe<Option<State<bool>>>,
        events: TestProbe<Vec<(&'static str, SystemBackEvent)>>,
    ) {
        let visible = remember(|| false);
        overlay.set(Some(visible));
        let app = events.clone();
        system_back_handler(true).on_back(move |event| {
            app.with_mut(|events| events.push(("app", event)));
        });
        system_back_handler(visible.get()).on_back(move |event| {
            events.with_mut(|events| events.push(("overlay", event)));
        });
    }

    #[test]
    fn gesture_keeps_its_owner_and_a_later_button_press_dismisses_the_overlay() {
        let overlay = TestProbe::default();
        let events = TestProbe::default();
        let probe = overlay.clone();
        let log = events.clone();
        let mut session = start_layout_test_session(layout_test(move || {
            fixture(probe.clone(), log.clone());
        }));
        pump_layout_test_session(&mut session, 0);
        let overlay = overlay.get().expect("mounted overlay");
        let started = SystemBackEvent::Started(BackSwipeEdge::Right);
        session.dispatch_system_back(started);
        session.run(|| overlay.set(true));
        pump_layout_test_session(&mut session, 16_000_000);
        session.dispatch_system_back(SystemBackEvent::Cancelled);
        session.dispatch_system_back(SystemBackEvent::Committed);
        assert_eq!(
            events.take(),
            vec![
                ("app", started),
                ("app", SystemBackEvent::Cancelled),
                ("overlay", SystemBackEvent::Committed),
            ]
        );
    }
}
