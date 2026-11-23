//! Material Design 3 menus for contextual action lists.
//!
//! ## Usage
//!
//! Present anchored overflow or context actions as surfaced menus.
use std::time::Duration;

use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    AxisConstraint, Callback, Color, CursorEventContent, Dp, DrawModifierContent,
    DrawModifierContext, DrawModifierNode, FloatRect, FocusDirection, FocusRequester,
    FocusScopeNode, FocusTraversalPolicy, KeyboardInput, KeyboardInputModifierNode,
    LayoutCoordinates, MOUSE_POINTER_ID, Modifier, PointerInput, PointerInputModifierNode, Px,
    PxPosition, PxSize, RenderSlot, RenderSlotWith, State,
    accesskit::{HasPopup, Role, Toggled},
    focus, launch, launch_once,
    layout::{Stack, group, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    popup::{PopupPositionInput, PopupPositionPolicy, WindowOrigin, popup},
    provide_context, remember,
    render::{ClipRegion, ClipShape},
    tessera, use_context, winit, with_frame_time,
};

use crate::{
    alignment::CrossAxisAlignment,
    animation::{Delay, animate_to},
    boxed::boxed,
    column::column,
    md3::{
        checkmark::checkmark,
        icon::icon,
        material_icons::filled,
        modifier::ModifierExt as _,
        scrollable::ScrollableController,
        surface::{SurfaceStyle, surface},
        text::text,
        theme::{MaterialAlpha, MaterialTheme},
    },
    modifier::with_keyboard_input,
    painter::Painter,
    row::row,
    shape_def::Shape,
    spacer::spacer,
};

use cascade::{MenuBranch, MenuBranchLease, MenuCascade};
use motion::{MenuShapeHover, MenuShapeMotion};

pub use appearance::{MenuColorStyle, MenuDefaults, MenuPosition, menu_group};

mod appearance;
mod cascade;
mod motion;
mod typeahead;

const MENU_MIN_WIDTH: Dp = Dp(112.0);
const MENU_MAX_WIDTH: Dp = Dp(280.0);
const MENU_MAX_HEIGHT: Dp = Dp(320.0);
const MENU_VERTICAL_GAP: Dp = Dp(4.0);
const MENU_HORIZONTAL_PADDING: Dp = Dp(12.0);
const MENU_LEADING_SIZE: Dp = Dp(20.0);
const MENU_ITEM_HEIGHT: Dp = MenuDefaults::ITEM_HEIGHT;
const MENU_TRAILING_SPACING: Dp = Dp(12.0);
const MENU_HOVER_DELAY: Duration = Duration::from_millis(150);

fn default_menu_width() -> AxisConstraint {
    AxisConstraint::new(Px::from(MENU_MIN_WIDTH), Some(Px::from(MENU_MAX_WIDTH)))
}

fn default_menu_modifier() -> Modifier {
    Modifier::new().constrain(Some(default_menu_width()), None)
}

fn default_max_height() -> Option<Px> {
    Some(Px::from(MENU_MAX_HEIGHT))
}

fn default_menu_shape() -> Shape {
    MenuDefaults::group_shape(MenuPosition::Standalone)
}

fn default_menu_color() -> Color {
    Color::new(1.0, 1.0, 1.0, 1.0)
}

fn default_scrim_color() -> Color {
    Color::new(0.0, 0.0, 0.0, 0.0)
}

/// Describes the anchor rectangle used to position a menu.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuAnchor {
    /// Top-left corner of the anchor rectangle, relative to the anchor content.
    pub origin: PxPosition,
    /// Size of the anchor rectangle.
    pub size: PxSize,
}

impl MenuAnchor {
    /// Creates a new anchor rectangle from origin and size.
    pub fn new(origin: PxPosition, size: PxSize) -> Self {
        Self { origin, size }
    }

    /// Creates an anchor positioned at `origin` with zero size.
    pub fn at(origin: PxPosition) -> Self {
        Self::new(origin, PxSize::new(Px::ZERO, Px::ZERO))
    }

    /// Creates an anchor from dp values for origin and size.
    pub fn from_dp(origin: (Dp, Dp), size: (Dp, Dp)) -> Self {
        Self::new(
            PxPosition::new(origin.0.into(), origin.1.into()),
            PxSize::new(size.0.into(), size.1.into()),
        )
    }
}

impl Default for MenuAnchor {
    fn default() -> Self {
        Self::at(PxPosition::new(Px::ZERO, Px::ZERO))
    }
}

/// Shared state for controlling menu visibility and anchor placement.
#[derive(Default, Clone, PartialEq)]
pub struct MenuController {
    is_open: bool,
    anchor: Option<MenuAnchor>,
    exit_direction: Option<FocusDirection>,
}

impl MenuController {
    /// Creates a new closed menu state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens the menu.
    ///
    /// If an anchor was previously set via [`open_at`](Self::open_at), it is
    /// reused. Otherwise, the menu defaults to anchoring to its trigger
    /// content.
    pub fn open(&mut self) {
        self.is_open = true;
        self.exit_direction = None;
    }

    /// Opens the menu at the provided anchor rectangle.
    ///
    /// An explicit `anchor` prop on the menu takes precedence over this
    /// saved rectangle.
    pub fn open_at(&mut self, anchor: MenuAnchor) {
        self.anchor = Some(anchor);
        self.is_open = true;
        self.exit_direction = None;
    }

    /// Closes the menu.
    pub fn close(&mut self) {
        self.is_open = false;
        self.exit_direction = None;
    }

    /// Toggles the open state, keeping the current anchor.
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
        self.exit_direction = None;
    }

    /// Returns whether the menu is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }
}

/// Controls how the menu is aligned relative to its anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuPlacement {
    /// Align to the anchor's start edge and expand downward.
    #[default]
    BelowStart,
    /// Align to the anchor's end edge and expand downward.
    BelowEnd,
    /// Align to the anchor's start edge and expand upward.
    AboveStart,
    /// Align to the anchor's end edge and expand upward.
    AboveEnd,
    /// Align to the anchor's end edge and expand rightward.
    RightStart,
    /// Align to the anchor's start edge and expand leftward.
    LeftStart,
}

/// The owner of actions declared in one open menu.
#[derive(Clone, Copy, PartialEq)]
pub struct MenuScope {
    host: MenuCascade,
    controller: State<MenuController>,
    typeahead: typeahead::MenuTypeahead,
}

#[derive(Clone, PartialEq)]
struct MenuConfig {
    anchor: Option<MenuAnchor>,
    window_anchor: Option<State<LayoutCoordinates>>,
    grouped: bool,
    color_style: MenuColorStyle,
    placement: MenuPlacement,
    offset: [Dp; 2],
    modifier: Modifier,
    max_height: Option<Px>,
    shape: Shape,
    animate_shape: bool,
    elevation: Dp,
    container_color: Color,
    close_on_background: bool,
    close_on_escape: bool,
    close_on_focus_loss: bool,
    on_dismiss: Option<Callback>,
}

