//! Keyboard prefix matching within a menu.
//!
//! ## Usage
//!
//! Move focus between labeled menu actions without activating them.

use parking_lot::Mutex;
use tessera_ui::{FloatRect, FocusRequester, State};

const PREFIX_TIMEOUT: u64 = 1_000_000_000;

#[derive(Clone, Copy, PartialEq)]
pub(super) struct MenuTypeahead(pub State<Mutex<Registry>>);

struct Item {
    requester: FocusRequester,
    label: String,
    enabled: bool,
    bounds: Option<FloatRect>,
}

#[derive(Default)]
pub(super) struct Registry {
    items: Vec<Item>,
    prefix: String,
    last_input: Option<u64>,
}

impl Registry {
    fn matches(&self, prefix: &str, cycle: bool) -> Vec<FocusRequester> {
        let count = self.items.len();
        let start = self
            .items
            .iter()
            .position(|item| item.requester.is_focused())
            .map_or(0, |index| index + usize::from(cycle));
        (0..count)
            .map(|offset| &self.items[(start + offset) % count])
            .filter(|item| item.enabled && item.bounds.is_some() && item.label.starts_with(prefix))
            .map(|item| item.requester)
            .collect()
    }

    fn candidates(&mut self, text: &str, now: u64) -> Vec<FocusRequester> {
        if self
            .last_input
            .is_none_or(|previous| now < previous || now - previous >= PREFIX_TIMEOUT)
        {
            self.prefix.clear();
        }
        self.last_input = Some(now);
        self.prefix.push_str(text);
        let first = self.prefix.chars().next().unwrap();
        let cycle = self.prefix.chars().all(|character| character == first);
        if cycle {
            self.prefix = first.to_string();
        }
        self.items.sort_by(|a, b| match (a.bounds, b.bounds) {
            (Some(a), Some(b)) => a.y.total_cmp(&b.y).then_with(|| a.x.total_cmp(&b.x)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
        let matches = self.matches(&self.prefix, cycle);
        if matches.is_empty() && self.prefix != text {
            self.prefix = text.to_owned();
            self.matches(text, true)
        } else {
            matches
        }
    }
}

pub(super) struct ItemLease {
    registry: State<Mutex<Registry>>,
    requester: FocusRequester,
}

impl ItemLease {
    pub(super) fn new(registry: State<Mutex<Registry>>, requester: FocusRequester) -> Self {
        Self {
            registry,
            requester,
        }
    }
}

impl Drop for ItemLease {
    fn drop(&mut self) {
        if self.registry.is_alive() {
            self.registry.with(|registry| {
                registry
                    .lock()
                    .items
                    .retain(|item| item.requester != self.requester);
            });
        }
    }
}

pub(super) fn update(
    registry: State<Mutex<Registry>>,
    requester: FocusRequester,
    label: &str,
    enabled: bool,
) {
    registry.with(|registry| {
        let mut registry = registry.lock();
        if let Some(item) = registry
            .items
            .iter_mut()
            .find(|item| item.requester == requester)
        {
            item.label = label.to_lowercase();
            item.enabled = enabled;
        } else {
            registry.items.push(Item {
                requester,
                label: label.to_lowercase(),
                enabled,
                bounds: None,
            });
        }
    });
}

pub(super) fn position(
    registry: State<Mutex<Registry>>,
    requester: FocusRequester,
    bounds: FloatRect,
) {
    registry.with(|registry| {
        let mut registry = registry.lock();
        if let Some(item) = registry
            .items
            .iter_mut()
            .find(|item| item.requester == requester)
        {
            item.bounds = Some(bounds);
        }
    });
}

pub(super) fn type_text(registry: State<Mutex<Registry>>, text: &str, now: u64) -> bool {
    if text.trim().is_empty() || text.chars().any(char::is_control) {
        return false;
    }
    let text = text.to_lowercase();
    let targets = registry.with(|registry| registry.lock().candidates(&text, now));
    for target in targets {
        target.request_focus();
        if target.is_focused() {
            break;
        }
    }
    true
}
