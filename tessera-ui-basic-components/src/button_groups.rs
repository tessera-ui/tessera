//! Material 3-style segmented buttons with single or multiple selection.
//!
//! ## Usage
//!
//! Used for grouping related actions.

use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use closure::closure;
use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{Color, ComputedData, Dp, Px, PxPosition, tessera};

use crate::{
    RippleState,
    alignment::MainAxisAlignment,
    animation,
    button::{ButtonArgs, button},
    material_color::global_material_scheme,
    row::{RowArgs, row},
    shape_def::{RoundedCorner, Shape},
    spacer::{SpacerArgs, spacer},
};

/// According to the [`ButtonGroups-Types`](https://m3.material.io/components/button-groups/specs#3b51d175-cc02-4701-b3f8-c9ffa229123a)
/// spec, the [`button_groups`] component supports two styles: `Standard` and `Connected`.
///
/// ## Standard
///
/// Buttons have spacing between them and do not need to be the same width.
///
/// ## Connected
///
/// Buttons are adjacent with no spacing, and each button must be the same width.
#[derive(Debug, Clone, Copy, Default)]
pub enum ButtonGroupsStyle {
    /// Buttons have spacing between them and do not need to be the same width.
    #[default]
    Standard,
    /// Buttons are adjacent with no spacing, and each button must be the same width.
    Connected,
}

/// According to the [`ButtonGroups-Configurations`](https://m3.material.io/components/button-groups/specs#0d2cf762-275c-4693-9484-fe011501439e)
/// spec, the [`button_groups`] component supports two selection modes: `Single` and `Multiple`.
///
/// ## Single
///
/// Only one button can be selected at a time.
///
/// ## Multiple
///
/// Multiple buttons can be selected at the same time.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonGroupsSelectionMode {
    /// Only one button can be selected at a time.
    #[default]
    Single,
    /// Multiple buttons can be selected at the same time.
    Multiple,
}

/// According to the [`ButtonGroups-Configurations`](https://m3.material.io/components/button-groups/specs#0d2cf762-275c-4693-9484-fe011501439e)
/// spec, the [`button_groups`] component supports a series of sizes.
#[derive(Debug, Clone, Copy, Default)]
pub enum ButtonGroupsSize {
    /// Extra small size.
    ExtraSmall,
    /// Small size.
    Small,
    /// Medium size.
    #[default]
    Medium,
    /// Large size.
    Large,
    /// Extra large size.
    ExtraLarge,
}

/// A scope for declaratively adding children to a [`button_groups`] component.
pub struct ButtonGroupsScope<'a> {
    child_closures: &'a mut Vec<Box<dyn FnOnce(Color) + Send + Sync>>,
    on_click_closures: &'a mut Vec<Arc<dyn Fn(bool) + Send + Sync>>,
}

impl ButtonGroupsScope<'_> {
    /// Add a child component to the button group, which will be wrapped by a [`button`] component.
    ///
    /// # Arguments
    ///
    /// - `child_closure` - A closure that takes a [`Color`] and returns a [`button`] component. The
    ///   `Color` argument should be used for the content of the child component.
    /// - `on_click_closure` - A closure that will be called when the button is clicked. The closure
    ///   takes a `bool` argument indicating whether the button is now active (selected) or not.
    pub fn child<F, C>(&mut self, child: F, on_click: C)
    where
        F: FnOnce(Color) + Send + Sync + 'static,
        C: Fn(bool) + Send + Sync + 'static,
    {
        self.child_closures.push(Box::new(child));
        self.on_click_closures.push(Arc::new(on_click));
    }
}

/// Arguments for the [`button_groups`] component.
#[derive(Builder, Default)]
pub struct ButtonGroupsArgs {
    /// Size of the button group.
    #[builder(default)]
    pub size: ButtonGroupsSize,
    /// Style of the button group.
    #[builder(default)]
    pub style: ButtonGroupsStyle,
    /// Selection mode of the button group.
    #[builder(default)]
    pub selection_mode: ButtonGroupsSelectionMode,
}

#[derive(Clone)]
struct ButtonGroupsLayout {
    container_height: Dp,
    between_space: Dp,
    active_button_shape: Shape,
    inactive_button_shape: Shape,
    inactive_button_shape_start: Shape,
    inactive_button_shape_end: Shape,
}

