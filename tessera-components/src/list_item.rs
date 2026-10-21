//! Material Design list item component.
//!
//! ## Usage
//!
//! Present rows of content in settings, inboxes, or selection lists.

use tessera_ui::{
    Callback, CallbackWith, Color, Dp, Modifier, RenderSlot, State, current_frame_nanos,
    layout::layout, modifier::ModifierCapabilityExt as _, provide_context, receive_frame_nanos,
    remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    animation::shape_motion::ShapeMotion,
    boxed::boxed,
    icon::IconSize,
    modifier::{InteractionState, ModifierExt as _, Padding},
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    text::text,
    theme::{ContentColor, MaterialAlpha, MaterialColorScheme, MaterialTheme, TextStyle},
};

use item_layout::ListItemLayout;
use selection::{ListItemActivation, ListItemSelectionSemantics};

pub use selection::ListItemSelectionMode;
pub use shapes::{ListItemPosition, ListItemShapes};

mod color_motion;
mod item_layout;
mod selection;
mod shapes;

#[cfg(test)]
mod tests;

#[cfg(all(test, not(target_family = "wasm")))]
mod color_tests;
#[cfg(all(test, not(target_family = "wasm")))]
mod long_click_tests;
#[cfg(all(test, not(target_family = "wasm")))]
mod native_tests;
#[cfg(all(test, not(target_family = "wasm")))]
mod reflow_tests;
#[cfg(all(test, not(target_family = "wasm")))]
mod selection_tests;
#[cfg(all(test, not(target_family = "wasm")))]
mod shape_tests;

