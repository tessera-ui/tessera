//! Material 3 button groups for related actions.
//!
//! ## Usage
//!
//! Used for grouping related actions.

use std::{collections::HashMap, time::Instant};

use derive_setters::Setters;
use tessera_ui::{
    CallbackWith, Color, ComputedData, Dp, LayoutInput, LayoutOutput, LayoutSpec, MeasurementError,
    Modifier, Px, PxPosition, RenderSlot, remember, tessera, use_context, with_frame_nanos,
};

use crate::{
    alignment::MainAxisAlignment,
    animation,
    button::{ButtonArgs, button},
    modifier::ModifierExt,
    row::{RowArgs, row},
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    theme::MaterialTheme,
};

/// According to the [`ButtonGroups-Types`](https://m3.material.io/components/button-groups/specs#3b51d175-cc02-4701-b3f8-c9ffa229123a)
/// spec, the [`button_groups`] component supports two styles: `Standard` and
/// `Connected`.
///
/// ## Standard
///
/// Buttons have spacing between them and do not need to be the same width.
///
/// ## Connected
///
/// Buttons are adjacent with no spacing, and each button must be the same
/// width.
#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub enum ButtonGroupsStyle {
    /// Buttons have spacing between them and do not need to be the same width.
    #[default]
    Standard,
    /// Buttons are adjacent with no spacing, and each button must be the same
    /// width.
    Connected,
}

/// According to the [`ButtonGroups-Configurations`](https://m3.material.io/components/button-groups/specs#0d2cf762-275c-4693-9484-fe011501439e)
/// spec, the [`button_groups`] component supports two selection modes: `Single`
/// and `Multiple`.
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
#[derive(Debug, Clone, PartialEq, Copy, Default)]
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
    child_closures: &'a mut Vec<CallbackWith<Color>>,
    on_click_closures: &'a mut Vec<CallbackWith<bool>>,
}

impl ButtonGroupsScope<'_> {
    /// Add a child component to the button group, which will be wrapped by a
    /// [`button`] component.
    ///
    /// # Arguments
    ///
    /// - `child_closure` - A closure that takes a [`Color`] and returns a
    ///   [`button`] component. The `Color` argument should be used for the
    ///   content of the child component.
    /// - `on_click_closure` - A closure that will be called when the button is
    ///   clicked. The closure takes a `bool` argument indicating whether the
    ///   button is now active (selected) or not.
    pub fn child<F, C>(&mut self, child: F, on_click: C)
    where
        F: Fn(Color) + Send + Sync + 'static,
        C: Fn(bool) + Send + Sync + 'static,
    {
        self.child_closures.push(CallbackWith::new(child));
        self.on_click_closures.push(CallbackWith::new(on_click));
    }
}

/// Arguments for the [`button_groups`] component.
#[derive(Clone, PartialEq, Default, Setters)]
pub struct ButtonGroupsArgs {
    /// Size of the button group.
    pub size: ButtonGroupsSize,
    /// Style of the button group.
    pub style: ButtonGroupsStyle,
    /// Selection mode of the button group.
    pub selection_mode: ButtonGroupsSelectionMode,
}

#[derive(Clone, PartialEq)]
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

#[derive(PartialEq, Default)]
struct ButtonItemState {
    actived: bool,
    elastic_state: ElasticState,
}

/// Internal state of a button group.
#[derive(PartialEq, Default)]
struct ButtonGroupsState {
    item_states: HashMap<usize, ButtonItemState>,
}

impl ButtonGroupsState {
    fn item_state_mut(&mut self, index: usize) -> &mut ButtonItemState {
        self.item_states.entry(index).or_default()
    }
}

