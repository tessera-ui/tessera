//! Floating action menus with staggered action expansion.
//!
//! ## Usage
//!
//! Offer related creation actions above a persistent floating action button.

use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui::{
    AxisConstraint, Callback, Color, ComputedData, Constraint, Dp, FocusProperties, FocusRequester,
    FocusScopeNode, FrameNanosControl, LayoutResult, MeasurementError, Modifier, Px, PxPosition,
    RenderSlot, State,
    accesskit::{HasPopup, Role},
    current_frame_nanos,
    layout::{MeasureScope, layout},
    modifier::FocusModifierExt as _,
    receive_frame_nanos, remember, tessera, use_context, winit,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment},
    animation::Animatable,
    boxed::boxed,
    column::column,
    icon::icon,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs, with_keyboard_input, with_pointer_input},
    painter::Painter,
    pos_misc::is_position_in_rect,
    scrollable::ScrollableController,
    shape_def::{RoundedCorner, Shape},
    surface::surface,
    text::text,
    theme::MaterialTheme,
};

/// Shared visibility state for a floating action menu.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FloatingActionButtonMenuState {
    expanded: bool,
}

impl FloatingActionButtonMenuState {
    /// Creates a closed menu.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether the actions are expanded.
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Opens the action list.
    pub fn open(&mut self) {
        self.expanded = true;
    }

    /// Closes the action list.
    pub fn close(&mut self) {
        self.expanded = false;
    }

    /// Toggles the action list.
    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }
}

/// An action in a floating action menu.
#[derive(Clone, Default, PartialEq)]
pub struct FloatingActionButtonMenuItem {
    /// Visible label and accessible action name.
    pub label: String,
    /// Optional leading icon, rendered at 24dp.
    pub icon: Option<Painter>,
    /// Action invoked after closing the menu.
    pub on_click: Option<Callback>,
}

impl FloatingActionButtonMenuItem {
    /// Creates an action with a label, optional icon, and callback.
    pub fn new(label: impl Into<String>, icon: Option<Painter>, on_click: Callback) -> Self {
        Self {
            label: label.into(),
            icon,
            on_click: Some(on_click),
        }
    }
}

/// Theme color set used by the action pills and toggle button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FloatingActionButtonMenuColor {
    /// Use primary container actions and a primary close button.
    #[default]
    Primary,
    /// Use secondary container actions and a secondary close button.
    Secondary,
    /// Use tertiary container actions and a tertiary close button.
    Tertiary,
}

/// Horizontal edge where a floating action menu is anchored.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FloatingActionButtonMenuAlignment {
    /// Align the menu to the left edge.
    Start,
    /// Align the menu to the right edge.
    #[default]
    End,
}

struct ItemMotion {
    width: Animatable,
    opacity: Animatable,
}

impl ItemMotion {
    fn new(visible: bool) -> Self {
        let value = f32::from(visible);
        Self {
            width: Animatable::spring(value, 800.0_f32.sqrt(), 0.6),
            opacity: Animatable::spring(value, 3800.0_f32.sqrt(), 1.0),
        }
    }
}

struct MenuMotion {
    count: Animatable,
    toggle: Animatable,
    items: Vec<ItemMotion>,
    frame_nanos: u64,
}

impl MenuMotion {
    fn new(expanded: bool, count: usize, frame_nanos: u64) -> Self {
        Self {
            count: Animatable::spring(
                if expanded { count as f32 } else { 0.0 },
                800.0_f32.sqrt(),
                1.0,
            ),
            toggle: Animatable::spring(f32::from(expanded), 3800.0_f32.sqrt(), 1.0),
            items: (0..count).map(|_| ItemMotion::new(expanded)).collect(),
            frame_nanos,
        }
    }

    fn set_target(&mut self, expanded: bool, count: usize) {
        self.items.resize_with(count, || ItemMotion::new(false));
        self.count
            .set_target(if expanded { count as f32 } else { 0.0 });
        self.toggle.set_target(f32::from(expanded));
    }

    fn advance(&mut self, frame_nanos: u64) {
        let dt = frame_nanos.saturating_sub(self.frame_nanos) as f32 / 1_000_000_000.0;
        self.frame_nanos = frame_nanos;
        self.count.tick(dt);
        self.toggle.tick(dt);
        let count = self
            .count
            .value()
            .round()
            .clamp(0.0, self.items.len() as f32) as usize;
        let first = self.items.len() - count;
        for (index, item) in self.items.iter_mut().enumerate() {
            let target = f32::from(index >= first);
            item.width.set_target(target);
            item.opacity.set_target(target);
            item.width.tick(dt);
            item.opacity.tick(dt);
        }
    }