#[cfg(test)]
thread_local! {
    static ITEM_BUILDS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Vertical placement of content inside a list row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListItemVerticalAlignment {
    /// Center shorter rows and top-align taller rows.
    #[default]
    Auto,
    /// Align all content at the top inset.
    Top,
    /// Center all content vertically.
    Center,
    /// Align all content at the bottom inset.
    Bottom,
}

/// Colors used by list items in different states.
#[derive(Clone, PartialEq, Copy, Debug)]
pub struct ListItemColors {
    /// Container color when enabled and not selected.
    pub container_color: Color,
    /// Headline text color when enabled and not selected.
    pub headline_color: Color,
    /// Overline text color when enabled and not selected.
    pub overline_color: Color,
    /// Supporting text color when enabled and not selected.
    pub supporting_color: Color,
    /// Leading slot content color when enabled and not selected.
    pub leading_color: Color,
    /// Trailing slot content color when enabled and not selected.
    pub trailing_color: Color,
    /// Container color when disabled.
    pub disabled_container_color: Color,
    /// Headline text color when disabled.
    pub disabled_headline_color: Color,
    /// Overline text color when disabled.
    pub disabled_overline_color: Color,
    /// Supporting text color when disabled.
    pub disabled_supporting_color: Color,
    /// Leading slot content color when disabled.
    pub disabled_leading_color: Color,
    /// Trailing slot content color when disabled.
    pub disabled_trailing_color: Color,
    /// Container color when selected.
    pub selected_container_color: Color,
    /// Headline text color when selected.
    pub selected_headline_color: Color,
    /// Overline text color when selected.
    pub selected_overline_color: Color,
    /// Supporting text color when selected.
    pub selected_supporting_color: Color,
    /// Leading slot content color when selected.
    pub selected_leading_color: Color,
    /// Trailing slot content color when selected.
    pub selected_trailing_color: Color,
    /// Container color while dragged.
    pub dragged_container_color: Color,
    /// Headline color while dragged.
    pub dragged_headline_color: Color,
    /// Overline color while dragged.
    pub dragged_overline_color: Color,
    /// Supporting text color while dragged.
    pub dragged_supporting_color: Color,
    /// Leading content color while dragged.
    pub dragged_leading_color: Color,
    /// Trailing content color while dragged.
    pub dragged_trailing_color: Color,
}

impl ListItemColors {
    fn container_color(self, enabled: bool, selected: bool, dragged: bool) -> Color {
        if !enabled {
            self.disabled_container_color
        } else if dragged {
            self.dragged_container_color
        } else if selected {
            self.selected_container_color
        } else {
            self.container_color
        }
    }

    fn headline_color(self, enabled: bool, selected: bool, dragged: bool) -> Color {
        if !enabled {
            self.disabled_headline_color
        } else if dragged {
            self.dragged_headline_color
        } else if selected {
            self.selected_headline_color
        } else {
            self.headline_color
        }
    }

    fn overline_color(self, enabled: bool, selected: bool, dragged: bool) -> Color {
        if !enabled {
            self.disabled_overline_color
        } else if dragged {
            self.dragged_overline_color
        } else if selected {
            self.selected_overline_color
        } else {
            self.overline_color
        }
    }

    fn supporting_color(self, enabled: bool, selected: bool, dragged: bool) -> Color {
        if !enabled {
            self.disabled_supporting_color
        } else if dragged {
            self.dragged_supporting_color
        } else if selected {
            self.selected_supporting_color
        } else {
            self.supporting_color
        }
    }

    fn leading_color(self, enabled: bool, selected: bool, dragged: bool) -> Color {
        if !enabled {
            self.disabled_leading_color
        } else if dragged {
            self.dragged_leading_color
        } else if selected {
            self.selected_leading_color
        } else {
            self.leading_color
        }
    }

    fn trailing_color(self, enabled: bool, selected: bool, dragged: bool) -> Color {
        if !enabled {
            self.disabled_trailing_color
        } else if dragged {
            self.dragged_trailing_color
        } else if selected {
            self.selected_trailing_color
        } else {
            self.trailing_color
        }
    }
}

/// Default tokens for list item components.
pub struct ListItemDefaults;

impl ListItemDefaults {
    /// Minimum height for a one-line list item container.
    pub const MIN_HEIGHT_ONE_LINE: Dp = Dp(56.0);
    /// Minimum height for a two-line list item container.
    pub const MIN_HEIGHT_TWO_LINE: Dp = Dp(72.0);
    /// Minimum height for a three-line list item container.
    pub const MIN_HEIGHT_THREE_LINE: Dp = Dp(88.0);
    /// Minimum size for leading content slots.
    pub const LEADING_MIN_SIZE: Dp = Dp(24.0);
    /// Minimum size for trailing content slots.
    pub const TRAILING_MIN_SIZE: Dp = Dp(24.0);
    /// Default icon size within an Expressive list item.
    pub const ICON_SIZE: Dp = Dp(20.0);
    /// Vertical gap between segmented list items.
    pub const SEGMENTED_GAP: Dp = Dp(2.0);
    /// Spacing between leading/trailing slots and the main text column.
    pub const INTERNAL_SPACING: Dp = Dp(12.0);
    /// Default content padding applied inside the list item container.
    pub const CONTENT_PADDING: Padding = Padding::new(Dp(16.0), Dp(10.0), Dp(16.0), Dp(10.0));
    /// Default tonal elevation for list items.
    pub const TONAL_ELEVATION: Dp = Dp(0.0);
    /// Default shadow elevation for list items.
    pub const SHADOW_ELEVATION: Dp = Dp(0.0);

    /// Default container shape for list items.
    pub fn shape() -> Shape {
        let corner = RoundedCorner::manual(Dp(4.0), 2.0);
        Shape::RoundedRectangle {
            top_left: corner,
            top_right: corner,
            bottom_right: corner,
            bottom_left: corner,
        }
    }

    /// Default colors for list items derived from the current theme.
    pub fn colors(scheme: &MaterialColorScheme) -> ListItemColors {
        let scheme = scheme.clone();
        let disabled = scheme
            .on_surface
            .with_alpha(MaterialAlpha::DISABLED_CONTENT);
        ListItemColors {
            container_color: scheme.surface,
            headline_color: scheme.on_surface,
            overline_color: scheme.on_surface_variant,
            supporting_color: scheme.on_surface_variant,
            leading_color: scheme.on_surface_variant,
            trailing_color: scheme.on_surface_variant,
            disabled_container_color: scheme.surface,
            disabled_headline_color: disabled,
            disabled_overline_color: disabled,
            disabled_supporting_color: disabled,
            disabled_leading_color: disabled,
            disabled_trailing_color: disabled,
            selected_container_color: scheme.secondary_container,
            selected_headline_color: scheme.on_secondary_container,
            selected_overline_color: scheme.on_secondary_container,
            selected_supporting_color: scheme.on_secondary_container,
            selected_leading_color: scheme.on_secondary_container,
            selected_trailing_color: scheme.on_secondary_container,
            dragged_container_color: scheme.tertiary_container,
            dragged_headline_color: scheme.on_tertiary_container,
            dragged_overline_color: scheme.on_tertiary_container,
            dragged_supporting_color: scheme.on_tertiary_container,
            dragged_leading_color: scheme.on_tertiary_container,
            dragged_trailing_color: scheme.on_tertiary_container,
        }
    }

    /// Compute the minimum container height based on content lines.
    pub fn min_height(has_overline: bool, has_supporting: bool) -> Dp {
        if has_overline && has_supporting {
            Self::MIN_HEIGHT_THREE_LINE
        } else if has_overline || has_supporting {
            Self::MIN_HEIGHT_TWO_LINE
        } else {
            Self::MIN_HEIGHT_ONE_LINE
        }
    }
}

/// # list_item
///
/// Render an adaptive Material list row for settings, inboxes, and selection
/// lists.
///
/// ## Usage
///
/// Present list rows in settings, inboxes, or preference screens.
///
/// ## Parameters
///
/// - `modifier` — optional modifier chain applied to the list item container.
/// - `enabled` — optional enabled state; defaults to `true`.
/// - `selected` — externally controlled selection or checked state.
/// - `selection_mode` — action, single-selection, or multiple-selection
///   behavior. The owner maintains exclusivity between single-selection rows.
/// - `on_selection_change` — requested selection in Single/Multiple modes;
///   takes precedence over `on_click`. Read latest application state in
///   callbacks.
/// - `headline` — primary headline text when `content` is absent.
/// - `content` — optional custom main content, replacing `headline`.
/// - `overline_content` — optional custom content replacing `overline_text`.
/// - `supporting_content` — optional custom content replacing
///   `supporting_text`. Custom slots inherit their role typography and color.
///   Supply explicit accessibility labels and descriptions when their meaning
///   is not text.
/// - `overline_text` — optional overline text.
/// - `supporting_text` — optional supporting text.
/// - `leading` — optional leading slot.
/// - `trailing` — optional trailing slot.
/// - `colors` — optional color palette override.
/// - `shape` — optional container shape override. A fixed shape overrides
///   position and state shapes.
/// - `shapes` — optional shapes for resting and active states.
/// - `position` — row position within a segmented list; defaults to standalone.
///   Place segmented rows with `ListItemDefaults::SEGMENTED_GAP` between them.
/// - `dragged` — apply drag colors and shape during an externally managed drag.
/// - `content_padding` — optional inner padding override.
/// - `tonal_elevation` — optional tonal elevation override.
/// - `shadow_elevation` — optional shadow elevation override.
/// - `min_height` — optional minimum height override.
/// - `vertical_alignment` — content alignment; short rows center by default and
///   taller rows align to their top inset.
/// - `on_long_click` — optional long-press action; successful holds suppress
///   ordinary clicks and selection requests. Also exposed as a context action.
/// - `on_click` — action callback, also used as a selection-mode fallback when
///   `on_selection_change` is absent.
/// - `interaction_state` — optional shared interaction state.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     list_item::{ListItemDefaults, ListItemSelectionMode, list_item},
///     text::text,
/// };
/// use tessera_ui::{Dp, remember, tessera};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// #[tessera]
/// fn demo() {
///     let selected = remember(|| false);
///     material_theme()
///         .theme(|| MaterialTheme::default())
///         .child(move || {
///             list_item()
///                 .selection_mode(ListItemSelectionMode::Multiple)
///                 .selected(selected.get())
///                 .on_selection_change(move |value| selected.set(value))
///                 .accessibility_label("Inbox")
///                 .content(|| {
///                     text().content("Inbox");
///                 })
///                 .supporting_text("3 new messages");
///         });
/// }
///
/// demo();
/// assert_eq!(ListItemDefaults::min_height(false, true), Dp(72.0));
/// ```
#[tessera]
pub fn list_item(
    modifier: Option<Modifier>,
    enabled: Option<bool>,
    selected: Option<bool>,
    selection_mode: Option<ListItemSelectionMode>,
    on_selection_change: Option<CallbackWith<bool>>,
    #[prop(into)] headline: Option<String>,
    content: Option<RenderSlot>,
    overline_content: Option<RenderSlot>,
    supporting_content: Option<RenderSlot>,
    #[prop(into)] overline_text: Option<String>,
    #[prop(into)] supporting_text: Option<String>,
    leading: Option<RenderSlot>,
    trailing: Option<RenderSlot>,
    colors: Option<ListItemColors>,
    shape: Option<Shape>,
    shapes: Option<ListItemShapes>,
    position: Option<ListItemPosition>,
    dragged: Option<bool>,
    content_padding: Option<Padding>,
    tonal_elevation: Option<Dp>,
    shadow_elevation: Option<Dp>,
    min_height: Option<Dp>,
    vertical_alignment: Option<ListItemVerticalAlignment>,
    on_click: Option<Callback>,
    on_long_click: Option<Callback>,
    interaction_state: Option<State<InteractionState>>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    #[cfg(test)]
    ITEM_BUILDS.set(ITEM_BUILDS.get() + 1);
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let typography = theme.typography;
    let selection_mode = selection_mode.unwrap_or_default();
    let modifier = modifier
        .unwrap_or_default()
        .push_semantics(ListItemSelectionSemantics {
            mode: selection_mode,
            selected,
        });
    let enabled = enabled.unwrap_or(true);
    let selected = selected.unwrap_or(false);
    let action = ListItemActivation {
        mode: selection_mode,
        selected,
        on_click,
        on_selection_change,
    };
    let has_on_click = action.has_handler();
    let activation = remember(|| action.clone());
    if activation.get() != action {
        activation.set(action);
    }
    let on_activate = Callback::new(move || activation.get().activate());
    let dragged = dragged.unwrap_or(false);
    let headline = headline.unwrap_or_default();
    let colors = colors.unwrap_or_else(|| ListItemDefaults::colors(&theme.color_scheme));
    let interaction = if let Some(interaction) = interaction_state {
        interaction
    } else {
        remember(InteractionState::new)
    };
    let interactive = enabled && (has_on_click || on_long_click.is_some());
    let shapes = shapes
        .unwrap_or_else(|| ListItemShapes::from_theme(&theme))
        .segmented(position.unwrap_or_default(), theme.shapes.large);
    let target = if let Some(shape) = shape {
        shape
    } else if interactive && interaction.with(InteractionState::is_pressed) {
        shapes.pressed_shape
    } else if dragged && enabled {
        shapes.dragged_shape
    } else if selected {
        shapes.selected_shape
    } else if interactive && interaction.with(InteractionState::is_focused) {
        shapes.focused_shape
    } else if interactive && interaction.with(InteractionState::is_hovered) {
        shapes.hovered_shape
    } else {
        shapes.shape
    };
    let motion = remember(|| ShapeMotion::new(target));
    let mut next = motion.get();
    next.retarget(target, enabled, current_frame_nanos());
    if next != motion.get() {
        motion.set(next);
    }
    let content_padding = content_padding.unwrap_or(ListItemDefaults::CONTENT_PADDING);
    let tonal_elevation = tonal_elevation.unwrap_or(ListItemDefaults::TONAL_ELEVATION);
    let shadow_elevation = shadow_elevation.unwrap_or(ListItemDefaults::SHADOW_ELEVATION);

    let overline_text = overline_text.filter(|value| !value.is_empty());
    let supporting_text = supporting_text.filter(|value| !value.is_empty());
    let has_overline = overline_text.is_some() || overline_content.is_some();
    let has_supporting = supporting_text.is_some() || supporting_content.is_some();

    let target_colors = [
        colors.container_color(enabled, selected, dragged),
        colors.headline_color(enabled, selected, dragged),
        colors.overline_color(enabled, selected, dragged),
        colors.supporting_color(enabled, selected, dragged),
        colors.leading_color(enabled, selected, dragged),
        colors.trailing_color(enabled, selected, dragged),
    ];
    let color_motion = remember(|| color_motion::ListColorsMotion::new(target_colors));
    let mut next_colors = color_motion.get();
    next_colors.retarget(target_colors, current_frame_nanos());
    if next_colors != color_motion.get() {
        color_motion.set(next_colors);
    }
    if next.is_animating() || next_colors.is_animating() {
        receive_frame_nanos(move |frame| {
            let mut next = motion.get();
            next.advance(frame);
            if next != motion.get() {
                motion.set(next);
            }
            let mut next_colors = color_motion.get();
            next_colors.advance(frame);
            if next_colors != color_motion.get() {
                color_motion.set(next_colors);
            }
            if next.is_animating() || next_colors.is_animating() {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }
    let [
        container_color,
        headline_color,
        overline_color,
        supporting_color,
        leading_color,
        trailing_color,
    ] = next_colors.value();

    let accessibility_label = accessibility_label
        .or_else(|| (content.is_none() && !headline.is_empty()).then(|| headline.clone()));
    let accessibility_description = accessibility_description
        .or_else(|| {
            if supporting_content.is_none() {
                supporting_text.clone()
            } else {
                None
            }
        })
        .or_else(|| {
            if overline_content.is_none() {
                overline_text.clone()
            } else {
                None
            }
        });
    let elevation = (shadow_elevation.0 > 0.0).then_some(shadow_elevation);

    surface()
        .modifier(modifier)
        .style(SurfaceStyle::Filled {
            color: container_color,
        })
        .shape(motion.get().shape())
        .content_color(headline_color)
        .enabled(enabled)
        .ripple_color(headline_color)
        .tonal_elevation(tonal_elevation)
        .accessibility_role(selection_mode.role())
        .elevation_optional(elevation)
        .interaction_state(interaction)
        .on_click_optional(has_on_click.then_some(on_activate))
        .on_long_click_optional(on_long_click)
        .accessibility_focusable(interactive)
        .accessibility_label_optional(accessibility_label)
        .accessibility_description_optional(accessibility_description)
        .child(move || {
            let headline = headline.clone();
            let overline_text = overline_text.clone();
            let supporting_text = supporting_text.clone();
            layout()
                .policy(ListItemLayout {
                    padding: content_padding,
                    min_height,
                    has_overline,
                    has_supporting,
                    supporting_line_height: typography
                        .body_medium
                        .line_height
                        .unwrap_or(Dp(typography.body_medium.font_size.0 * 1.2)),
                    vertical_alignment: vertical_alignment.unwrap_or_default(),
                })
                .child(move || {
                    list_item_slot()
                        .content_optional(leading)
                        .color(leading_color)
                        .style(typography.title_medium)
                        .leading(true);
                    list_item_slot()
                        .content_optional(trailing)
                        .color(trailing_color)
                        .style(typography.label_small);
                    list_item_text_line()
                        .text_value_optional(overline_text.clone())
                        .content_optional(overline_content)
                        .style(typography.label_small)
                        .color(overline_color);
                    list_item_text_line()
                        .text_value(headline.clone())
                        .content_optional(content)
                        .style(typography.body_large)
                        .color(headline_color);
                    list_item_text_line()
                        .text_value_optional(supporting_text.clone())
                        .content_optional(supporting_content)
                        .style(typography.body_medium)
                        .color(supporting_color);
                });
        });
}

#[tessera]
fn list_item_text_line(
    #[prop(into)] text_value: Option<String>,
    content: Option<RenderSlot>,
    style: Option<crate::theme::TextStyle>,
    color: Option<Color>,
) {
    let text_value = text_value.unwrap_or_default();
    if let Some(content) = content {
        list_item_slot()
            .content_shared(content)
            .color_optional(color)
            .style_optional(style)
            .main(true);
    } else if text_value.is_empty() {
        spacer();
    } else {
        text()
            .content(text_value)
            .style(style.unwrap_or_default())
            .color(color.unwrap_or(Color::TRANSPARENT));
    }
}

#[tessera]
fn list_item_slot(
    content: Option<RenderSlot>,
    color: Option<Color>,
    style: Option<TextStyle>,
    leading: Option<bool>,
    main: Option<bool>,
) {
    let color = color.unwrap_or(Color::TRANSPARENT);
    let style = style.unwrap_or_default();
    let icon_size = ListItemDefaults::ICON_SIZE;
    provide_context(
        || ContentColor { current: color },
        move || {
            let current = use_context::<ContentColor>().expect("list item content color");
            if current.get().current != color {
                current.set(ContentColor { current: color });
            }
            provide_context(
                || style,
                move || {
                    let current = use_context::<TextStyle>().expect("list item slot typography");
                    if current.get() != style {
                        current.set(style);
                    }
                    provide_context(
                        || IconSize(icon_size),
                        move || {
                            let current = use_context::<IconSize>().expect("list item icon size");
                            if current.get().0 != icon_size {
                                current.set(IconSize(icon_size));
                            }
                            let padding = if content.is_none() || main.unwrap_or(false) {
                                Padding::default()
                            } else if leading.unwrap_or(false) {
                                Padding::new(
                                    Dp::ZERO,
                                    Dp::ZERO,
                                    ListItemDefaults::INTERNAL_SPACING,
                                    Dp::ZERO,
                                )
                            } else {
                                Padding::new(
                                    ListItemDefaults::INTERNAL_SPACING,
                                    Dp::ZERO,
                                    Dp::ZERO,
                                    Dp::ZERO,
                                )
                            };
                            boxed()
                                .alignment(if main.unwrap_or(false) {
                                    Alignment::TopStart
                                } else {
                                    Alignment::Center
                                })
                                .modifier(Modifier::new().padding(padding))
                                .children(move || {
                                    if let Some(content) = content {
                                        content.render();
                                    }
                                });
                        },
                    );
                },
            );
        },
    );
}