/// # button_groups
///
/// Button groups organize buttons and add interactions between them.
///
/// ## Usage
///
/// Used for grouping related actions.
///
/// State for selection and animations is managed internally via `remember`; no
/// external state handle is required.
///
/// ## Parameters
///
/// - `args` — configures size, style, and selection mode; see
///   [`ButtonGroupsArgs`].
/// - `scope_config` — closure that configures the children of the button group
///   using a [`ButtonGroupsScope`].
///
/// # Example
///
/// ```
/// use tessera_components::{
///     button_groups::{ButtonGroupsArgs, button_groups},
///     text::{TextArgs, text},
/// };
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # let args = tessera_components::theme::MaterialThemeProviderArgs::new(|| MaterialTheme::default(), || {
/// button_groups(&ButtonGroupsArgs::default(), |scope| {
///     scope.child(
///         |color| {
///             text(&TextArgs {
///                 text: "Button 1".to_string(),
///                 color,
///                 ..Default::default()
///             })
///         },
///         |_| {
///             println!("Button 1 clicked");
///         },
///     );
///
///     scope.child(
///         |color| {
///             text(&TextArgs {
///                 text: "Button 2".to_string(),
///                 color,
///                 ..Default::default()
///             })
///         },
///         |actived| {
///             println!("Button 2 clicked");
///         },
///     );
///
///     scope.child(
///         |color| {
///             text(&TextArgs {
///                 text: "Button 3".to_string(),
///                 color,
///                 ..Default::default()
///             })
///         },
///         |_| {
///             println!("Button 3 clicked");
///         },
///     );
/// });
/// # });
/// # material_theme(&args);
/// ```
pub fn button_groups<F>(args: &ButtonGroupsArgs, scope_config: F)
where
    F: FnOnce(&mut ButtonGroupsScope),
{
    let args = args.clone();
    let mut child_closures = Vec::new();
    let mut on_click_closures = Vec::new();
    {
        let mut scope = ButtonGroupsScope {
            child_closures: &mut child_closures,
            on_click_closures: &mut on_click_closures,
        };
        scope_config(&mut scope);
    }
    let render_args = ButtonGroupsRenderArgs {
        size: args.size,
        style: args.style,
        selection_mode: args.selection_mode,
        child_closures,
        on_click_closures,
    };

    button_groups_node(&render_args);
}

#[derive(Clone, PartialEq)]
struct ButtonGroupsRenderArgs {
    size: ButtonGroupsSize,
    style: ButtonGroupsStyle,
    selection_mode: ButtonGroupsSelectionMode,
    child_closures: Vec<CallbackWith<Color>>,
    on_click_closures: Vec<CallbackWith<bool>>,
}