    fn is_idle(&self) -> bool {
        self.count.is_idle()
            && self.toggle.is_idle()
            && self
                .items
                .iter()
                .all(|item| item.width.is_idle() && item.opacity.is_idle())
    }

    fn is_visible(&self) -> bool {
        self.items.iter().any(|item| item.opacity.value() > 0.001)
    }
}

#[derive(Clone, Copy, PartialEq)]
struct PanelBounds {
    position: PxPosition,
    size: ComputedData,
}

impl Default for PanelBounds {
    fn default() -> Self {
        Self {
            position: PxPosition::ZERO,
            size: ComputedData::ZERO,
        }
    }
}

#[derive(Clone)]
struct MenuLayout {
    alignment: FloatingActionButtonMenuAlignment,
    bounds: Arc<RwLock<PanelBounds>>,
}

impl PartialEq for MenuLayout {
    fn eq(&self, other: &Self) -> bool {
        self.alignment == other.alignment
    }
}

impl tessera_ui::LayoutModifierNode for MenuLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let children = input.children();
        let main = children[0].measure(input, &input.parent_constraint().without_min())?;
        let parent = input.parent_constraint();
        let size = ComputedData {
            width: parent.width().resolve_max().unwrap_or(main.width),
            height: parent.height().resolve_max().unwrap_or(main.height),
        };
        children[1].measure(input, &Constraint::exact(size.width, size.height))?;
        let margin = Dp(16.0).to_px();
        let panel = children[2].measure(
            input,
            &Constraint::new(
                AxisConstraint::new(Px::ZERO, Some((size.width - margin * 2).max(Px::ZERO))),
                AxisConstraint::new(Px::ZERO, Some((size.height - margin * 2).max(Px::ZERO))),
            ),
        )?;
        let x = match self.alignment {
            FloatingActionButtonMenuAlignment::Start => {
                margin.min((size.width - panel.width).max(Px::ZERO))
            }
            FloatingActionButtonMenuAlignment::End => {
                (size.width - margin - panel.width).max(Px::ZERO)
            }
        };
        let position = PxPosition::new(x, (size.height - margin - panel.height).max(Px::ZERO));
        *self.bounds.write() = PanelBounds {
            position,
            size: panel.size(),
        };
        let mut result = LayoutResult::new(size);
        result.place_child(children[0], PxPosition::ZERO);
        result.place_child(children[1], PxPosition::ZERO);
        result.place_child(children[2], position);
        Ok(result)
    }
}

#[derive(Clone, Copy, PartialEq)]
struct PanelLayout {
    alignment: FloatingActionButtonMenuAlignment,
}

impl tessera_ui::LayoutModifierNode for PanelLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let children = input.children();
        let button = children[1].measure(input, &input.parent_constraint().without_min())?;
        let max_height = input
            .parent_constraint()
            .height()
            .resolve_max()
            .unwrap_or(Px::MAX);
        let list = children[0].measure(
            input,
            &Constraint::new(
                input.parent_constraint().width().without_min(),
                AxisConstraint::new(
                    Px::ZERO,
                    Some((max_height - button.height - Dp(8.0).to_px()).max(Px::ZERO)),
                ),
            ),
        )?;
        let gap = if list.height > Px::ZERO {
            Dp(8.0).to_px()
        } else {
            Px::ZERO
        };
        let width = button.width.max(list.width);
        let height = list.height + gap + button.height;
        let mut result = LayoutResult::new(ComputedData { width, height });
        let x = |child_width| match self.alignment {
            FloatingActionButtonMenuAlignment::Start => Px::ZERO,
            FloatingActionButtonMenuAlignment::End => width - child_width,
        };
        result.place_child(children[0], PxPosition::new(x(list.width), Px::ZERO));
        result.place_child(
            children[1],
            PxPosition::new(x(button.width), list.height + gap),
        );
        Ok(result)
    }
}

#[derive(Clone, Copy, PartialEq)]
struct RevealLayout {
    progress: f32,
    visible: bool,
    gap: bool,
    alignment: FloatingActionButtonMenuAlignment,
}

#[derive(Clone, Copy, PartialEq)]
struct ActionContentLayout {
    has_icon: bool,
}