impl Default for MenuConfig {
    fn default() -> Self {
        Self {
            anchor: None,
            window_anchor: None,
            grouped: false,
            color_style: MenuColorStyle::Standard,
            placement: MenuPlacement::default(),
            offset: [Dp(0.0), MENU_VERTICAL_GAP],
            modifier: default_menu_modifier(),
            max_height: default_max_height(),
            shape: default_menu_shape(),
            animate_shape: true,
            elevation: Dp(3.0),
            container_color: default_menu_color(),
            close_on_background: true,
            close_on_escape: true,
            close_on_focus_loss: false,
            on_dismiss: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct MenuPopupPosition {
    placement: MenuPlacement,
    offset: [Dp; 2],
    anchor: Option<MenuAnchor>,
    window_anchor: Option<LayoutCoordinates>,
}

impl PopupPositionPolicy for MenuPopupPosition {
    fn position(&self, input: &PopupPositionInput) -> PxPosition {
        let anchor = self
            .window_anchor
            .map(|anchor| anchor.window_bounds())
            .or_else(|| {
                self.anchor.map(|anchor| {
                    input
                        .anchor
                        .local_to_window
                        .map_rect(FloatRect::from_position_size(anchor.origin, anchor.size))
                })
            })
            .unwrap_or_else(|| input.anchor.window_bounds());
        resolve_menu_position_in_bounds(
            anchor,
            self.placement,
            input.popup_size,
            input.bounds,
            self.offset,
        )
    }
}

struct MenuScrollClip {
    controller: State<ScrollableController>,
}

impl DrawModifierNode for MenuScrollClip {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        let mut metadata = ctx.render_input.metadata_mut();
        let size = metadata.computed_data().expect("measured menu viewport");
        let overflow = self
            .controller
            .with(|scroll| scroll.child_size().height > size.height);
        // Group shadows need paint overflow; active scrolling still crops
        // content at the viewport's top and bottom edges.
        let margin = Dp(16.0).to_pixels_f32();
        let vertical_margin = if overflow { 0.0 } else { margin };
        metadata.set_clip_region(ClipRegion {
            position: PxPosition::from_f32_xy(-margin, -vertical_margin),
            size: [
                size.width.to_f32() + 2.0 * margin,
                size.height.to_f32() + 2.0 * vertical_margin,
            ],
            shape: ClipShape::RoundedRectangle {
                radii: [0.0; 4],
                exponents: [2.0; 4],
            },
        });
        content.draw(ctx.render_input);
    }
}

fn resolve_menu_position_in_bounds(
    anchor: tessera_ui::FloatRect,
    placement: MenuPlacement,
    menu_size: PxSize,
    bounds: tessera_ui::FloatRect,
    offset: [Dp; 2],
) -> PxPosition {
    let anchor_x = anchor.x;
    let anchor_y = anchor.y;
    let anchor_end_x = anchor_x + anchor.width;
    let anchor_end_y = anchor_y + anchor.height;

    let width = menu_size.width.to_f32();
    let height = menu_size.height.to_f32();
    let dx = offset[0].to_pixels_f32();
    let dy = offset[1].to_pixels_f32();
    let start_x = anchor_x + dx;
    let end_x = anchor_end_x - width + dx;
    let above = anchor_y - height - dy;
    let below = anchor_end_y + dy;
    let right = anchor_end_x + dx;
    let left = anchor_x - width - dx;
    let start_y = anchor_y + dy;
    let end_y = anchor_end_y - height + dy;
    let (x, alternate_x, y, alternate_y) = match placement {
        MenuPlacement::BelowStart => (start_x, end_x, below, above),
        MenuPlacement::BelowEnd => (end_x, start_x, below, above),
        MenuPlacement::AboveStart => (start_x, end_x, above, below),
        MenuPlacement::AboveEnd => (end_x, start_x, above, below),
        MenuPlacement::RightStart => (right, left, start_y, end_y),
        MenuPlacement::LeftStart => (left, right, start_y, end_y),
    };
    PxPosition::from_f32_xy(
        fit_menu_axis(x, alternate_x, width, bounds.x, bounds.width),
        fit_menu_axis(y, alternate_y, height, bounds.y, bounds.height),
    )
}

fn fit_menu_axis(preferred: f32, alternate: f32, extent: f32, origin: f32, available: f32) -> f32 {
    let max = (origin + available - extent).max(origin);
    if preferred >= origin && preferred <= max {
        preferred
    } else if alternate >= origin && alternate <= max {
        alternate
    } else {
        preferred.clamp(origin, max)
    }
}

fn apply_close_action(controller: State<MenuController>, on_dismiss: Option<Callback>) {
    if let Some(callback) = on_dismiss {
        callback.call();
    }
    controller.with_mut(MenuController::close);
}

fn close_for_traversal(
    controller: State<MenuController>,
    on_dismiss: Option<Callback>,
    reverse: bool,
) {
    if let Some(callback) = on_dismiss {
        callback.call();
    }
    controller.with_mut(|controller| {
        controller.close();
        controller.exit_direction = Some(if reverse {
            FocusDirection::Previous
        } else {
            FocusDirection::Next
        });
    });
    focus::defer_keyboard_until_layout();
}

#[derive(Clone, Copy)]
struct MenuKeyboardBoundary {
    controller: State<MenuController>,
    rendered_open: bool,
}

impl MenuKeyboardBoundary {
    fn check(&self) {
        if self.controller.with(MenuController::is_open) != self.rendered_open {
            focus::defer_keyboard_until_layout();
        }
    }
}

impl KeyboardInputModifierNode for MenuKeyboardBoundary {
    fn on_keyboard_input(&self, _: KeyboardInput<'_>) {
        self.check();
    }
}

impl PointerInputModifierNode for MenuKeyboardBoundary {
    fn on_pointer_input(&self, _: PointerInput<'_>) {
        self.check();
    }
}

/// # menu_anchor
///
/// Shows a Material Design 3 menu above its anchor content.
///
/// ## Usage
///
/// Wrap a trigger and its adjacent content with contextual or overflow actions.
///
/// ## Parameters
///
/// - `anchor` — current anchor relative to the trigger; takes precedence over
///   the controller's saved anchor and can follow layout changes
/// - `window_anchor` — observed trigger coordinates, including ancestor transforms;
///   takes precedence over `anchor`
/// - `placement` — how the menu aligns relative to its anchor
/// - `grouped` — render separate `menu_group` surfaces on a transparent panel
/// - `color_style` — standard or vibrant colors, inherited by groups and items
/// - `offset` — x/y distance away from the anchor on the expansion axis, and a
///   shift on the other axis
/// - `modifier` — extra layout modifiers for the menu container
/// - `max_height` — optional maximum menu height before scrolling
/// - `shape` — optional fixed panel shape; default corners respond to hover and
///   keyboard focus
/// - `elevation` — menu elevation level
/// - `container_color` — background color of the menu panel
/// - `scrim_color` — background color behind the menu
/// - `close_on_background` — whether presses outside the menu chain dismiss it
/// - `close_on_escape` — whether Escape dismisses the menu
/// - `close_on_focus_loss` — whether focus leaving the menu dismisses it
/// - `on_dismiss` — optional callback before dismissal
/// - `controller` — optional external controller for open state and anchor
/// - `focus_restorer_fallback` — optional fallback focus target on dismiss
/// - `anchor_content` — trigger content that anchors the menu
/// - `menu_content` — optional menu content slot
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::md3::{
///     menus::{MenuAnchor, MenuController, MenuPlacement, menu_item, menu_anchor},
///     text::text,
/// };
/// use tessera_ui::{Dp, remember};
/// # use tessera_components::md3::theme::{MaterialTheme, material_theme};
///
/// let controller = remember(MenuController::new);
/// controller.with_mut(MenuController::open);
/// assert!(controller.with(MenuController::is_open));
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(move || {
/// menu_anchor()
///     .anchor(MenuAnchor::from_dp(
///         (Dp(16.0), Dp(16.0)),
///         (Dp(80.0), Dp(40.0)),
///     ))
///     .placement(MenuPlacement::BelowStart)
///     .controller(controller)
///     .anchor_content(|| {
///         text("Actions");
///     })
///     .menu_content(|menu| {
///         menu_item().menu(menu).label("Edit").on_click(|| {});
///     });
/// # });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn menu_anchor(
    anchor: Option<MenuAnchor>,
    window_anchor: Option<State<tessera_ui::LayoutCoordinates>>,
    grouped: Option<bool>,
    color_style: Option<MenuColorStyle>,
    placement: Option<MenuPlacement>,
    offset: Option<[Dp; 2]>,
    modifier: Option<Modifier>,
    max_height: Option<Px>,
    shape: Option<Shape>,
    elevation: Option<Dp>,
    container_color: Option<Color>,
    scrim_color: Option<Color>,
    close_on_background: Option<bool>,
    close_on_escape: Option<bool>,
    close_on_focus_loss: Option<bool>,
    on_dismiss: Option<Callback>,
    controller: Option<State<MenuController>>,
    focus_restorer_fallback: Option<FocusRequester>,
    anchor_content: Option<RenderSlot>,
    menu_content: Option<RenderSlotWith<MenuScope>>,
) {
    let grouped = grouped.unwrap_or(false);
    let inherited_style = use_context::<MenuColorStyle>().map(|style| style.get());
    let color_style = color_style.or(inherited_style).unwrap_or_default();
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let config = MenuConfig {
        anchor,
        window_anchor,
        grouped,
        color_style,
        placement: placement.unwrap_or_default(),
        offset: offset.unwrap_or([Dp(0.0), MENU_VERTICAL_GAP]),
        modifier: modifier.unwrap_or_else(default_menu_modifier),
        max_height: max_height.or_else(default_max_height),
        animate_shape: shape.is_none(),
        shape: shape.unwrap_or_else(default_menu_shape),
        elevation: elevation.unwrap_or(Dp(3.0)),
        container_color: container_color.unwrap_or_else(|| {
            if grouped {
                Color::TRANSPARENT
            } else {
                color_style.container(&scheme)
            }
        }),
        close_on_background: close_on_background.unwrap_or(true),
        close_on_escape: close_on_escape.unwrap_or(true),
        close_on_focus_loss: close_on_focus_loss.unwrap_or(false),
        on_dismiss,
    };
    let controller = if let Some(controller) = controller {
        controller
    } else {
        remember(MenuController::new)
    };
    let host = MenuCascade {
        root: controller,
        entries: remember(Vec::new),
        root_scope: remember(FocusScopeNode::new).get(),
        restore_parent: remember(|| None),
        bounds: remember(|| None),
        on_dismiss,
        close_on_escape: config.close_on_escape,
    };
    let dismiss_requested = remember(|| false);
    if dismiss_requested.get() {
        dismiss_requested.set(false);
        if controller.with(MenuController::is_open) {
            apply_close_action(controller, on_dismiss);
        }
    }
    let is_open = controller.with(MenuController::is_open);
    let was_open = remember(|| false);
    let anchor_scope = remember(FocusScopeNode::new).get();
    let saved_focus = remember(|| None::<focus::FocusRestorationTarget>);
    let pending_restore = remember(|| false);
    if was_open.get() != is_open {
        was_open.set(is_open);
        focus::defer_keyboard_until_layout();
        if is_open {
            saved_focus.set(focus::save_focus());
        } else {
            pending_restore.set(host.root_scope.has_focus());
            host.restore_parent.set_if_changed(None);
        }
    }

    let boundary = MenuKeyboardBoundary {
        controller,
        rendered_open: is_open,
    };
    let mut modifier = Modifier::new()
        .push_keyboard_input(boundary)
        .push_pointer_input(boundary);
    if pending_restore.get() || host.restore_parent.get().is_some() {
        modifier = modifier.on_globally_positioned(move |_| {
            if controller.with(MenuController::is_open) {
                if let Some(scope) = host.restore_parent.with_mut(Option::take) {
                    scope.request_focus();
                }
            } else if pending_restore.with_mut(std::mem::take) {
                if let Some(fallback) = focus_restorer_fallback {
                    fallback.request_focus();
                } else if !saved_focus.get().is_some_and(|target| target.restore()) {
                    anchor_scope.restore_focus();
                }
                if let Some(direction) = controller.with_mut(|menu| menu.exit_direction.take()) {
                    focus::request_move_focus(direction);
                }
            }
        });
    }
    let anchor_content = anchor_content.unwrap_or_else(RenderSlot::empty);
    let menu_content = if let Some(menu_content) = menu_content {
        menu_content
    } else {
        RenderSlotWith::new(|_| {})
    };
    boxed().modifier(modifier).children(move || {
        menu_trigger(anchor_scope).content_shared(anchor_content);
        if is_open {
            menu_backdrop().color_optional(scrim_color);
            menu_popup_layer(
                config.clone(),
                host,
                controller,
                host.root_scope,
                host.bounds,
                menu_content,
                dismiss_requested,
                false,
            );
        }
    });
}

#[tessera]
fn menu_trigger(scope: FocusScopeNode, content: Option<RenderSlot>) {
    group()
        .modifier(Modifier::new().focus_scope(Some(scope)))
        .child(content.unwrap_or_else(RenderSlot::empty));
}

#[tessera]
fn menu_backdrop(color: Option<Color>) {
    popup(WindowOrigin).child(move || {
        surface()
            .style(SurfaceStyle::Filled {
                color: color.unwrap_or_else(default_scrim_color),
            })
            .modifier(Modifier::new().fill_max_size())
            .block_input(true)
            .child(|| {});
    });
}

#[tessera]
fn menu_popup_layer(
    config: MenuConfig,
    host: MenuCascade,
    controller: State<MenuController>,
    focus_scope: FocusScopeNode,
    bounds: State<Option<FloatRect>>,
    content: RenderSlotWith<MenuScope>,
    dismiss_requested: State<bool>,
    is_submenu: bool,
) {
    let position = MenuPopupPosition {
        placement: config.placement,
        offset: config.offset,
        anchor: config
            .anchor
            .or_else(|| controller.with(|menu| menu.anchor)),
        window_anchor: config.window_anchor.map(|anchor| anchor.get()),
    };
    let initial_focus = remember(|| true);
    let mut modifier = Modifier::new()
        .on_layout_coordinates(move |coordinates: LayoutCoordinates| {
            bounds.set_if_changed(Some(coordinates.window_bounds()));
        })
        .push_pointer_preview_input(cascade::MenuHierarchyPointer(host));
    if initial_focus.get() {
        modifier = modifier.on_globally_positioned(move |_| {
            if initial_focus.with_mut(std::mem::take) && controller.with(MenuController::is_open) {
                focus_scope.request_focus();
            }
        });
    }
    popup(position)
        .modifier(modifier)
        .dismiss_on_outside_press(!is_submenu && config.close_on_background)
        .dismiss_on_escape(config.close_on_escape)
        // Dismissal can run while the layout-only runtime is borrowed.
        .on_dismiss_request(move || {
            dismiss_requested.set_if_changed(true);
        })
        .child(move || {
            menu_panel(
                config.clone(),
                host,
                controller,
                focus_scope,
                content,
                dismiss_requested,
                is_submenu,
            );
        });
}

fn closes_submenu(
    is_submenu: bool,
    placement: MenuPlacement,
    key: winit::keyboard::NamedKey,
) -> bool {
    is_submenu
        && matches!(
            (placement, key),
            (
                MenuPlacement::RightStart,
                winit::keyboard::NamedKey::ArrowLeft
            ) | (
                MenuPlacement::LeftStart,
                winit::keyboard::NamedKey::ArrowRight
            )
        )
}

#[tessera]
fn menu_panel(
    provider: MenuConfig,
    host: MenuCascade,
    controller: State<MenuController>,
    focus_scope: FocusScopeNode,
    menu_content: RenderSlotWith<MenuScope>,
    dismiss_requested: State<bool>,
    is_submenu: bool,
) {
    let on_dismiss = provider.on_dismiss;
    let placement = provider.placement;
    let focus_seen = remember(|| false);
    let typeahead = remember(typeahead::Registry::default);
    let scroll = remember(ScrollableController::new);
    let color_style = provider.color_style;
    let grouped = provider.grouped;
    let motion = remember(|| MenuShapeMotion::new(MenuPosition::Standalone));
    let focused = focus_scope.has_focus();
    if provider.close_on_focus_loss && !is_submenu {
        let was_focused = focus_seen.get();
        if was_focused != focused {
            focus_seen.set(focused);
        }
        if was_focused && !focused && controller.with(MenuController::is_open) {
            apply_close_action(controller, on_dismiss);
        }
    }
    if !grouped && provider.animate_shape {
        let focused = focused && !host.descendant_has_focus(controller);
        let hovered = motion.with(MenuShapeMotion::hovered);
        launch(
            (focused, hovered),
            animate_to(motion, (MenuPosition::Standalone, focused)),
        );
    }
    let hover = MenuShapeHover {
        motion,
        host: Some(host),
        controller: Some(controller),
    };

    let mut modifier = with_keyboard_input(
        Modifier::new()
            .layout_coordinate_space()
            .focus_scope(Some(focus_scope))
            .focus_traversal_policy(FocusTraversalPolicy::vertical().wrap(true)),
        move |mut input| {
            if input.key_modifiers.control_key()
                || input.key_modifiers.alt_key()
                || input.key_modifiers.super_key()
            {
                return;
            }

            let mut handled = false;
            for event in input.keyboard_events.iter() {
                if event.state != winit::event::ElementState::Pressed {
                    continue;
                }

                handled = match &event.logical_key {
                    winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab) => {
                        close_for_traversal(
                            host.root,
                            host.on_dismiss,
                            input.key_modifiers.shift_key(),
                        );
                        true
                    }
                    winit::keyboard::Key::Named(key)
                        if closes_submenu(is_submenu, placement, *key) =>
                    {
                        dismiss_requested.set_if_changed(true);
                        focus::defer_keyboard_until_layout();
                        true
                    }
                    winit::keyboard::Key::Character(text) => {
                        typeahead::type_text(typeahead, text.as_str(), input.timestamp)
                    }
                    _ => false,
                };

                if handled {
                    break;
                }
            }

            if handled {
                input.block_keyboard();
            }
        },
    );
    if !grouped && provider.animate_shape {
        modifier = modifier.push_pointer_preview_input(hover);
    }

    layout(Stack).modifier(modifier).child(move || {
        let menu_content = menu_content;
        surface()
            .style(SurfaceStyle::Filled {
                color: provider.container_color,
            })
            .shape(if grouped {
                Shape::RECTANGLE
            } else if provider.animate_shape {
                motion.with(MenuShapeMotion::shape)
            } else {
                provider.shape
            })
            .modifier(provider.modifier.clone().constrain(
                None,
                Some(AxisConstraint::new(Px::ZERO, provider.max_height)),
            ))
            .accessibility_role(Role::Menu)
            .block_input(true)
            .elevation(if grouped {
                Dp::ZERO
            } else {
                provider.elevation
            })
            .child(move || {
                let menu_content = menu_content;
                provide_context(color_style, move || {
                    let modifier = Modifier::new().fill_max_width();
                    let modifier = if grouped {
                        modifier.push_draw(MenuScrollClip { controller: scroll })
                    } else {
                        modifier.clip_shape(if provider.animate_shape {
                            motion.with(MenuShapeMotion::shape)
                        } else {
                            provider.shape
                        })
                    };
                    column()
                        .modifier(
                            modifier.vertical_scroll(scroll).padding_symmetric(
                                Dp::ZERO,
                                if grouped { Dp::ZERO } else { Dp(2.0) },
                            ),
                        )
                        .cross_axis_alignment(CrossAxisAlignment::Start)
                        .children(move || {
                            menu_content.render(MenuScope {
                                host,
                                controller,
                                typeahead: typeahead::MenuTypeahead(typeahead),
                            });
                        });
                });
            });
    });
}

/// Selection behavior announced for a menu item.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuItemSelectionMode {
    /// One checked item in a mutually exclusive set.
    #[default]
    Radio,
    /// An independently checked item in a set of choices.
    Checkbox,
}

#[derive(Clone, PartialEq)]
struct MenuItemConfig {
    pub menu: Option<MenuScope>,
    pub label: String,
    pub supporting_text: Option<String>,
    pub trailing_text: Option<String>,
    pub leading_icon: Option<Painter>,
    pub trailing_icon: Option<Painter>,
    pub submenu_content: Option<RenderSlotWith<MenuScope>>,
    pub submenu_placement: MenuPlacement,
    pub selected: bool,
    pub selection: Option<Toggled>,
    pub selection_mode: Option<MenuItemSelectionMode>,
    pub shape: Shape,
    pub container_color: Color,
    pub enabled: bool,
    pub close_on_click: bool,
    pub height: Dp,
    pub label_color: Color,
    pub supporting_color: Color,
    pub disabled_color: Color,
    pub on_click: Option<Callback>,
}

impl MenuItemConfig {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            menu: None,
            label: label.into(),
            supporting_text: None,
            trailing_text: None,
            leading_icon: None,
            trailing_icon: None,
            submenu_content: None,
            submenu_placement: MenuPlacement::RightStart,
            selected: false,
            selection: None,
            selection_mode: None,
            shape: MenuDefaults::item_shape(MenuPosition::Standalone, false),
            container_color: Color::TRANSPARENT,
            enabled: true,
            close_on_click: true,
            height: MENU_ITEM_HEIGHT,
            label_color: Color::new(0.1, 0.1, 0.1, 1.0),
            supporting_color: Color::new(0.4, 0.4, 0.4, 1.0),
            disabled_color: Color::new(0.1, 0.1, 0.1, 1.0)
                .with_alpha(MaterialAlpha::DISABLED_CONTENT),
            on_click: None,
        }
    }
}