#[tessera]
fn button_groups_node(args: &ButtonGroupsRenderArgs) {
    let state = remember(ButtonGroupsState::default);
    let child_closures = args.child_closures.clone();
    let on_click_closures = args.on_click_closures.clone();
    let layout = ButtonGroupsLayout::new(args.size, args.style);
    let child_len = child_closures.len();
    let selection_mode = args.selection_mode;
    let row_modifier = RowArgs::default().modifier.height(layout.container_height);
    row(
        RowArgs::default()
            .modifier(row_modifier)
            .main_axis_alignment(MainAxisAlignment::Start),
        move |scope| {
            for (index, child_closure) in child_closures.iter().cloned().enumerate() {
                let on_click_closure = on_click_closures[index].clone();
                let item_layout = layout.clone();
                let between_space = layout.between_space;

                scope.child(move || {
                    let actived =
                        state.with(|s| s.item_states.get(&index).is_some_and(|item| item.actived));
                    if actived {
                        let on_click_closure = on_click_closure.clone();
                        let mut button_args = ButtonArgs::filled(move || {
                            on_click_closure.call(false);
                            state.with_mut(|s| {
                                let item = s.item_state_mut(index);
                                item.actived = false;
                                item.elastic_state.toggle();
                            });
                        });
                        button_args.shape = item_layout.active_button_shape;
                        let scheme = use_context::<MaterialTheme>()
                            .expect("MaterialTheme must be provided")
                            .get()
                            .color_scheme;
                        let label_color = scheme.on_primary;
                        button(&crate::button::ButtonArgs::with_child(button_args, {
                            let child_closure = child_closure.clone();
                            move || {
                                let child_closure = child_closure.clone();
                                let elastic_args = ElasticContainerArgs {
                                    state,
                                    index,
                                    child: RenderSlot::new(move || child_closure.call(label_color)),
                                };
                                elastic_container(&elastic_args);
                            }
                        }));
                    } else {
                        let on_click_closure = on_click_closure.clone();
                        let mut button_args = ButtonArgs::filled(move || {
                            on_click_closure.call(true);
                            state.with_mut(|s| {
                                if selection_mode == ButtonGroupsSelectionMode::Single {
                                    for (other_index, item) in s.item_states.iter_mut() {
                                        if *other_index != index && item.actived {
                                            item.actived = false;
                                            item.elastic_state.toggle();
                                        }
                                    }
                                }

                                let item = s.item_state_mut(index);
                                item.actived = true;
                                item.elastic_state.toggle();
                            });
                        });
                        let scheme = use_context::<MaterialTheme>()
                            .expect("MaterialTheme must be provided")
                            .get()
                            .color_scheme;
                        button_args.color = scheme.secondary_container;
                        if index == 0 {
                            button_args.shape = item_layout.inactive_button_shape_start;
                        } else if index == child_len - 1 {
                            button_args.shape = item_layout.inactive_button_shape_end;
                        } else {
                            button_args.shape = item_layout.inactive_button_shape;
                        }

                        let scheme = use_context::<MaterialTheme>()
                            .expect("MaterialTheme must be provided")
                            .get()
                            .color_scheme;
                        let label_color = scheme.on_secondary_container;
                        button(&crate::button::ButtonArgs::with_child(button_args, {
                            let child_closure = child_closure.clone();
                            move || {
                                let child_closure = child_closure.clone();
                                let elastic_args = ElasticContainerArgs {
                                    state,
                                    index,
                                    child: RenderSlot::new(move || child_closure.call(label_color)),
                                };
                                elastic_container(&elastic_args);
                            }
                        }));
                    }
                });
                if index != child_len - 1 {
                    scope.child(move || {
                        spacer(&crate::spacer::SpacerArgs::new(
                            Modifier::new().width(between_space),
                        ));
                    })
                }
            }
        },
    )
}

#[derive(PartialEq)]
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

    fn is_animating(&self) -> bool {
        self.last_toggle
            .is_some_and(|last_toggle| last_toggle.elapsed().as_secs_f32() < 0.25)
    }
}

#[tessera]
fn elastic_container(args: &ElasticContainerArgs) {
    let frame_tick = remember(|| 0_u64);
    let _ = frame_tick.with(|tick| *tick);

    args.child.render();
    let progress = args
        .state
        .with_mut(|s| s.item_state_mut(args.index).elastic_state.update());

    let should_schedule_frame = args.state.with(|s| {
        s.item_states
            .get(&args.index)
            .is_some_and(|item| item.elastic_state.is_animating())
    });
    if should_schedule_frame {
        let frame_tick_for_frame = frame_tick;
        with_frame_nanos(move |_| {
            frame_tick_for_frame.with_mut(|tick| *tick = tick.wrapping_add(1));
        });
    }

    layout(ElasticContainerLayout { progress })
}

#[derive(Clone, PartialEq)]
struct ElasticContainerArgs {
    state: tessera_ui::State<ButtonGroupsState>,
    index: usize,
    child: RenderSlot,
}

#[derive(Clone, Copy, PartialEq)]
struct ElasticContainerLayout {
    progress: f32,
}

impl LayoutSpec for ElasticContainerLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let child_id = input.children_ids()[0];
        let child_size = input.measure_child_in_parent_constraint(child_id)?;
        let additional_width = child_size.width.mul_f32(0.15 * self.progress);
        output.place_child(child_id, PxPosition::new(additional_width / 2, Px::ZERO));

        Ok(ComputedData {
            width: child_size.width + additional_width,
            height: child_size.height,
        })
    }
}
