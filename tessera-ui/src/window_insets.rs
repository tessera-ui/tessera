//! Window safe areas and on-screen keyboard occlusion.
//!
//! Use [`WindowInsets`] with inset layout modifiers. Platform integrations publish
//! changes through the session's [`WindowInsetsController`].

use crate::{Dp, State, StateOwner};

pub(crate) mod platform;

/// Insets values in density-independent pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets {
    /// Left insets.
    pub left: Dp,
    /// Top insets.
    pub top: Dp,
    /// Right insets.
    pub right: Dp,
    /// Bottom insets.
    pub bottom: Dp,
}

impl Insets {
    /// Creates insets with explicit edges.
    pub const fn new(left: Dp, top: Dp, right: Dp, bottom: Dp) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// Creates symmetric insets on all edges.
    pub const fn all(value: Dp) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }

    /// Creates symmetric insets for horizontal and vertical edges.
    pub const fn symmetric(horizontal: Dp, vertical: Dp) -> Self {
        Self {
            left: horizontal,
            top: vertical,
            right: horizontal,
            bottom: vertical,
        }
    }

    /// Creates insets with only horizontal edges.
    pub const fn horizontal(value: Dp) -> Self {
        Self::symmetric(value, Dp(0.0))
    }

    /// Creates insets with only vertical edges.
    pub const fn vertical(value: Dp) -> Self {
        Self::symmetric(Dp(0.0), value)
    }

    /// Creates insets with only the left edge.
    pub const fn left(value: Dp) -> Self {
        Self {
            left: value,
            top: Dp(0.0),
            right: Dp(0.0),
            bottom: Dp(0.0),
        }
    }

    /// Creates insets with only the top edge.
    pub const fn top(value: Dp) -> Self {
        Self {
            left: Dp(0.0),
            top: value,
            right: Dp(0.0),
            bottom: Dp(0.0),
        }
    }

    /// Creates insets with only the right edge.
    pub const fn right(value: Dp) -> Self {
        Self {
            left: Dp(0.0),
            top: Dp(0.0),
            right: value,
            bottom: Dp(0.0),
        }
    }

    /// Creates insets with only the bottom edge.
    pub const fn bottom(value: Dp) -> Self {
        Self {
            left: Dp(0.0),
            top: Dp(0.0),
            right: Dp(0.0),
            bottom: value,
        }
    }
}

impl Insets {
    /// Combines edge lengths by taking the larger value on each edge.
    pub fn union(self, other: Self) -> Self {
        Self::new(
            Dp(self.left.0.max(other.left.0)),
            Dp(self.top.0.max(other.top.0)),
            Dp(self.right.0.max(other.right.0)),
            Dp(self.bottom.0.max(other.bottom.0)),
        )
    }

    /// Subtracts occupied edges, clamping each remaining length to zero.
    pub fn exclude(self, consumed: Self) -> Self {
        Self::new(
            Dp((self.left.0 - consumed.left.0).max(0.0)),
            Dp((self.top.0 - consumed.top.0).max(0.0)),
            Dp((self.right.0 - consumed.right.0).max(0.0)),
            Dp((self.bottom.0 - consumed.bottom.0).max(0.0)),
        )
    }
}

/// A live window inset source, read during measurement by inset modifiers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WindowInsets {
    /// Status bars, navigation bars, cutouts and other window safe areas.
    SystemBars,
    /// The docked on-screen keyboard occluding the drawable window.
    Ime,
    /// The union of system bars and keyboard occlusion.
    SafeDrawing,
    /// Explicit edge lengths, including custom parent-consumed space.
    Fixed(Insets),
}

impl WindowInsets {
    /// Reads current lengths and tracks the active build or layout dependency.
    pub fn get(self) -> Insets {
        if let Self::Fixed(insets) = self {
            return insets;
        }
        let (system, ime) = WindowInsetsController::current().0.get();
        match self {
            Self::SystemBars => system,
            Self::Ime => ime,
            _ => system.union(ime),
        }
    }
}

/// A copyable controller for one application session's window insets.
#[derive(Clone, Copy, PartialEq)]
pub struct WindowInsetsController(State<(Insets, Insets)>);

struct WindowInsetsState {
    _owner: StateOwner,
    controller: WindowInsetsController,
}

impl Default for WindowInsetsState {
    fn default() -> Self {
        let owner = StateOwner::new();
        let controller = WindowInsetsController(owner.state_with_key((), Default::default));
        Self {
            _owner: owner,
            controller,
        }
    }
}

impl WindowInsetsController {
    /// Returns the active application's window inset controller.
    pub fn current() -> Self {
        crate::local_runtime_resource::<WindowInsetsState>().controller
    }

    /// Publishes safe-area and keyboard lengths in density-independent units.
    /// Unchanged values preserve cached builds and measurements.
    pub fn set(self, system_bars: Insets, ime: Insets) {
        self.0.set_if_changed((system_bars, ime));
    }
}