impl Default for MenuItemConfig {
    fn default() -> Self {
        Self::new("")
    }
}

/// # menu_item
///
/// Material Design 3 menu item for use inside [`menu_anchor`].
///
/// ## Usage
///
/// Build contextual or overflow menu actions inside
/// `menu_anchor().menu_content(...)`.
///
/// ## Parameters
///
/// - `menu` — action owner received from the enclosing menu content slot.
/// - `label` — primary text shown for the menu action
/// - `supporting_text` — optional supporting text below the label
/// - `trailing_text` — optional trailing text such as a shortcut hint
/// - `leading_icon` — optional icon shown on the leading side
/// - `trailing_icon` — optional trailing icon before the submenu disclosure
///   arrow
/// - `submenu_content` — optional submenu opened by activation or mouse dwell;
///   branch activation opens this submenu instead of calling `on_click`.
/// - `submenu_placement` — optional submenu placement override
/// - `selected` — caller-owned visual and checked state; activation never
///   changes it automatically.
/// - `selection_mode` — optional radio or checkbox semantics for a leaf or a
///   submenu branch; omit it for an ordinary action.
/// - `position` — item position within its group, controlling corner shapes
/// - `shape` — optional fixed item shape override
/// - `container_color` — optional item background override
/// - `color_style` — optional standard or vibrant color override
/// - `enabled` — whether the item can be activated
/// - `close_on_click` — whether selecting a leaf closes the menu chain; submenu
///   expansion does not close the chain.
/// - `height` — optional item height override
/// - `label_color` — optional label text color override
/// - `supporting_color` — optional supporting or trailing text color override
/// - `disabled_color` — optional disabled content color override
/// - `on_click` — optional leaf activation callback; `submenu_content` takes
///   precedence when both are provided.
///
/// ## Examples
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::md3::{
///     menus::{MenuController, menu_item, menu_anchor},
///     text::text,
/// };
/// # use tessera_components::md3::theme::{MaterialTheme, material_theme};
/// use tessera_ui::remember;
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// menu_anchor()
///     .controller(remember(|| {
///         let mut controller = MenuController::new();
///         controller.open();
///         controller
///     }))
///     .menu_content(|menu| {
///         menu_item().menu(menu).label("Open");
///     })
///     .anchor_content(|| {
///         text("Main content");
///     });
/// # });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn menu_item(
    menu: Option<MenuScope>,
    #[prop(into)] label: Option<String>,
    #[prop(into)] supporting_text: Option<String>,
    #[prop(into)] trailing_text: Option<String>,
    #[prop(into)] leading_icon: Option<Painter>,
    #[prop(into)] trailing_icon: Option<Painter>,
    submenu_content: Option<RenderSlotWith<MenuScope>>,
    submenu_placement: Option<MenuPlacement>,
    selected: Option<bool>,
    selection_mode: Option<MenuItemSelectionMode>,
    position: Option<MenuPosition>,
    shape: Option<Shape>,
    container_color: Option<Color>,
    color_style: Option<MenuColorStyle>,
    enabled: Option<bool>,
    close_on_click: Option<bool>,
    height: Option<Dp>,
    label_color: Option<Color>,
    supporting_color: Option<Color>,
    disabled_color: Option<Color>,
    on_click: Option<Callback>,
) {
    let label = label.unwrap_or_default();
    let selected = selected.unwrap_or(false);
    let selection = selection_mode.map(|_| {
        if selected {
            Toggled::True
        } else {
            Toggled::False
        }
    });
    let position = position.unwrap_or_default();
    let enabled = enabled.unwrap_or(true) && (on_click.is_some() || submenu_content.is_some());
    let inherited_style = use_context::<MenuColorStyle>().map(|style| style.get());
    let color_style = color_style.or(inherited_style).unwrap_or_default();
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;

    let mut args = MenuItemConfig {
        menu,
        label,
        supporting_text,
        trailing_text,
        leading_icon,
        trailing_icon,
        submenu_content,
        submenu_placement: submenu_placement.unwrap_or(MenuPlacement::RightStart),
        selected,
        selection_mode,
        selection,
        shape: shape.unwrap_or_else(|| MenuDefaults::item_shape(position, selected)),
        container_color: container_color.unwrap_or_else(|| {
            if selected && enabled {
                color_style.selected_container(&scheme)
            } else {
                Color::TRANSPARENT
            }
        }),
        enabled,
        close_on_click: close_on_click.unwrap_or(true),
        height: height.unwrap_or(MENU_ITEM_HEIGHT),
        label_color: label_color.unwrap_or_else(|| color_style.content(&scheme, selected)),
        supporting_color: supporting_color
            .unwrap_or_else(|| color_style.supporting(&scheme, selected)),
        disabled_color: disabled_color.unwrap_or(
            color_style
                .content(&scheme, false)
                .with_alpha(MaterialAlpha::DISABLED_CONTENT),
        ),
        on_click,
    };

    if args.close_on_click
        && args.on_click.is_some()
        && args.submenu_content.is_none()
        && let Some(menu) = args.menu
    {
        let host = menu.host;
        let previous = args.on_click;
        args.on_click = Some(Callback::new(move || {
            if let Some(callback) = previous {
                callback.call();
            }
            host.close_all();
        }));
    }

    menu_item_inner().args(args);
}

