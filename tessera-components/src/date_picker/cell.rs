//! Calendar day and year choices for custom date-selection layouts.
//!
//! ## Usage
//!
//! Compose selectable calendar cells in scheduling and date-entry controls.

use tessera_ui::{
    CallbackWith, Color, ComputedData, Constraint, Dp, FocusRequester, LayoutResult,
    MeasurementError, Modifier, Px, PxPosition, PxSize, State,
    layout::{MeasureScope, layout},
    modifier::ModifierCapabilityExt as _,
    remember, tessera, use_context,
};

use crate::{
    modifier::{
        InteractionState, InteractionVisualState, ModifierExt as _, SelectableArgs, SemanticsArgs,
    },
    pipelines::shape::command::ShapeCommand,
    shape_def::{ResolvedShape, Shape},
    surface::record_focus_indicator,
    text::text,
    theme::{MaterialAlpha, MaterialTheme},
};

use super::{
    CalendarDate, CalendarRangeBand, DATE_CELL_SIZE, DATE_VISUAL_SIZE,
    focus::CalendarFocusIndicator, format_month_name,
};

/// Position of a date within the continuous band of an inclusive range.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CalendarRangePosition {
    /// A range with the same starting and ending date.
    #[default]
    Isolated,
    /// The first date, with a band extending toward the following date.
    Start,
    /// An interior date, with a band extending in both directions.
    Middle,
    /// The last date, with a band extending toward the preceding date.
    End,
}

impl CalendarRangePosition {
    fn band(self) -> CalendarRangeBand {
        CalendarRangeBand {
            start: matches!(self, Self::Isolated | Self::Start),
            end: matches!(self, Self::Isolated | Self::End),
        }
    }
}

/// # date_picker_day
///
/// Render an interactive calendar date for custom calendars and scheduling
/// controls.
///
/// ## Usage
///
/// Arrange real date choices in a calendar, booking flow, or compact date
/// selector.
///
/// ## Parameters
///
/// - `date`, `on_select` — date represented by the cell and its activation
///   callback.
/// - `selected`, `is_today`, `in_range`, `outside_month`, `range_position` —
///   selection and range appearance.
/// - `enabled`, `presentation_state` — interaction availability and optional
///   visual state.
/// - `modifier`, `focus_requester` — layout, keyboard behavior, and external
///   focus control.
/// - `label_formatter` — optional visible day text; accessibility retains the
///   actual date.
/// - `accessibility_label`, `accessibility_description` — optional spoken date
///   information.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     date_picker::{CalendarDate, date_picker_day},
///     theme::material_theme,
/// };
/// use tessera_ui::{remember, tessera};
/// #[tessera]
/// fn schedule() {
///     material_theme().child(|| {
///         let date = CalendarDate::new(2025, 8, 17).unwrap();
///         let selected = remember(|| date);
///         date_picker_day(date)
///             .selected(selected.get() == date)
///             .on_select(move |date| selected.set(date));
///         assert_eq!(selected.get(), date);
///     });
/// }
/// schedule();
/// ```
#[tessera]
pub fn date_picker_day(
    date: CalendarDate,
    on_select: Option<CallbackWith<CalendarDate>>,
    selected: Option<bool>,
    is_today: Option<bool>,
    in_range: Option<bool>,
    outside_month: Option<bool>,
    range_position: Option<CalendarRangePosition>,
    enabled: Option<bool>,
    presentation_state: Option<InteractionVisualState>,
    modifier: Option<Modifier>,
    focus_requester: Option<FocusRequester>,
    label_formatter: Option<CallbackWith<CalendarDate, String>>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let on_select = on_select.unwrap_or_else(|| CallbackWith::new(|_| {}));
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    let selected = selected.unwrap_or(false);
    let enabled = enabled.unwrap_or(true);
    let is_today = is_today.unwrap_or(false);
    let interaction = remember(InteractionState::new);
    let internal_focus = remember(FocusRequester::new).get();
    let focus = focus_requester.unwrap_or(internal_focus);
    let color = if selected {
        scheme.on_primary
    } else if in_range.unwrap_or(false) {
        scheme.on_secondary_container
    } else if is_today {
        scheme.primary
    } else if outside_month.unwrap_or(false) {
        scheme.on_surface_variant
    } else {
        scheme.on_surface
    };
    let color = if enabled {
        color
    } else {
        color.with_alpha(MaterialAlpha::DISABLED_CONTENT)
    };
    let visual = CalendarCellVisual {
        shape: Shape::Ellipse,
        fill: if selected {
            scheme.primary
        } else {
            Color::TRANSPARENT
        },
        outline: (!selected && is_today).then_some(scheme.primary),
        content_color: color,
        focus_color: scheme.secondary,
        interaction,
        presentation_state,
        enabled,
        disabled_focus: !enabled && focus.is_focused(),
    };
    let range_band = range_position.map(CalendarRangePosition::band);
    let label =
        label_formatter.map_or_else(|| date.day().to_string(), |formatter| formatter.call(date));
    layout()
        .modifier(
            modifier
                .unwrap_or_default()
                .size(DATE_CELL_SIZE, DATE_CELL_SIZE)
                .selectable_with(SelectableArgs {
                    selected,
                    on_click: tessera_ui::Callback::new(move || on_select.call(date)),
                    enabled,
                    interaction_state: Some(interaction),
                    focus_requester: Some(focus),
                    label: Some(accessibility_label.unwrap_or_else(|| {
                        format!(
                            "{} {}, {}",
                            format_month_name(date.month()),
                            date.day(),
                            date.year()
                        )
                    })),
                    description: accessibility_description,
                    ..Default::default()
                }),
        )
        .policy(CalendarDayLayout {
            fractional_center: false,
            range_band,
        })
        .child(move || {
            let label = label.clone();
            if range_band.is_some() {
                layout().modifier(Modifier::new().background(scheme.secondary_container));
            }
            layout()
                .modifier(
                    Modifier::new()
                        .size(DATE_VISUAL_SIZE, DATE_VISUAL_SIZE)
                        .push_draw(CalendarFocusIndicator {
                            visible: focus.is_focus_visible(),
                            shape: Shape::Ellipse,
                            color: scheme.secondary,
                        }),
                )
                .policy(CalendarDayLayout {
                    fractional_center: true,
                    range_band: None,
                })
                .draw(visual)
                .child(move || {
                    text()
                        .content(label.clone())
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            hidden: true,
                            ..Default::default()
                        }))
                        .style(theme.typography.body_large)
                        .color(color);
                });
        });
}

