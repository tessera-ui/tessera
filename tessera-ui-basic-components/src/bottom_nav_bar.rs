//! A bottom navigation bar for switching between primary application screens.
//!
//! This module provides the [`bottom_nav_bar`] component, which creates a persistent,
//! horizontal bar at the bottom of the UI. It is designed to work with a router to
//! control which main screen or "shard" is currently visible.
//!
//! # Key Components
//!
//! * **[`bottom_nav_bar`]**: The main function that renders the navigation bar.
//! * **[`BottomNavBarState`]**: A state object that must be created to track the
//!   currently selected navigation item.
//! * **[`BottomNavBarScope`]**: A scope provided to the `bottom_nav_bar`'s closure
//!   to add individual navigation items.
//!
//! # Usage
//!
//! The typical layout involves placing the `bottom_nav_bar` in a `column` below a
//! `router_root` component. This ensures the navigation bar remains visible while the
//! content above it changes.
//!
//! 1.  **Create State**: Create an `Arc<RwLock<BottomNavBarState>>` at a high level
//!     in your application state.
//! 2.  **Define Layout**: In your root component, create a `column`. Place a `router_root`
//!     in the first (weighted) child slot and the `bottom_nav_bar` in the second.
//! 3.  **Add Items**: Inside the `bottom_nav_bar` closure, use the provided scope's
//!     [`child`](BottomNavBarScope::child) method to add each navigation destination.
//!     - The first argument to `child` is a closure that renders the item's content (e.g., an icon or text).
//!     - The second argument is an `on_click` closure where you perform the navigation,
//!       typically by calling `tessera_ui::router::push` with the destination shard.
//!
//! # Example
//!
//! ```
//! use std::sync::Arc;
//! use parking_lot::RwLock;
//! use tessera_ui::{tessera, router::{self, router_root}};
//! use tessera_ui_basic_components::{
//!     bottom_nav_bar::{bottom_nav_bar, BottomNavBarState},
//!     column::{ColumnArgsBuilder, column},
//!     text::{text, TextArgsBuilder},
//! };
//!
//! // Assume HomeScreenDestination and ProfileScreenDestination are defined shards.
//! # use tessera_ui::shard;
//! # #[tessera] #[shard] fn home_screen() {}
//! # #[tessera] #[shard] fn profile_screen() {}
//!
//! #[tessera]
//! fn app_root() {
//!     let nav_bar_state = Arc::new(RwLock::new(BottomNavBarState::new(0)));
//!
//!     column(ColumnArgsBuilder::default().build().unwrap(), move |scope| {
//!         // The router viewport takes up the remaining space.
//!         scope.child_weighted(|| {
//!             router_root(HomeScreenDestination {});
//!         }, 1.0);
//!
//!         // The navigation bar is always visible at the bottom.
//!         scope.child(move || {
//!             bottom_nav_bar(nav_bar_state.clone(), |nav_scope| {
//!                 // Add the "Home" item.
//!                 nav_scope.child(
//!                     || text(TextArgsBuilder::default().text("Home".to_string()).build().unwrap()),
//!                     move || {
//!                         router::pop(); // Clear the backstack
//!                         router::push(HomeScreenDestination {});
//!                     },
//!                 );
//!
//!                 // Add the "Profile" item.
//!                 nav_scope.child(
//!                     || text(TextArgsBuilder::default().text("Profile".to_string()).build().unwrap()),
//!                     move || {
//!                         router::pop();
//!                         router::push(ProfileScreenDestination {});
//!                     },
//!                 );
//!             });
//!         });
//!     });
//! }
//! ```
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

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

    /// Returns the index of the currently selected navigation item.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Returns the index of the previously selected navigation item.
    /// This is useful for animations when transitioning between selected items.
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
