use std::{collections::HashMap, sync::Arc, time::{Duration, Instant}};

use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, tessera};

use crate::{
    RippleState,
    alignment::MainAxisAlignment,
    animation,
    button::{ButtonArgsBuilder, button},
    row::{RowArgsBuilder, row},
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
};

const ANIMATION_DURATION: Duration = Duration::from_millis(300);
const ACTIVE_COLOR: Color = Color::from_rgb_u8(225, 235, 255);
const INACTIVE_COLOR: Color = Color::WHITE;

fn interpolate_color(from: Color, to: Color, progress: f32) -> Color {
    Color {
        r: from.r + (to.r - from.r) * progress,
        g: from.g + (to.g - from.g) * progress,
        b: from.b + (to.b - from.b) * progress,
        a: from.a + (to.a - from.a) * progress,
    }
}

/// A horizontal bottom navigation bar that hosts multiple navigation items (children),
/// each with its own click callback. The currently selected item is visually highlighted
/// (pill style) and tracked inside a shared [`BottomNavBarState`].
///
/// # State Handling
///
/// * The `state: Arc<RwLock<BottomNavBarState>>` holds:
///   - `selected`: index of the active item
///   - A lazily created `RippleState` per item (for button ripple feedback)
/// * The active item is rendered with a capsule shape & filled color; inactive items are
///   rendered as transparent buttons.
///
/// # Building Children
///
/// Children are registered via the provided closure `scope_config` which receives a
/// mutable [`BottomNavBarScope`]. Each child is added with:
/// `scope.child(content_closure, on_click_closure)`.
///
/// `on_click_closure` is responsible for performing side effects (e.g. pushing a new route).
/// The component automatically updates `selected` and triggers the ripple state before
/// invoking the user `on_click`.
///
/// # Layout
///
/// Internally the bar is:
/// * A full‑width `surface` (non‑interactive container)
/// * A `row` whose children are spaced using `MainAxisAlignment::SpaceAround`
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use parking_lot::RwLock;
/// use tessera_ui_basic_components::{
///     bottom_nav_bar::{bottom_nav_bar, BottomNavBarState},
///     text::text,
/// };
/// use tessera_ui::router;
///
/// let nav_state = Arc::new(RwLock::new(BottomNavBarState::new(0)));
/// bottom_nav_bar(nav_state.clone(), |scope| {
///     scope.child(
///         || text("Home"),
///         || {
///             router::pop();
///             // push new destination...
///         },
///     );
///     scope.child(
///         || text("Profile"),
///         || {
///             // navigate to profile...
///         },
///     );
/// });
/// ```
///
/// # Notes
///
/// * Indices are assigned in the order children are added.
/// * The bar itself does not do routing — supply routing logic inside each child's
///   `on_click` closure.
/// * Thread safety for `selected` & ripple states is provided by `RwLock`.
#[tessera]
pub fn bottom_nav_bar<F>(state: Arc<RwLock<BottomNavBarState>>, scope_config: F)
where
    F: FnOnce(&mut BottomNavBarScope),
{
    let mut child_closures: Vec<(Box<dyn FnOnce() + Send + Sync>, Arc<dyn Fn() + Send + Sync>)> =
        Vec::new();

    {
        let mut scope = BottomNavBarScope {
            child_closures: &mut child_closures,
        };
        scope_config(&mut scope);
    }

    let progress = {
        let mut state = state.write();
        state.animation_progress().unwrap_or(1.0)
    };

    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .color(Color::from_rgb(9.333, 9.333, 9.333))
            .build()
            .unwrap(),
        None,
        move || {
            row(
                RowArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .main_axis_alignment(MainAxisAlignment::SpaceAround)
                    .build()
                    .unwrap(),
                move |row_scope| {
                    for (index, (child_content, on_click)) in child_closures.into_iter().enumerate()
                    {
                        let state_clone = state.clone();
                        row_scope.child(move || {
                            let (selected, previous_selected) = {
                                let s = state_clone.read();
                                (s.selected(), s.previous_selected())
                            };
                            let ripple_state = state_clone.write().ripple_state(index);

                            let (color, shape) = if index == selected {
                                (
                                    interpolate_color(INACTIVE_COLOR, ACTIVE_COLOR, progress),
                                    Shape::HorizontalCapsule,
                                )
                            } else if index == previous_selected {
                                (
                                    interpolate_color(ACTIVE_COLOR, INACTIVE_COLOR, progress),
                                    Shape::HorizontalCapsule,
                                )
                            } else {
                                (INACTIVE_COLOR, Shape::default())
                            };

                            let button_args = ButtonArgsBuilder::default()
                                .color(color)
                                .shape(shape)
                                .on_click(Arc::new(move || {
                                    state_clone.write().set_selected(index);
                                    on_click();
                                }))
                                .build()
                                .unwrap();

                            button(button_args, ripple_state, || {
                                child_content();
                            });
                        });
                    }
                },
            );
        },
    );
}

/// Holds selection & per-item ripple state for the bottom navigation bar.
///
/// `selected` is the currently active item index. `ripple_states` lazily allocates a
/// `RippleState` (shared for each item) on first access, enabling ripple animations
/// on its associated button.
pub struct BottomNavBarState {
    selected: usize,
    previous_selected: usize,
    ripple_states: HashMap<usize, Arc<RippleState>>,
    anim_start_time: Option<Instant>,
}

impl Default for BottomNavBarState {
    fn default() -> Self {
        Self::new(0)
    }
}

impl BottomNavBarState {
    /// Create a new state with an initial selected index.
    pub fn new(selected: usize) -> Self {
        Self {
            selected,
            previous_selected: selected,
            ripple_states: HashMap::new(),
            anim_start_time: None,
        }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn previous_selected(&self) -> usize {
        self.previous_selected
    }

    fn set_selected(&mut self, index: usize) {
        if self.selected != index {
            self.previous_selected = self.selected;
            self.selected = index;
            self.anim_start_time = Some(Instant::now());
        }
    }

    fn animation_progress(&mut self) -> Option<f32> {
        if let Some(start_time) = self.anim_start_time {
            let elapsed = start_time.elapsed();
            if elapsed < ANIMATION_DURATION {
                Some(animation::easing(elapsed.as_secs_f32() / ANIMATION_DURATION.as_secs_f32()))
            } else {
                self.anim_start_time = None;
                None
            }
        } else {
            None
        }
    }

    fn ripple_state(&mut self, index: usize) -> Arc<RippleState> {
        self.ripple_states
            .entry(index)
            .or_insert_with(|| Arc::new(RippleState::new()))
            .clone()
    }
}

/// Scope passed to the closure for defining children of the BottomNavBar.
pub struct BottomNavBarScope<'a> {
    child_closures: &'a mut Vec<(Box<dyn FnOnce() + Send + Sync>, Arc<dyn Fn() + Send + Sync>)>,
}

impl<'a> BottomNavBarScope<'a> {
    /// Add a navigation item.
    ///
    /// * `child`: visual content (icon / label). Executed when the bar renders; must be
    ///   side‑effect free except for building child components.
    /// * `on_click`: invoked when this item is pressed; typical place for routing logic.
    ///
    /// The index of the added child becomes its selection index.
    pub fn child<C, O>(&mut self, child: C, on_click: O)
    where
        C: FnOnce() + Send + Sync + 'static,
        O: Fn() + Send + Sync + 'static,
    {
        self.child_closures
            .push((Box::new(child), Arc::new(on_click)));
    }
}