impl tessera_ui::LayoutModifierNode for ActionContentLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let children = input.children();
        let padding = Dp(24.0).to_px();
        let leading = if self.has_icon {
            Dp(32.0).to_px()
        } else {
            Px::ZERO
        };
        let label = children[usize::from(self.has_icon)];
        let label_size = label.measure(
            input,
            &Constraint::new(
                AxisConstraint::new(
                    Px::ZERO,
                    input
                        .parent_constraint()
                        .width()
                        .resolve_max()
                        .map(|max| (max - padding * 2 - leading).max(Px::ZERO)),
                ),
                AxisConstraint::new(Px::ZERO, Some(Dp(24.0).to_px())),
            ),
        )?;
        let width = input
            .parent_constraint()
            .width()
            .clamp((padding * 2 + leading + label_size.width).max(Dp(56.0).to_px()));
        let height = input.parent_constraint().height().clamp(Dp(56.0).to_px());
        let mut result = LayoutResult::new(ComputedData { width, height });
        if self.has_icon {
            let icon = children[0];
            let side = Dp(24.0).to_px();
            icon.measure(input, &Constraint::exact(side, side))?;
            result.place_child(icon, PxPosition::new(padding, (height - side) / 2));
        }
        result.place_child(
            label,
            PxPosition::new(padding + leading, (height - label_size.height) / 2),
        );
        Ok(result)
    }
}

impl tessera_ui::LayoutModifierNode for RevealLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let child = input.children()[0];
        let size = child.measure(input, &input.parent_constraint().without_min())?;
        let width = input.parent_constraint().width().clamp(Px::new(
            (size.width.to_f32() * self.progress.max(0.0)).round() as i32,
        ));
        let gap = if self.gap { Dp(4.0).to_px() } else { Px::ZERO };
        let height = if self.visible {
            size.height + gap
        } else {
            Px::ZERO
        };
        let mut result = LayoutResult::new(ComputedData { width, height });
        let x = if self.alignment == FloatingActionButtonMenuAlignment::End {
            width - size.width
        } else {
            Px::ZERO
        };
        result.place_child(child, PxPosition::new(x, gap));
        Ok(result)
    }
}

#[derive(Clone, PartialEq)]
struct MenuData {
    items: Vec<FloatingActionButtonMenuItem>,
    requesters: Vec<FocusRequester>,
    trigger: FocusRequester,
    expanded: bool,
    enabled: bool,
    alignment: FloatingActionButtonMenuAlignment,
    colors: [Color; 4],
    open_label: String,
    close_label: String,
    scrim_color: Color,
}

fn dismiss(state: State<FloatingActionButtonMenuState>, trigger: FocusRequester) {
    state.with_mut(FloatingActionButtonMenuState::close);
    trigger.request_focus();
}

fn focus_properties(data: &MenuData, index: usize) -> FocusProperties {
    let enabled: Vec<_> = data
        .items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.on_click.is_some())
        .map(|(i, _)| i)
        .collect();
    let mut order = enabled;
    order.push(data.items.len());
    let position = order
        .iter()
        .position(|i| *i == index)
        .unwrap_or(order.len() - 1);
    let requester = |i: usize| {
        if i == data.items.len() {
            data.trigger
        } else {
            data.requesters[i]
        }
    };
    if !data.expanded {
        return FocusProperties::default().can_focus(index == data.items.len() && data.enabled);
    }
    let next = requester(order[(position + 1) % order.len()]);
    let previous = requester(order[(position + order.len() - 1) % order.len()]);
    FocusProperties::default()
        .next(next)
        .down(next)
        .previous(previous)
        .up(previous)
}