/// # date_picker_year
///
/// Render an interactive year choice for calendars and year-selection controls.
///
/// ## Usage
///
/// Arrange year choices in scheduling, date entry, and historical date
/// selectors.
///
/// ## Parameters
///
/// - `year`, `on_select` — represented year and its activation callback.
/// - `selected`, `is_today`, `enabled` — selection, current-year appearance,
///   and availability.
/// - `presentation_state` — optional appearance that preserves real interaction
///   and focus.
/// - `modifier`, `focus_requester` — layout, keyboard behavior, and external
///   focus control.
/// - `label_formatter` — optional visible year text; accessibility retains the
///   actual year.
/// - `accessibility_label`, `accessibility_description` — optional spoken year
///   information.
///
/// ## Examples
///
/// ```
/// use tessera_components::{date_picker::date_picker_year, theme::material_theme};
/// use tessera_ui::{remember, tessera};
/// #[tessera]
/// fn years() {
///     material_theme().child(|| {
///         let selected = remember(|| 2025);
///         date_picker_year(2026)
///             .selected(selected.get() == 2026)
///             .on_select(move |year| selected.set(year));
///         assert_eq!(selected.get(), 2025);
///     });
/// }
/// years();
/// ```
#[tessera]
pub fn date_picker_year(
    year: i32,
    on_select: Option<CallbackWith<i32>>,
    selected: Option<bool>,
    is_today: Option<bool>,
    enabled: Option<bool>,
    presentation_state: Option<InteractionVisualState>,
    modifier: Option<Modifier>,
    focus_requester: Option<FocusRequester>,
    label_formatter: Option<CallbackWith<i32, String>>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let on_select = on_select.unwrap_or_else(|| CallbackWith::new(|_| {}));
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    let selected = selected.unwrap_or(false);
    let enabled = enabled.unwrap_or(true);
    let today = is_today.unwrap_or(false);
    let interaction = remember(InteractionState::new);
    let internal_focus = remember(FocusRequester::new).get();
    let focus = focus_requester.unwrap_or(internal_focus);
    let color = if !enabled {
        scheme
            .on_surface
            .with_alpha(MaterialAlpha::DISABLED_CONTENT)
    } else if selected {
        scheme.on_primary
    } else if today {
        scheme.primary
    } else {
        scheme.on_surface_variant
    };
    let visual = CalendarCellVisual {
        shape: Shape::CAPSULE,
        fill: if selected {
            scheme.primary
        } else {
            Color::TRANSPARENT
        },
        outline: (!selected && today).then_some(scheme.primary),
        content_color: color,
        focus_color: scheme.secondary,
        interaction,
        presentation_state,
        enabled,
        disabled_focus: !enabled && focus.is_focused(),
    };
    let label = label_formatter.map_or_else(|| year.to_string(), |formatter| formatter.call(year));
    layout()
        .modifier(
            modifier
                .unwrap_or_default()
                .size(Dp(112.0), Dp(52.0))
                .selectable_with(SelectableArgs {
                    selected,
                    on_click: tessera_ui::Callback::new(move || on_select.call(year)),
                    enabled,
                    interaction_state: Some(interaction),
                    focus_requester: Some(focus),
                    label: Some(accessibility_label.unwrap_or_else(|| year.to_string())),
                    description: accessibility_description,
                    ..Default::default()
                }),
        )
        .child(move || {
            let label = label.clone();
            layout()
                .modifier(
                    Modifier::new()
                        .size(Dp(72.0), Dp(36.0))
                        .offset(Dp(20.0), Dp(8.0))
                        .push_draw(CalendarFocusIndicator {
                            visible: focus.is_focus_visible(),
                            shape: Shape::CAPSULE,
                            color: scheme.secondary,
                        }),
                )
                .draw(visual)
                .child(move || {
                    let label = label.clone();
                    crate::boxed::boxed()
                        .alignment(crate::alignment::Alignment::Center)
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            text()
                                .content(label.clone())
                                .modifier(Modifier::new().semantics(SemanticsArgs {
                                    hidden: true,
                                    ..Default::default()
                                }))
                                .style(theme.typography.body_large)
                                .color(color);
                        });
                });
        });
}