#[tessera]
fn menu_item_surface(
    item: Option<MenuItemConfig>,
    submenu_request: Option<State<MenuBranch>>,
    focus_requester: Option<FocusRequester>,
) {
    let item = item.unwrap_or_default();
    let enabled = item.enabled && (item.on_click.is_some() || item.submenu_content.is_some());
    let has_submenu = item.submenu_content.is_some();
    let focus_requester = if let Some(requester) = focus_requester {
        requester
    } else {
        remember(FocusRequester::new).get()
    };
    let typeahead = item.menu.map(|menu| menu.typeahead);
    let _typeahead_lease =
        remember(|| typeahead.map(|context| typeahead::ItemLease::new(context.0, focus_requester)));
    if let Some(context) = typeahead {
        typeahead::update(context.0, focus_requester, &item.label, enabled);
    }
    let hover = remember(MenuHoverIntent::default);
    let host = item.menu.map(|menu| menu.host);
    let parent = item.menu.map(|menu| menu.controller);
    if enabled && hover.with(|hover| hover.deadline.is_some()) {
        launch_once(async move {
            loop {
                let now = with_frame_time(|now| now).await;
                let Some(mut deadline) = hover.with(|hover| hover.deadline) else {
                    break;
                };
                if !hover.with(|hover| hover.inside)
                    || !focus_requester.is_focused()
                    || parent.is_none_or(|parent| !parent.with(MenuController::is_open))
                {
                    hover.with_mut(|hover| hover.deadline = None);
                    break;
                }
                if !deadline.elapsed(now) {
                    continue;
                }
                hover.with_mut(|hover| hover.deadline = None);
                if let Some((host, parent)) = host.as_ref().zip(parent) {
                    host.apply_hover(parent, submenu_request);
                }
                break;
            }
        });
    }
    let modifier = if let Some((host, request)) = host.zip(submenu_request) {
        let placement = item.submenu_placement;
        with_keyboard_input(Modifier::new(), move |mut input| {
            if input.key_modifiers.control_key()
                || input.key_modifiers.alt_key()
                || input.key_modifiers.super_key()
            {
                return;
            }
            let value = request.get();
            if !value.enabled {
                return;
            }
            let opening = if placement == MenuPlacement::LeftStart {
                winit::keyboard::NamedKey::ArrowLeft
            } else {
                winit::keyboard::NamedKey::ArrowRight
            };
            let should_open = input.keyboard_events.iter().any(|event| {
                event.state == winit::event::ElementState::Pressed
                    && event.logical_key == winit::keyboard::Key::Named(opening)
            });
            if should_open && !value.controller.with(MenuController::is_open) {
                host.open(request);
                input.block_keyboard();
            }
        })
    } else {
        Modifier::new()
    };

    let modifier = if let Some(context) = typeahead {
        modifier.on_layout_rect_in_space(move |bounds| {
            typeahead::position(context.0, focus_requester, bounds);
        })
    } else {
        modifier
    };
    let submenu = if let Some(request) = submenu_request {
        let host = host.expect("submenu cascade");
        let branch = request.get();
        let dismiss_requested = remember(|| false);
        if dismiss_requested.get() {
            dismiss_requested.set(false);
            host.remove(request, branch.parent, true);
        }
        branch
            .controller
            .with(MenuController::is_open)
            .then_some((branch, dismiss_requested))
    } else {
        None
    };
    layout(Stack).modifier(modifier).child(move || {
        let on_click = if has_submenu {
            let request = submenu_request.expect("submenu request");
            let host = host.expect("submenu cascade");
            Some(Callback::new(move || {
                host.open(request);
            }))
        } else {
            item.on_click
        };
        let theme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get();
        let label_color = if enabled {
            item.label_color
        } else {
            item.disabled_color
        };
        let supporting_color = if enabled {
            item.supporting_color
        } else {
            item.disabled_color
        };
        let selection = item.selection;
        let expanded = has_submenu.then_some(submenu.is_some());
        surface()
            .style(SurfaceStyle::Filled {
                color: item.container_color,
            })
            .shape(item.shape)
            .enabled(enabled)
            .modifier(
                Modifier::new()
                    .fill_max_width()
                    .padding_symmetric(Dp(4.0), Dp::ZERO)
                    .minimum_interactive_component_size()
                    .push_pointer_preview_input(MenuItemHoverFocus {
                        enabled,
                        requester: focus_requester,
                        host,
                        controller: parent,
                        request: submenu_request,
                        hover,
                    })
                    .push_semantics(MenuItemDisclosure(expanded)),
            )
            .accessibility_role(match item.selection_mode {
                Some(MenuItemSelectionMode::Radio) => Role::MenuItemRadio,
                Some(MenuItemSelectionMode::Checkbox) => Role::MenuItemCheckBox,
                None => Role::MenuItem,
            })
            .accessibility_toggled_optional(selection)
            .accessibility_label(item.label.clone())
            .block_input(true)
            .ripple_color(label_color)
            .focus_requester(focus_requester)
            .on_click_optional(on_click)
            .accessibility_description_optional(item.supporting_text.clone())
            .child({
                let item = item.clone();
                move || {
                    let item = item.clone();
                    row()
                        .modifier(
                            Modifier::new()
                                .fill_max_width()
                                .constrain(
                                    None,
                                    Some(AxisConstraint::at_least(Px::from(item.height))),
                                )
                                .padding_symmetric(MENU_HORIZONTAL_PADDING, Dp(8.0)),
                        )
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            if item.selected {
                                checkmark()
                                    .color(label_color)
                                    .size(MENU_LEADING_SIZE)
                                    .padding([2.0, 2.0]);
                                spacer().modifier(Modifier::new().width(MENU_TRAILING_SPACING));
                            } else if let Some(painter) = item.leading_icon.clone() {
                                icon()
                                    .painter(painter)
                                    .size(MENU_LEADING_SIZE)
                                    .tint(supporting_color);
                                spacer().modifier(Modifier::new().width(MENU_TRAILING_SPACING));
                            }
                            let label = item.label.clone();
                            let supporting = item.supporting_text.clone();
                            column()
                                .modifier(Modifier::new().weight(1.0))
                                .cross_axis_alignment(CrossAxisAlignment::Start)
                                .children(move || {
                                    text(label.clone())
                                        .style(theme.typography.body_large)
                                        .color(label_color);
                                    if let Some(supporting) = supporting.as_ref() {
                                        text(supporting.clone())
                                            .style(theme.typography.body_medium)
                                            .color(supporting_color);
                                    }
                                });
                            if item.trailing_icon.is_some()
                                || item.trailing_text.is_some()
                                || has_submenu
                            {
                                spacer().modifier(Modifier::new().width(MENU_TRAILING_SPACING));
                                if let Some(painter) = item.trailing_icon.clone() {
                                    icon()
                                        .painter(painter)
                                        .size(MENU_LEADING_SIZE)
                                        .tint(supporting_color);
                                } else if let Some(trailing) = item.trailing_text.clone() {
                                    text(trailing)
                                        .style(theme.typography.label_small)
                                        .color(supporting_color);
                                }
                                if has_submenu {
                                    if item.trailing_icon.is_some() || item.trailing_text.is_some()
                                    {
                                        spacer()
                                            .modifier(Modifier::new().width(MENU_TRAILING_SPACING));
                                    }
                                    icon()
                                        .painter(
                                            if item.submenu_placement == MenuPlacement::LeftStart {
                                                filled::CHEVRON_LEFT_SVG
                                            } else {
                                                filled::CHEVRON_RIGHT_SVG
                                            },
                                        )
                                        .size(MENU_LEADING_SIZE)
                                        .tint(supporting_color);
                                }
                            }
                        });
                }
            });
        if let Some((branch, dismiss_requested)) = submenu {
            let host = host.expect("submenu cascade");
            let color_style = use_context::<MenuColorStyle>()
                .map(|style| style.get())
                .unwrap_or_default();
            let config = MenuConfig {
                placement: item.submenu_placement,
                offset: [Dp::ZERO; 2],
                color_style,
                container_color: color_style.container(&theme.color_scheme),
                close_on_escape: host.close_on_escape,
                ..Default::default()
            };
            menu_popup_layer(
                config,
                host,
                branch.controller,
                branch.scope,
                branch.bounds,
                item.submenu_content.expect("submenu content"),
                dismiss_requested,
                true,
            );
        }
    });
}