impl ButtonGroupsLayout {
    fn new(size: ButtonGroupsSize, style: ButtonGroupsStyle) -> Self {
        // See https://m3.material.io/components/button-groups/specs#f41a7d35-b9c2-4340-b3bb-47b34acaaf45
        let container_height = match size {
            ButtonGroupsSize::ExtraSmall => Dp(32.0),
            ButtonGroupsSize::Small => Dp(40.0),
            ButtonGroupsSize::Medium => Dp(56.0),
            ButtonGroupsSize::Large => Dp(96.0),
            ButtonGroupsSize::ExtraLarge => Dp(136.0),
        };
        let between_space = match style {
            ButtonGroupsStyle::Standard => match size {
                ButtonGroupsSize::ExtraSmall => Dp(18.0),
                ButtonGroupsSize::Small => Dp(12.0),
                _ => Dp(8.0),
            },
            ButtonGroupsStyle::Connected => Dp(2.0),
        };
        let active_button_shape = match style {
            ButtonGroupsStyle::Standard => Shape::rounded_rectangle(Dp(16.0)),
            ButtonGroupsStyle::Connected => Shape::capsule(),
        };
        let inactive_button_shape = match style {
            ButtonGroupsStyle::Standard => Shape::capsule(),
            ButtonGroupsStyle::Connected => Shape::rounded_rectangle(Dp(16.0)),
        };
        let inactive_button_shape_start = match style {
            ButtonGroupsStyle::Standard => active_button_shape,
            ButtonGroupsStyle::Connected => Shape::RoundedRectangle {
                top_left: RoundedCorner::Capsule,
                top_right: RoundedCorner::manual(Dp(16.0), 3.0),
                bottom_right: RoundedCorner::manual(Dp(16.0), 3.0),
                bottom_left: RoundedCorner::Capsule,
            },
        };
        let inactive_button_shape_end = match style {
            ButtonGroupsStyle::Standard => active_button_shape,
            ButtonGroupsStyle::Connected => Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(Dp(16.0), 3.0),
                top_right: RoundedCorner::Capsule,
                bottom_right: RoundedCorner::Capsule,
                bottom_left: RoundedCorner::manual(Dp(16.0), 3.0),
            },
        };
        Self {
            container_height,
            between_space,
            active_button_shape,
            inactive_button_shape,
            inactive_button_shape_start,
            inactive_button_shape_end,
        }
    }
}

#[derive(Default, Clone)]
struct ButtonItemState {
    ripple_state: RippleState,
    actived: Arc<AtomicBool>,
    elastic_state: Arc<RwLock<ElasticState>>,
}

/// State of a button group.
#[derive(Clone, Default)]
pub struct ButtonGroupsState {
    item_states: Arc<RwLock<HashMap<usize, ButtonItemState>>>,
}