/// # floating_action_button_menu
///
/// Overlay related primary actions above a floating action button.
///
/// ## Usage
///
/// Offer two to six related creation actions while retaining the page beneath
/// them.
///
/// ## Parameters
///
/// - `state` — optional shared visibility state; otherwise stored internally.
/// - `items` — action labels, optional icons, and callbacks; missing callbacks
///   disable actions.
/// - `main_content` — page content beneath the menu, preserved across
///   visibility changes.
/// - `modifier` — size and placement of the entire menu host.
/// - `alignment` — bottom start or bottom end, inset by 16dp.
/// - `color` — primary, secondary, or tertiary theme colors.
/// - `scrim_color` — optional overlay color; transparent by default.
/// - `enabled` — whether the toggle and actions accept input.
/// - `open_label`, `close_label` — localized accessible names for the toggle.
///
/// ## Examples
///
/// ```
/// use tessera_components::floating_action_button_menu::{
///     FloatingActionButtonMenuState, floating_action_button_menu,
/// };
/// use tessera_ui::{remember, tessera};
///
/// let mut visibility = FloatingActionButtonMenuState::new();
/// assert!(!visibility.is_expanded());
/// visibility.open();
/// assert!(visibility.is_expanded());
/// visibility.close();
/// assert!(!visibility.is_expanded());
///
/// #[tessera]
/// fn creation_actions() {
///     let state = remember(FloatingActionButtonMenuState::new);
///     floating_action_button_menu().state(state);
/// }
/// ```
#[tessera]
pub fn floating_action_button_menu(
    state: Option<State<FloatingActionButtonMenuState>>,
    items: Option<Vec<FloatingActionButtonMenuItem>>,
    main_content: Option<RenderSlot>,
    modifier: Option<Modifier>,
    alignment: Option<FloatingActionButtonMenuAlignment>,
    color: Option<FloatingActionButtonMenuColor>,
    scrim_color: Option<Color>,
    enabled: Option<bool>,
    #[prop(into)] open_label: Option<String>,
    #[prop(into)] close_label: Option<String>,
) {
    let state = if let Some(state) = state {
        state
    } else {
        remember(FloatingActionButtonMenuState::new)
    };
    let enabled = enabled.unwrap_or(true);
    if !enabled && state.with(FloatingActionButtonMenuState::is_expanded) {
        state.with_mut(FloatingActionButtonMenuState::close);
    }
    let expanded = state.with(FloatingActionButtonMenuState::is_expanded);
    let items = items.unwrap_or_default();
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let scheme = theme.color_scheme;
    let colors = match color.unwrap_or_default() {
        FloatingActionButtonMenuColor::Primary => [
            scheme.primary_container,
            scheme.on_primary_container,
            scheme.primary,
            scheme.on_primary,
        ],
        FloatingActionButtonMenuColor::Secondary => [
            scheme.secondary_container,
            scheme.on_secondary_container,
            scheme.secondary,
            scheme.on_secondary,
        ],
        FloatingActionButtonMenuColor::Tertiary => [
            scheme.tertiary_container,
            scheme.on_tertiary_container,
            scheme.tertiary,
            scheme.on_tertiary,
        ],
    };
    let alignment = alignment.unwrap_or_default();
    let trigger = remember(FocusRequester::new).get();
    let requesters = remember(Vec::<FocusRequester>::new);
    if requesters.with(|v| v.len()) != items.len() {
        requesters.with_mut(|v| v.resize_with(items.len(), FocusRequester::new));
    }
    let current = MenuData {
        items,
        requesters: requesters.get(),
        trigger,
        expanded,
        enabled,
        alignment,
        colors,
        open_label: open_label.unwrap_or_else(|| "Open actions".into()),
        close_label: close_label.unwrap_or_else(|| "Close actions".into()),
        scrim_color: scrim_color.unwrap_or(Color::TRANSPARENT),
    };
    let data = remember(|| current.clone());
    if data.with(|data| data != &current) {
        data.set(current);
    }
    let now = current_frame_nanos();
    let count = data.with(|data| data.items.len());
    let motion = remember(|| MenuMotion::new(expanded, count, now));
    if motion
        .with(|motion| motion.toggle.target() != f32::from(expanded) || motion.items.len() != count)
    {
        motion.with_mut(|motion| {
            motion.advance(now);
            motion.set_target(expanded, count);
        });
    }
    if !motion.with(MenuMotion::is_idle) {
        receive_frame_nanos(move |now| {
            motion.with_mut(|motion| motion.advance(now));
            if motion.with(MenuMotion::is_idle) {
                FrameNanosControl::Stop
            } else {
                FrameNanosControl::Continue
            }
        });
    }
    let bounds = remember(|| Arc::new(RwLock::new(PanelBounds::default()))).get();
    let pending_focus = remember(|| false);
    let previous = remember(|| false);
    if previous.get() != expanded {
        previous.set(expanded);
        pending_focus.set(true);
    }
    layout()
        .modifier(modifier.unwrap_or_default())
        .policy(MenuLayout {
            alignment,
            bounds: bounds.clone(),
        })
        .child(move || {
            boxed()
                .modifier(Modifier::new().semantics(SemanticsArgs {
                    hidden: data.with(|data| data.expanded),
                    merge_descendants: true,
                    ..Default::default()
                }))
                .children_optional(main_content.clone());
            fab_menu_scrim(data, state, motion).bounds(bounds.clone());
            fab_menu_panel(data, state, motion, pending_focus);
        });
}

