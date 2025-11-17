//! Bottom navigation bar for switching between primary app screens.
//!
//! ## Usage
//!
//! Use for top-level navigation between a small number of primary application screens.
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use parking_lot::{Mutex, RwLock};
use tessera_ui::{Color, DimensionValue, tessera};

use crate::{
    RippleState,
    alignment::MainAxisAlignment,
    animation,
    button::{ButtonArgsBuilder, button},
    pipelines::ShadowProps,
    row::{RowArgsBuilder, row},
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
};

const ANIMATION_DURATION: Duration = Duration::from_millis(300);
const ACTIVE_COLOR: Color = Color::from_rgb_u8(225, 235, 255);
const INACTIVE_COLOR: Color = Color::WHITE;
const ACTIVE_COLOR_SHADOW: Color = Color::from_rgba_u8(100, 115, 140, 100);

fn interpolate_color(from: Color, to: Color, progress: f32) -> Color {
    Color {
        r: from.r + (to.r - from.r) * progress,
        g: from.g + (to.g - from.g) * progress,
        b: from.b + (to.b - from.b) * progress,
        a: from.a + (to.a - from.a) * progress,
    }
}

/// # bottom_nav_bar
///
/// Provides a bottom navigation bar for switching between primary app screens.
///
/// ## Usage
///
/// Use for top-level navigation between a small number of primary application screens.
///
/// ## Parameters
///
/// - `state` — a clonable [`BottomNavBarState`] used to track the selected item.
/// - `scope_config` — a closure that receives a [`BottomNavBarScope`] for adding navigation items.
///
/// ## Examples
///
/// ```
/// use tessera_ui_basic_components::bottom_nav_bar::BottomNavBarState;
///
/// // Create a new state, starting with the first item (index 0) selected.
/// let state = BottomNavBarState::new(0);
/// assert_eq!(state.selected(), 0);
///
/// // The default state also selects the first item.
/// let default_state = BottomNavBarState::default();
/// assert_eq!(default_state.selected(), 0);
/// ```
#[tessera]
pub fn bottom_nav_bar<F>(state: BottomNavBarState, scope_config: F)
where
    F: FnOnce(&mut BottomNavBarScope),
{
    let mut child_closures = Vec::new();

    {
        let mut scope = BottomNavBarScope {
            child_closures: &mut child_closures,
        };
        scope_config(&mut scope);
    }

    let progress = state.animation_progress().unwrap_or(1.0);

    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .style(Color::from_rgb(9.333, 9.333, 9.333).into())
            .shadow(ShadowProps::default())
            .block_input(true)
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
                            let selected = state_clone.selected();
                            let previous_selected = state_clone.previous_selected();
                            let ripple_state = state_clone.ripple_state(index);

                            let color;
                            let shadow_color;
                            if index == selected {
                                color = interpolate_color(INACTIVE_COLOR, ACTIVE_COLOR, progress);
                                shadow_color =
                                    interpolate_color(INACTIVE_COLOR, ACTIVE_COLOR_SHADOW, progress)
                            } else if index == previous_selected {
                                color = interpolate_color(ACTIVE_COLOR, INACTIVE_COLOR, progress);
                                shadow_color =
                                    interpolate_color(ACTIVE_COLOR_SHADOW, INACTIVE_COLOR, progress)
                            } else {
                                color = INACTIVE_COLOR;
                                shadow_color = INACTIVE_COLOR;
                            }

                            let button_args = ButtonArgsBuilder::default()
                                .color(color)
                                .shape(Shape::HorizontalCapsule)
                                .on_click(Arc::new(move || {
                                    if index != selected {
                                        state_clone.set_selected(index);
                                        on_click.lock().take().unwrap()();
                                    }
                                }))
                                .shadow(ShadowProps {
                                    color: shadow_color,
                                    ..Default::default()
                                })
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
struct BottomNavBarStateInner {
    selected: usize,
    previous_selected: usize,
    ripple_states: HashMap<usize, RippleState>,
    anim_start_time: Option<Instant>,
}

impl BottomNavBarStateInner {
    fn new(selected: usize) -> Self {
        Self {
            selected,
            previous_selected: selected,
            ripple_states: HashMap::new(),
            anim_start_time: None,
        }
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
                Some(animation::easing(
                    elapsed.as_secs_f32() / ANIMATION_DURATION.as_secs_f32(),
                ))
            } else {
                self.anim_start_time = None;
                None
            }
        } else {
            None
        }
    }

    fn ripple_state(&mut self, index: usize) -> RippleState {
        self.ripple_states.entry(index).or_default().clone()
    }
}

#[derive(Clone)]
pub struct BottomNavBarState {
    inner: Arc<RwLock<BottomNavBarStateInner>>,
}

impl BottomNavBarState {
    /// Create a new state with an initial selected index.
    pub fn new(selected: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(BottomNavBarStateInner::new(selected))),
        }
    }

    /// Returns the index of the currently selected navigation item.
    pub fn selected(&self) -> usize {
        self.inner.read().selected
    }

    /// Returns the index of the previously selected navigation item.
    pub fn previous_selected(&self) -> usize {
        self.inner.read().previous_selected
    }

    fn set_selected(&self, index: usize) {
        self.inner.write().set_selected(index);
    }

    fn animation_progress(&self) -> Option<f32> {
        self.inner.write().animation_progress()
    }

    fn ripple_state(&self, index: usize) -> RippleState {
        self.inner.write().ripple_state(index)
    }
}

impl Default for BottomNavBarState {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Scope passed to the closure for defining children of the BottomNavBar.
pub struct BottomNavBarScope<'a> {
    child_closures: &'a mut Vec<(
        Box<dyn FnOnce() + Send + Sync>,
        Arc<Mutex<Option<Box<dyn FnOnce() + Send + Sync>>>>,
    )>,
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
        O: FnOnce() + Send + Sync + 'static,
    {
        self.child_closures.push((
            Box::new(child),
            Arc::new(Mutex::new(Some(Box::new(on_click)))),
        ));
    }
}