struct MenuItemHoverFocus {
    enabled: bool,
    requester: FocusRequester,
    host: Option<MenuCascade>,
    controller: Option<State<MenuController>>,
    request: Option<State<MenuBranch>>,
    hover: State<MenuHoverIntent>,
}

#[derive(Default)]
struct MenuHoverIntent {
    inside: bool,
    mouse: bool,
    deadline: Option<Delay>,
}

impl PointerInputModifierNode for MenuItemHoverFocus {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        let moved_at = input
            .pointer_changes
            .iter()
            .filter(|change| {
                change.pointer_id == MOUSE_POINTER_ID
                    && matches!(change.content, CursorEventContent::Moved(_))
            })
            .map(|change| change.timestamp)
            .max();
        let moved = moved_at.is_some();
        let mouse = input
            .pointer_changes
            .last()
            .map(|change| change.pointer_id == MOUSE_POINTER_ID)
            .unwrap_or_else(|| self.hover.with(|hover| hover.mouse));
        let inside = self.enabled
            && mouse
            && input.cursor_position_rel.is_some_and(|point| {
                point.x >= 0.0
                    && point.y >= 0.0
                    && point.x < input.computed_data.width.to_f32()
                    && point.y < input.computed_data.height.to_f32()
            })
            && self
                .host
                .as_ref()
                .zip(self.controller)
                .is_none_or(|(host, controller)| {
                    host.allows_hover(controller, input.cursor_position_abs())
                });
        let schedule = inside
            && moved
            && self.hover.with(|hover| hover.deadline.is_none())
            && self
                .host
                .as_ref()
                .zip(self.controller)
                .is_some_and(|(host, parent)| {
                    if let Some(request) = self.request {
                        !request.get().controller.with(MenuController::is_open)
                    } else {
                        host.has_descendants(parent)
                    }
                });
        if schedule
            || self
                .hover
                .with(|hover| hover.inside != inside || hover.mouse != mouse)
        {
            self.hover.with_mut(|hover| {
                hover.inside = inside;
                hover.mouse = mouse;
                if !inside {
                    hover.deadline = None;
                } else if schedule {
                    hover.deadline = Some(Delay::new(moved_at, MENU_HOVER_DELAY));
                }
            });
        }
        if inside && moved {
            self.requester.request_focus();
        }
    }
}