#[tessera]
fn fab_menu_scrim(
    data: State<MenuData>,
    state: State<FloatingActionButtonMenuState>,
    motion: State<MenuMotion>,
    bounds: Option<Arc<RwLock<PanelBounds>>>,
) {
    let bounds = bounds.expect("menu bounds must be provided");
    let active = data.with(|data| data.expanded) || motion.with(MenuMotion::is_visible);
    let modifier = if active {
        with_pointer_input(Modifier::new().fill_max_size(), move |mut input| {
            let bounds = *bounds.read();
            let outside = input.cursor_position_rel.is_some_and(|position| {
                !is_position_in_rect(
                    position,
                    bounds.position,
                    bounds.size.width,
                    bounds.size.height,
                )
            });
            let close = outside && input.has_unconsumed_release();
            input.block_all();
            if close {
                dismiss(state, data.with(|data| data.trigger));
            }
        })
    } else {
        Modifier::new().fill_max_size()
    };
    boxed()
        .modifier(modifier.background(if active {
            data.with(|data| data.scrim_color)
        } else {
            Color::TRANSPARENT
        }))
        .children(|| {});
}

#[tessera]
fn fab_menu_panel(
    data: State<MenuData>,
    state: State<FloatingActionButtonMenuState>,
    motion: State<MenuMotion>,
    pending_focus: State<bool>,
) {
    let snapshot = data.get();
    let scope = remember(FocusScopeNode::new).get();
    let scroll = remember(ScrollableController::new);
    let modifier = with_keyboard_input(
        Modifier::new()
            .focus_scope_with(scope)
            .semantics(SemanticsArgs {
                test_tag: cfg!(test).then(|| "fab-menu-panel".into()),
                merge_descendants: true,
                ..Default::default()
            }),
        move |mut input| {
            if !data.with(|data| data.expanded)
                || input.key_modifiers.control_key()
                || input.key_modifiers.alt_key()
                || input.key_modifiers.super_key()
            {
                return;
            }
            if input.keyboard_events.iter().any(|event| {
                event.state == winit::event::ElementState::Pressed
                    && event.logical_key
                        == winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape)
            }) {
                dismiss(state, data.with(|data| data.trigger));
                input.block_keyboard();
            }
        },
    );
    layout()
        .modifier(modifier)
        .policy(PanelLayout {
            alignment: snapshot.alignment,
        })
        .child(move || {
            column()
                .modifier(
                    Modifier::new()
                        .clip_to_bounds()
                        .vertical_scroll(scroll)
                        .semantics(SemanticsArgs {
                            role: Some(Role::Menu),
                            hidden: !data.with(|data| data.expanded),
                            merge_descendants: true,
                            ..Default::default()
                        }),
                )
                .cross_axis_alignment(
                    if data.with(|data| data.alignment) == FloatingActionButtonMenuAlignment::End {
                        CrossAxisAlignment::End
                    } else {
                        CrossAxisAlignment::Start
                    },
                )
                .children(move || {
                    let first_visible = motion.with(|motion| {
                        motion
                            .items
                            .iter()
                            .position(|item| item.opacity.value() > 0.001)
                    });
                    for index in 0..data.with(|data| data.items.len()) {
                        fab_menu_action(
                            index,
                            data,
                            state,
                            motion,
                            first_visible.is_some_and(|first| index > first),
                        );
                    }
                });
            let snapshot = data.get();
            let progress = motion.with(|motion| motion.toggle.value().clamp(0.0, 1.0));
            let corner =
                RoundedCorner::manual(Dp(16.0 + 12.0 * f64::from(progress)), 3.0 - progress);
            let semantics = SemanticsArgs {
                expanded: Some(snapshot.expanded),
                has_popup: Some(HasPopup::Menu),
                ..Default::default()
            };
            surface()
                .modifier(
                    Modifier::new()
                        .size(Dp(56.0), Dp(56.0))
                        .semantics(semantics)
                        .on_globally_positioned(move |_| {
                            if pending_focus.get() {
                                pending_focus.set(false);
                                data.with(|data| data.trigger).request_focus();
                            }
                        }),
                )
                .style(
                    snapshot.colors[0]
                        .lerp(&snapshot.colors[2], progress)
                        .into(),
                )
                .content_color(snapshot.colors[1].lerp(&snapshot.colors[3], progress))
                .shape(Shape::RoundedRectangle {
                    top_left: corner,
                    top_right: corner,
                    bottom_right: corner,
                    bottom_left: corner,
                })
                .elevation(Dp(6.0))
                .tonal_elevation(Dp(0.0))
                .content_alignment(Alignment::Center)
                .enabled(snapshot.enabled)
                .accessibility_label(if snapshot.expanded {
                    snapshot.close_label.clone()
                } else {
                    snapshot.open_label.clone()
                })
                .accessibility_focusable(snapshot.enabled)
                .focus_requester(snapshot.trigger)
                .focus_properties(focus_properties(&snapshot, snapshot.items.len()))
                .on_click(move || {
                    state.with_mut(FloatingActionButtonMenuState::toggle);
                })
                .child(move || {
                    icon()
                        .painter(if data.with(|data| data.expanded) {
                            filled::CLOSE_SVG
                        } else {
                            filled::ADD_SVG
                        })
                        .size(Dp(24.0
                            - f64::from(
                                motion.with(|motion| motion.toggle.value().clamp(0.0, 1.0)),
                            ) * 4.0));
                });
        });
}