/// # button_groups
///
/// Button groups organize buttons and add interactions between them.
///
/// ## Usage
///
/// Used for grouping related actions.
///
/// ## Arguments
///
/// - `args` - Arguments for configuring the button group.
/// - `state` - State of the button group.
/// - `scope_config` - A closure that configures the children of the button group using
///   a [`ButtonGroupsScope`].
///
/// # Example
///
/// ```
/// use tessera_ui_basic_components::{
///    button_groups::{ButtonGroupsArgs, ButtonGroupsState, button_groups},
///    text::{TextArgs, text},
/// };
///
/// let button_groups_state = ButtonGroupsState::default();
/// button_groups(
///     ButtonGroupsArgs::default(),
///     button_groups_state.clone(),
///     |scope| {
///         scope.child(
///             |color| {
///                 text(TextArgs {
///                     text: "Button 1".to_string(),
///                     color,
///                     ..Default::default()
///                 })
///             },
///             |_| {
///                 println!("Button 1 clicked");
///             },
///         );
///
///         scope.child(
///             |color| {
///                 text(TextArgs {
///                     text: "Button 2".to_string(),
///                     color,
///                     ..Default::default()
///                 })
///             },
///             |actived| {
///                 println!("Button 2 clicked");
///             },
///         );
///
///         scope.child(
///             |color| {
///                 text(TextArgs {
///                     text: "Button 3".to_string(),
///                     color,
///                     ..Default::default()
///                 })
///             },
///             |actived| {
///                 println!("Button 3 clicked");
///             },
///         );
///     },
/// );
/// ```
#[tessera]
pub fn button_groups<F>(
    args: impl Into<ButtonGroupsArgs>,
    state: ButtonGroupsState,
    scope_config: F,
) where
    F: FnOnce(&mut ButtonGroupsScope),
{
    let args = args.into();
    let mut child_closures = Vec::new();
    let mut on_click_closures = Vec::new();
    {
        let mut scope = ButtonGroupsScope {
            child_closures: &mut child_closures,
            on_click_closures: &mut on_click_closures,
        };
        scope_config(&mut scope);
    }
    let layout = ButtonGroupsLayout::new(args.size, args.style);
    let child_len = child_closures.len();
    let selection_mode = args.selection_mode;
    row(
        RowArgs {
            height: layout.container_height.into(),
            main_axis_alignment: MainAxisAlignment::SpaceBetween,
            ..Default::default()
        },
        closure!(
            clone state,
            |scope| {
                for (index, child_closure) in child_closures.into_iter().enumerate() {
                    let on_click_closure = on_click_closures[index].clone();
                    let item_state = state.item_states.write().entry(index).or_default().clone();

                    scope.child(
                        closure!(clone state, clone layout, || {
                            let ripple_state = item_state.ripple_state.clone();
                            let actived = item_state.actived.load(Ordering::Acquire);
                            let elastic_state = item_state.elastic_state.clone();
                            if actived {
                                let mut button_args = ButtonArgs::filled(
                                    Arc::new(move || {
                                        on_click_closure(false);
                                        item_state.actived.store(false, Ordering::Release);
                                        item_state.elastic_state.write().toggle();
                                    })
                                );
                                button_args.shape = layout.active_button_shape;
                                button(button_args, ripple_state, || elastic_container(elastic_state, move || child_closure(global_material_scheme().on_primary)));
                            } else {
                                let mut button_args = ButtonArgs::filled(
                                    Arc::new(move || {
                                        on_click_closure(true);
                                        if selection_mode == ButtonGroupsSelectionMode::Single {
                                            // Deactivate all other buttons if in single selection mode
                                            for item in state.item_states.read().values() {
                                                if item.actived.load(Ordering::Acquire) {
                                                    item.actived.store(false, Ordering::Release);
                                                    item.elastic_state.write().toggle();
                                                }
                                            }
                                        }
                                        item_state.actived.store(true, Ordering::Release);
                                        item_state.elastic_state.write().toggle();
                                    })
                                );
                                button_args.color = global_material_scheme().secondary_container;
                                if index == 0 {
                                    button_args.shape = layout.inactive_button_shape_start;
                                } else if index == child_len - 1 {
                                    button_args.shape = layout.inactive_button_shape_end;
                                } else {
                                    button_args.shape = layout.inactive_button_shape;
                                }

                                button(button_args, ripple_state, move || elastic_container(
                                    elastic_state,
                                    move || child_closure(global_material_scheme().on_secondary_container))
                                );
                            }
                        })
                    );
                    if index != child_len - 1 {
                        scope.child(move || {
                            spacer(SpacerArgs {
                                width: layout.between_space.into(),
                                ..Default::default()
                            });
                        })
                    }
                }
            }
        ),
    )
}

struct ElasticState {
    expended: bool,
    last_toggle: Option<Instant>,
    start_progress: f32,
}

impl Default for ElasticState {
    fn default() -> Self {
        Self {
            expended: false,
            last_toggle: None,
            start_progress: 0.0,
        }
    }
}

impl ElasticState {
    fn toggle(&mut self) {
        let current_visual_progress = self.calculate_current_progress();
        self.expended = !self.expended;
        self.last_toggle = Some(Instant::now());
        self.start_progress = current_visual_progress;
    }

    fn update(&mut self) -> f32 {
        let current_progress = self.calculate_current_progress();
        if self.expended {
            animation::spring(current_progress, 15.0, 0.35)
        } else {
            animation::easing(current_progress)
        }
    }

    fn calculate_current_progress(&self) -> f32 {
        let Some(last_toggle) = self.last_toggle else {
            return if self.expended { 1.0 } else { 0.0 };
        };

        let elapsed = last_toggle.elapsed().as_secs_f32();
        let duration = 0.25;
        let t = (elapsed / duration).clamp(0.0, 1.0);
        let start = self.start_progress;
        let target = if self.expended { 1.0 } else { 0.0 };

        start + (target - start) * t
    }
}

#[tessera]
fn elastic_container(state: Arc<RwLock<ElasticState>>, child: impl FnOnce()) {
    child();
    let progress = state.write().update();
    measure(Box::new(move |input| {
        let child_id = input.children_ids[0];
        let child_size = input.measure_child(child_id, input.parent_constraint)?;
        let additional_width = child_size.width.mul_f32(0.15 * progress);
        input.place_child(child_id, PxPosition::new(additional_width / 2, Px::ZERO));

        Ok(ComputedData {
            width: child_size.width + additional_width,
            height: child_size.height,
        })
    }))
}