struct MenuItemDisclosure(Option<bool>);

impl tessera_ui::SemanticsModifierNode for MenuItemDisclosure {
    fn apply(
        &self,
        node: &mut tessera_ui::AccessibilityNode,
        _handler: &mut Option<tessera_ui::AccessibilityActionHandler>,
    ) {
        node.expanded = self.0;
        node.has_popup = self.0.map(|_| HasPopup::Menu);
    }
}

#[tessera]
fn menu_item_inner(args: Option<MenuItemConfig>) {
    let args = args.unwrap_or_default();
    let submenu_content = args.submenu_content;

    if submenu_content.is_some() {
        let submenu_controller = remember(MenuController::new);
        let focus_requester = remember(FocusRequester::new).get();
        let menu = args.menu.expect("submenu items require a menu scope");
        let host = menu.host;
        let parent = menu.controller;
        let value = MenuBranch {
            parent,
            controller: submenu_controller,
            scope: remember(FocusScopeNode::new).get(),
            enabled: args.enabled,
            bounds: remember(|| None),
        };
        let request = remember(|| value);
        request.set_if_changed(value);
        let _lease = remember(|| MenuBranchLease {
            cascade: host,
            entry: request,
            parent,
        });
        if !args.enabled {
            host.remove(request, parent, true);
        }
        menu_item_surface()
            .item(args)
            .submenu_request(request)
            .focus_requester(focus_requester);
    } else {
        menu_item_surface().item(args);
    }
}