#[tessera]
fn fab_menu_action(
    index: usize,
    data: State<MenuData>,
    state: State<FloatingActionButtonMenuState>,
    motion: State<MenuMotion>,
    gap: bool,
) {
    let snapshot = data.get();
    let (progress, opacity) = motion.with(|motion| {
        (
            motion.items[index].width.value(),
            motion.items[index].opacity.value().clamp(0.0, 1.0),
        )
    });
    let visible = opacity > 0.001;
    layout()
        .modifier(
            Modifier::new()
                .clip_to_bounds()
                .alpha(opacity)
                .semantics(SemanticsArgs {
                    hidden: !visible || !snapshot.expanded,
                    test_tag: cfg!(test).then(|| format!("fab-menu-item-{index}")),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .policy(RevealLayout {
            progress,
            visible,
            gap,
            alignment: snapshot.alignment,
        })
        .child(move || {
            let snapshot = data.get();
            let item = snapshot.items[index].clone();
            let visible = motion.with(|motion| motion.items[index].opacity.value() > 0.001);
            let enabled =
                snapshot.enabled && snapshot.expanded && visible && item.on_click.is_some();
            surface()
                .modifier(
                    Modifier::new()
                        .default_min_size(Dp(56.0), Dp(56.0))
                        .height(Dp(56.0)),
                )
                .style(snapshot.colors[0].into())
                .content_color(snapshot.colors[1])
                .shape(Shape::CAPSULE)
                .content_alignment(Alignment::CenterStart)
                .tonal_elevation(Dp(0.0))
                .enabled(enabled)
                .accessibility_role(Role::MenuItem)
                .accessibility_label(item.label.clone())
                .accessibility_focusable(enabled)
                .focus_requester(snapshot.requesters[index])
                .focus_properties(focus_properties(&snapshot, index).can_focus(enabled))
                .on_click(move || {
                    let snapshot = data.get();
                    dismiss(state, snapshot.trigger);
                    if let Some(callback) = &snapshot.items[index].on_click {
                        callback.call();
                    }
                })
                .child(move || {
                    layout()
                        .policy(ActionContentLayout {
                            has_icon: data.with(|data| data.items[index].icon.is_some()),
                        })
                        .child(move || {
                            let item = data.with(|data| data.items[index].clone());
                            let style = use_context::<MaterialTheme>()
                                .unwrap()
                                .get()
                                .typography
                                .title_medium;
                            if let Some(painter) = item.icon.clone() {
                                icon().painter(painter).size(Dp(24.0));
                            }
                            text()
                                .content(item.label.clone())
                                .style(style)
                                .modifier(Modifier::new().clip_to_bounds());
                        });
                });
        });
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use tessera_ui::{
        CursorEventContent, GestureState, PointerChange, PressKeyEventType,
        testing::{
            layout_test, pump_layout_test_session, pump_layout_test_session_with_input,
            start_layout_test_session,
        },
    };

    use crate::theme::material_theme;

    use super::*;

    thread_local! {
        static STATE: Cell<Option<State<FloatingActionButtonMenuState>>> = const { Cell::new(None) };
        static BACKGROUND: Cell<Option<State<usize>>> = const { Cell::new(None) };
        static ACTION: Cell<Option<State<usize>>> = const { Cell::new(None) };
    }

    fn click(
        session: &mut tessera_ui::testing::LayoutTestSession<impl Fn()>,
        frame: &mut u64,
        x: f32,
        y: f32,
    ) -> tessera_ui::testing::LayoutSnapshot {
        for event in [
            CursorEventContent::Pressed(PressKeyEventType::Left),
            CursorEventContent::Released(PressKeyEventType::Left),
        ] {
            *frame += 1;
            pump_layout_test_session_with_input(
                session,
                *frame * 16_666_667,
                vec![PointerChange::new(
                    tessera_ui::time::Instant::now(),
                    1,
                    event,
                    GestureState::TapCandidate,
                )],
                Some(PxPosition::from_f32_xy(x, y)),
            );
        }
        *frame += 1;
        pump_layout_test_session(session, *frame * 16_666_667)
    }

    #[tessera]
    fn fixture(expanded: bool, alignment: FloatingActionButtonMenuAlignment, count: usize) {
        let state = remember(|| FloatingActionButtonMenuState { expanded });
        let action = remember(|| 0usize);
        STATE.set(Some(state));
        ACTION.set(Some(action));
        material_theme().child(move || {
            let items = (0..count)
                .map(|index| FloatingActionButtonMenuItem {
                    label: format!("Action {index}"),
                    icon: Some(filled::EDIT_SVG.into()),
                    on_click: if index == 1 {
                        None
                    } else {
                        Some(Callback::new(move || action.set(index + 1)))
                    },
                })
                .collect();
            floating_action_button_menu()
                .state(state)
                .alignment(alignment)
                .items(items)
                .main_content(move || {
                    let background = remember(|| 7usize);
                    BACKGROUND.set(Some(background));
                    surface()
                        .modifier(Modifier::new().size(Dp(120.0), Dp(48.0)))
                        .on_click(move || background.with_mut(|value| *value += 1))
                        .child(|| {});
                });
        });
    }

    #[test]
    fn action_list_matches_spacing_and_stays_inside_short_viewports() {
        for (width, height) in [(400, 400), (240, 200)] {
            let snapshot = layout_test(|| {
                fixture(true, FloatingActionButtonMenuAlignment::End, 3);
            })
            .viewport_px(width, height)
            .run();
            let panel = snapshot.node("fab-menu-panel").bounds();
            assert_eq!(panel.x + panel.width, Px::new(width as i32 - 16));
            assert_eq!(panel.y + panel.height, Px::new(height as i32 - 16));
            assert!(panel.y >= Px::new(16));
            assert!(snapshot.node("fab-menu-item-0").bounds().width <= Px::new(width as i32 - 32));
            snapshot
                .node("fab-menu-item-0")
                .size(snapshot.node("fab-menu-item-0").bounds().width.raw(), 56);
            assert_eq!(
                snapshot.node("fab-menu-item-1").bounds().height,
                Px::new(60)
            );
            if height == 400 {
                assert_eq!(panel.height, Px::new(240));
                assert_eq!(snapshot.node("fab-menu-item-2").bounds().y, Px::new(260));
            } else {
                assert_eq!(panel.height, Px::new(168));
            }
        }
        let snapshot = layout_test(|| {
            fixture(false, FloatingActionButtonMenuAlignment::Start, 3);
        })
        .viewport_px(400, 400)
        .run();
        snapshot.node("fab-menu-panel").size(56, 56);
        assert_eq!(snapshot.node("fab-menu-panel").bounds().x, Px::new(16));
    }

    #[test]
    fn stagger_reveals_bottom_first_hides_top_first_and_reverses_to_rest() {
        let mut motion = MenuMotion::new(false, 3, 0);
        motion.set_target(true, 3);
        motion.advance(33_333_334);
        assert!(motion.items[2].opacity.value() > 0.0);
        assert_eq!(motion.items[0].opacity.value(), 0.0);
        for frame in 3..120 {
            motion.advance(frame * 16_666_667);
        }
        assert!(motion.is_idle());
        motion.set_target(false, 3);
        motion.advance(2_033_333_334);
        assert!(motion.items[0].opacity.value() < motion.items[2].opacity.value());
        motion.set_target(true, 3);
        for frame in 123..260 {
            motion.advance(frame * 16_666_667);
        }
        assert!(motion.is_idle());
        assert!(motion.items.iter().all(|item| item.opacity.value() == 1.0));
        motion.set_target(false, 3);
        for frame in 260..400 {
            motion.advance(frame * 16_666_667);
        }
        assert!(motion.is_idle());
        assert!(!motion.is_visible());
    }

    #[test]
    fn overlay_blocks_background_preserves_state_and_actions_dismiss() {
        let mut session = start_layout_test_session(
            layout_test(|| {
                fixture(false, FloatingActionButtonMenuAlignment::End, 3);
            })
            .viewport_px(400, 400),
        );
        pump_layout_test_session(&mut session, 0);
        let mut frame = 0u64;
        click(&mut session, &mut frame, 60.0, 24.0);
        assert_eq!(BACKGROUND.get().unwrap().get(), 8);
        click(&mut session, &mut frame, 356.0, 356.0);
        assert!(
            STATE
                .get()
                .unwrap()
                .with(FloatingActionButtonMenuState::is_expanded)
        );
        for _ in 0..100 {
            frame += 1;
            pump_layout_test_session(&mut session, frame * 16_666_667);
        }
        assert_eq!(BACKGROUND.get().unwrap().get(), 8);
        let snapshot = click(&mut session, &mut frame, 350.0, 232.0);
        assert!(
            STATE
                .get()
                .unwrap()
                .with(FloatingActionButtonMenuState::is_expanded),
            "disabled action must not dismiss"
        );
        let bounds = snapshot.node("fab-menu-item-2").bounds();
        click(
            &mut session,
            &mut frame,
            bounds.x.to_f32() + 50.0,
            bounds.y.to_f32() + 28.0,
        );
        assert_eq!(ACTION.get().unwrap().get(), 3);
        assert!(
            !STATE
                .get()
                .unwrap()
                .with(FloatingActionButtonMenuState::is_expanded)
        );
        STATE
            .get()
            .unwrap()
            .with_mut(FloatingActionButtonMenuState::open);
        frame += 1;
        pump_layout_test_session(&mut session, frame * 16_666_667);
        click(&mut session, &mut frame, 60.0, 24.0);
        assert!(
            !STATE
                .get()
                .unwrap()
                .with(FloatingActionButtonMenuState::is_expanded)
        );
        assert_eq!(
            BACKGROUND.get().unwrap().get(),
            8,
            "dismissal must not click through"
        );
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn rendered_menu_restores_pixels_and_exposes_disclosure_state() {
        use tessera_ui::{HeadlessConfig, HeadlessRenderer};

        fn disclosure_states(renderer: &HeadlessRenderer) -> Vec<bool> {
            let tree = renderer
                .accessibility_snapshot()
                .expect("accessibility tree");
            let mut states = Vec::new();
            for (_, node) in &tree.nodes {
                if let Some(expanded) = node.is_expanded() {
                    assert_eq!(node.has_popup(), Some(HasPopup::Menu));
                    states.push(expanded);
                }
            }
            assert_eq!(
                tree.nodes
                    .iter()
                    .filter(|(_, node)| node.role() == Role::MenuItem)
                    .count(),
                3,
                "menu wrappers must preserve action semantics"
            );
            let menu = tree
                .nodes
                .iter()
                .find(|(_, node)| node.role() == Role::Menu)
                .expect("menu semantic group");
            assert_eq!(menu.1.is_hidden(), !states[0]);
            states
        }

        let mut renderer = HeadlessRenderer::new(
            Box::new(|| {
                fixture(true, FloatingActionButtonMenuAlignment::End, 3);
            }),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 400,
                height: 400,
                ..Default::default()
            },
        )
        .expect("GPU renderer");
        renderer.render_frames(3);
        let expanded = renderer.read_pixels().unwrap();
        let corner = (335 * 400 + 335) * 4;
        let interior = (356 * 400 + 335) * 4;
        let contrast: u32 = (0..3)
            .map(|channel| expanded[corner + channel].abs_diff(expanded[interior + channel]) as u32)
            .sum();
        assert!(
            contrast > 32,
            "the expanded close button must have circular corners instead of a squircle"
        );
        assert_eq!(disclosure_states(&renderer), vec![true]);
        STATE
            .get()
            .unwrap()
            .with_mut(FloatingActionButtonMenuState::close);
        renderer.render_frames(120);
        assert_ne!(expanded, renderer.read_pixels().unwrap());
        assert_eq!(disclosure_states(&renderer), vec![false]);
        STATE
            .get()
            .unwrap()
            .with_mut(FloatingActionButtonMenuState::open);
        renderer.render_frames(120);
        let reopened = renderer.read_pixels().unwrap();
        assert!(
            expanded
                .iter()
                .zip(&reopened)
                .all(|(before, after)| before.abs_diff(*after) <= 1),
            "menu labels and shapes must recover within one channel step after a complete animation cycle"
        );
    }
}