#[derive(Clone, PartialEq)]
struct CalendarDayLayout {
    fractional_center: bool,
    range_band: Option<CalendarRangeBand>,
}

impl tessera_ui::LayoutModifierNode for CalendarDayLayout {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let width = scope
            .parent_constraint()
            .width()
            .clamp(DATE_CELL_SIZE.to_px());
        let height = scope
            .parent_constraint()
            .height()
            .clamp(DATE_CELL_SIZE.to_px());
        let mut result = LayoutResult::new(ComputedData { width, height });
        for (index, child) in scope.children().into_iter().enumerate() {
            if index == 0
                && let Some(band) = self.range_band
            {
                let visual = DATE_VISUAL_SIZE.to_px().min(height);
                let x = if band.start { width / 2 } else { Px::ZERO };
                let end = if band.end { width / 2 } else { width };
                child.measure(scope, &Constraint::new((end - x).max(Px::ZERO), visual))?;
                result.place_child(child, PxPosition::new(x, (height - visual) / 2));
                continue;
            }
            let size = child.measure(scope, &scope.parent_constraint().without_min())?;
            let position = if self.fractional_center {
                PxPosition::from_f32_xy(
                    (width - size.width).0.max(0) as f32 / 2.0,
                    (height - size.height).0.max(0) as f32 / 2.0,
                )
            } else {
                PxPosition::new((width - size.width) / 2, (height - size.height) / 2)
            };
            result.place_child(child, position);
        }
        Ok(result)
    }
}

#[derive(Clone, Copy, PartialEq)]
struct CalendarCellVisual {
    shape: Shape,
    fill: Color,
    outline: Option<Color>,
    content_color: Color,
    focus_color: Color,
    interaction: State<InteractionState>,
    presentation_state: Option<InteractionVisualState>,
    enabled: bool,
    disabled_focus: bool,
}

impl tessera_ui::DrawModifierNode for CalendarCellVisual {
    fn draw(
        &self,
        ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        let live = self.interaction.get();
        let visual = live.with_presentation(self.presentation_state, self.enabled);
        let alpha = visual.state_layer_alpha().max(if self.disabled_focus {
            MaterialAlpha::FOCUSED
        } else {
            0.0
        });
        let fill = self
            .fill
            .blend_over(self.content_color.with_alpha(1.0), alpha);
        if self.outline.is_some() || fill.a > 0.0 {
            let bounds = ctx
                .render_input
                .metadata_mut()
                .computed_data()
                .expect("calendar cell bounds");
            let command = match self
                .shape
                .resolve_for_size(PxSize::new(bounds.width, bounds.height))
            {
                ResolvedShape::Ellipse => match self.outline {
                    Some(color) if alpha > 0.0 => ShapeCommand::FilledOutlinedEllipse {
                        color: fill,
                        border_color: color,
                        border_width: Dp(1.0).to_pixels_f32(),
                    },
                    Some(color) => ShapeCommand::OutlinedEllipse {
                        color,
                        border_width: Dp(1.0).to_pixels_f32(),
                    },
                    None => ShapeCommand::Ellipse { color: fill },
                },
                ResolvedShape::Rounded {
                    corner_radii,
                    corner_g2,
                } => match self.outline {
                    Some(color) => ShapeCommand::FilledOutlinedRect {
                        color: fill,
                        border_color: color,
                        corner_radii,
                        corner_g2,
                        border_width: Dp(1.0).to_pixels_f32(),
                    },
                    None => ShapeCommand::Rect {
                        color: fill,
                        corner_radii,
                        corner_g2,
                    },
                },
            };
            ctx.render_input
                .metadata_mut()
                .fragment_mut()
                .push_draw_command(command);
        }
        if visual.is_focused() && !live.is_focused() {
            record_focus_indicator(ctx.render_input, self.shape, self.focus_color);
        }
        content.draw(ctx.render_input);
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "cell_tests.rs"]
mod tests;