#[cfg(test)]
mod tests {
    use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
    use tessera_ui::testing::UiTest;
    use tessera_ui::{
        Dp, LayoutCoordinates, Modifier, Px, PxPosition, State, modifier::FocusModifierExt as _,
        remember, scaled_box, tessera, testing::LayoutSnapshot,
    };

    use crate::{
        animation::test_instant,
        column::column,
        md3::{button::button, theme::material_theme},
        spacer::spacer,
    };

    use super::{MenuAnchor, MenuController, menu_anchor, menu_item};

    #[derive(Clone, Copy, Default)]
    struct FixtureHandles {
        typeahead: Option<State<super::typeahead::Registry>>,
        child_typeahead: Option<State<super::typeahead::Registry>>,
        remove_delete: Option<State<bool>>,
        controller: Option<State<MenuController>>,
        main_state: Option<State<u32>>,
    }

    #[tessera]
    fn typeahead_fixture(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
        let controller = remember(|| {
            let mut controller = MenuController::new();
            controller.open();
            controller
        });
        let remove_delete = remember(|| false);
        observed.with_mut(|handles| handles.controller = Some(controller));
        observed.with_mut(|handles| handles.remove_delete = Some(remove_delete));
        material_theme().child(move || {
            let observed = observed.clone();
            menu_anchor()
                .controller(controller)
                .anchor_content(|| {
                    crate::spacer::spacer().modifier(Modifier::new().fill_max_size());
                })
                .menu_content(move |menu| {
                    let registry = menu.typeahead.0;
                    observed.with_mut(|handles| handles.typeahead = Some(registry));
                    menu_item().menu(menu).label("Preview").on_click(|| {});
                    menu_item().menu(menu).label("Share").submenu_content({
                        let observed = observed.clone();
                        move |menu| {
                            let registry = menu.typeahead.0;
                            observed.with_mut(|handles| handles.child_typeahead = Some(registry));
                            menu_item().menu(menu).label("Copy link").on_click(|| {});
                            menu_item().menu(menu).label("Send").on_click(|| {});
                        }
                    });
                    if !remove_delete.get() {
                        menu_item().menu(menu).label("Delete").on_click(|| {});
                    }
                    menu_item().menu(menu).label("Duplicate").on_click(|| {});
                    menu_item()
                        .menu(menu)
                        .label("Download")
                        .enabled(false)
                        .on_click(|| {});
                });
        });
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn gpu_menu_prefix_matching_moves_focus_skips_disabled_items_and_cleans_removed_items() {
        use tessera_ui::{
            accesskit::{Action, Role},
            testing::UiTest,
        };

        let observed = tessera_ui::testing::TestProbe::default();
        let mut renderer = UiTest::new()
            .viewport(600, 400)
            .rendered(crate::TesseraComponents)
            .content(|| typeahead_fixture(observed.clone()));
        renderer.frames(6);
        let registry = observed.get().typeahead.unwrap();
        let focused = |renderer: &UiTest| {
            let tree = renderer.accessibility_snapshot().unwrap();
            tree.nodes
                .iter()
                .find(|(id, _)| *id == tree.focus)
                .and_then(|(_, node)| node.label())
                .map(str::to_owned)
        };
        assert_eq!(focused(&renderer).as_deref(), Some("Preview"));
        for (text, time, expected) in [
            ("d", 0, "Delete"),
            ("D", 100_000_000, "Duplicate"),
            ("e", 200_000_000, "Delete"),
            ("du", 2_000_000_000, "Duplicate"),
            ("do", 4_000_000_000, "Duplicate"),
            ("d", 6_000_000_000, "Delete"),
        ] {
            assert!(super::typeahead::type_text(
                registry,
                text,
                test_instant(time)
            ));
            renderer.frames(2);
            assert_eq!(
                focused(&renderer).as_deref(),
                Some(expected),
                "prefix {text}"
            );
        }
        observed.get().remove_delete.unwrap().set(true);
        renderer.frames(3);
        super::typeahead::type_text(registry, "d", test_instant(8_000_000_000));
        renderer.frames(2);
        assert_eq!(focused(&renderer).as_deref(), Some("Duplicate"));
        let tree = renderer.accessibility_snapshot().unwrap();
        let share = tree
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::MenuItem && node.label() == Some("Share"))
            .unwrap()
            .0;
        assert!(renderer.accessibility_action(share, Action::Click));
        renderer.frames(6);
        let child = observed.get().child_typeahead.unwrap();
        super::typeahead::type_text(child, "s", test_instant(0));
        renderer.frames(2);
        assert_eq!(focused(&renderer).as_deref(), Some("Send"));
        observed
            .get()
            .controller
            .unwrap()
            .with_mut(MenuController::close);
        renderer.frames(4);
        assert!(!registry.is_alive());
        assert!(!child.is_alive());
    }

    #[tessera]
    fn menu_controller_fixture(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
        let controller = remember(MenuController::new);
        observed.with_mut(|handles| handles.controller = Some(controller));
        material_theme().child(move || {
            menu_anchor()
                .controller(controller)
                .anchor_content({
                    let observed = observed.clone();
                    move || {
                        let state = remember(|| 0_u32);
                        observed.with_mut(|handles| handles.main_state = Some(state));
                        column().children(|| {});
                    }
                })
                .menu_content(|menu| {
                    column()
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            test_tag: Some("menu_content_tag".to_string()),
                            ..Default::default()
                        }))
                        .children(move || {
                            menu_item().menu(menu).label("Edit");
                        });
                });
        });
    }

    #[tessera]
    fn menu_background_fixture(
        close_on_background: bool,
        observed: tessera_ui::testing::TestProbe<FixtureHandles>,
    ) {
        let controller = remember(MenuController::new);
        let state = remember(|| 0_u32);
        observed.with_mut(|handles| handles.controller = Some(controller));
        observed.with_mut(|handles| handles.main_state = Some(state));
        material_theme().child(move || {
            menu_anchor()
                .controller(controller)
                .close_on_background(close_on_background)
                .anchor_content(move || {
                    button()
                        .modifier(Modifier::new().fill_max_size())
                        .on_click(move || state.with_mut(|count| *count += 1));
                })
                .menu_content(|menu| {
                    column()
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            test_tag: Some("menu_background_content".to_string()),
                            ..Default::default()
                        }))
                        .children(move || {
                            menu_item().menu(menu).label("Edit").enabled(false);
                        });
                });
        });
    }

    #[test]
    fn menu_background_dismisses_without_activating_underlying_content() {
        for (close_on_background, click_inside, expected_open) in [
            (true, false, false),
            (false, false, true),
            (true, true, true),
        ] {
            let observed = tessera_ui::testing::TestProbe::default();
            let mut session = UiTest::new()
                .content(|| menu_background_fixture(close_on_background, observed.clone()))
                .viewport(400, 300);
            session.frame();
            let controller = observed.get().controller.expect("controller");
            controller.with_mut(|menu| {
                menu.open_at(MenuAnchor::at(PxPosition::new(Px(10), Px(10))));
            });
            let opened = session.frames(1);
            let bounds = opened.node("menu_background_content").bounds();
            session.click_at(if click_inside {
                PxPosition::new(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2)
            } else {
                PxPosition::new(Px(350), Px(250))
            });
            assert_eq!(
                controller.with(MenuController::is_open),
                expected_open,
                "close_on_background={close_on_background}, click_inside={click_inside}"
            );
            assert_eq!(observed.get().main_state.unwrap().get(), 0);
        }
    }

    #[test]
    fn menu_renders_menu_items_after_controller_opens() {
        let observed = tessera_ui::testing::TestProbe::default();
        let mut session = UiTest::new()
            .content(|| menu_controller_fixture(observed.clone()))
            .viewport(400, 300);
        session.frame();

        let controller = observed.get().controller.expect("controller");
        controller.with_mut(|menu| {
            menu.open_at(MenuAnchor::at(PxPosition::new(Px(10), Px(10))));
        });

        let opened: LayoutSnapshot = session.frames(1);
        opened.node("menu_content_tag").exists();
    }

    #[tessera]
    fn window_anchor_fixture(observed: tessera_ui::testing::TestProbe<Option<State<Dp>>>) {
        let anchor = remember(LayoutCoordinates::default);
        let x = remember(|| Dp(40.0));
        observed.set(Some(x));
        material_theme().child(move || {
            tessera_ui::layout::layout(tessera_ui::layout::Stack)
                .modifier(Modifier::new().offset(Dp(12.0), Dp(18.0)))
                .child(move || {
                    scaled_box(0.5).child(move || {
                        menu_anchor()
                            .window_anchor(anchor)
                            .anchor(MenuAnchor::at(PxPosition::new(Px(999), Px(999))))
                            .controller(remember(|| {
                                let mut controller = MenuController::new();
                                controller.open();
                                controller
                            }))
                            .placement(super::MenuPlacement::BelowStart)
                            .offset([Dp::ZERO; 2])
                            .modifier(Modifier::new().width(Dp(80.0)).semantics(SemanticsArgs {
                                test_tag: Some("anchored-menu".into()),
                                ..Default::default()
                            }))
                            .anchor_content(move || {
                                spacer().modifier(
                                    Modifier::new()
                                        .size(Dp(80.0), Dp(40.0))
                                        .offset(x.get(), Dp(30.0))
                                        .on_layout_coordinates(move |coordinates| {
                                            anchor.set_if_changed(coordinates);
                                        }),
                                );
                            })
                            .menu_content(|_| {
                                spacer().modifier(Modifier::new().height(Dp(20.0)));
                            });
                    });
                });
        });
    }

    #[test]
    fn window_anchor_follows_trigger_geometry_through_ancestor_transforms() {
        let observed = tessera_ui::testing::TestProbe::default();
        let mut session = UiTest::new()
            .content(|| {
                window_anchor_fixture(observed.clone());
            })
            .viewport(400, 300);
        session.frames(2).node("anchored-menu").position(32, 53);
        observed.get().expect("trigger position").set(Dp(100.0));
        session.frames(2).node("anchored-menu").position(62, 53);
    }

    #[test]
    fn menu_visibility_preserves_anchor_content_state() {
        let observed = tessera_ui::testing::TestProbe::default();
        let mut session = UiTest::new()
            .content(|| menu_controller_fixture(observed.clone()))
            .viewport(400, 300);
        session.frame();
        let main_state = observed.get().main_state.expect("main_state");
        main_state.set(7);
        let controller = observed.get().controller.expect("controller");
        controller.with_mut(|menu| {
            menu.open_at(MenuAnchor::at(PxPosition::new(Px(10), Px(10))));
        });
        session.frames(1);
        assert!(observed.get().main_state.unwrap() == main_state);
        assert_eq!(main_state.get(), 7);

        controller.with_mut(MenuController::close);
        session.frames(1);
        assert!(observed.get().main_state.unwrap() == main_state);
        assert_eq!(main_state.get(), 7);
    }
}

#[cfg(test)]
#[path = "menus/focus_tests.rs"]
mod focus_tests;

#[cfg(test)]
#[path = "menus/appearance_tests.rs"]
mod appearance_tests;

#[cfg(test)]
#[path = "menus/selection_tests.rs"]
mod selection_tests;

#[cfg(test)]
#[path = "menus/submenu_tests.rs"]
mod submenu_tests;

#[cfg(test)]
#[path = "menus/shadow_tests.rs"]
mod shadow_tests;

#[cfg(test)]
#[path = "menus/placement_tests.rs"]
mod placement_tests;

#[cfg(test)]
#[path = "menus/motion_tests.rs"]
mod motion_tests;

#[cfg(test)]
mod navigation_tests {
    use tessera_ui::winit::keyboard::NamedKey;

    use super::{MenuPlacement, closes_submenu};

    #[test]
    fn horizontal_close_key_applies_only_to_submenus_and_their_return_direction() {
        for (placement, close, open) in [
            (
                MenuPlacement::RightStart,
                NamedKey::ArrowLeft,
                NamedKey::ArrowRight,
            ),
            (
                MenuPlacement::LeftStart,
                NamedKey::ArrowRight,
                NamedKey::ArrowLeft,
            ),
        ] {
            assert!(!closes_submenu(false, placement, close));
            assert!(!closes_submenu(false, placement, open));
            assert!(closes_submenu(true, placement, close));
            assert!(!closes_submenu(true, placement, open));
            assert!(!closes_submenu(true, placement, NamedKey::ArrowDown));
        }
        assert!(!closes_submenu(
            true,
            MenuPlacement::BelowStart,
            NamedKey::ArrowLeft
        ));
    }
}
