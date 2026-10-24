//! Material Design 3 menus for contextual action lists.
//!
//! ## Usage
//!
//! Present anchored overflow or context actions as surfaced menus.
use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui::{
    AxisConstraint, Callback, ClipRegion, ClipShape, Color, ComputedData, CursorEventContent, Dp,
    DrawModifierContent, DrawModifierContext, DrawModifierNode, FocusDirection, FocusManager,
    FocusRequester, FocusScopeNode, FocusTraversalPolicy, FrameNanosControl, KeyboardInput,
    KeyboardInputModifierNode, LayoutResult, MOUSE_POINTER_ID, MeasurementError, Modifier,
    ParentConstraint, PointerInput, PointerInputModifierNode, Px, PxPosition, PxSize, RenderSlot,
    State,
    accesskit::{HasPopup, Role, Toggled},
    current_frame_nanos,
    layout::{MeasureScope, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    provide_context, receive_frame_nanos, remember, tessera, use_context, winit,
};

use crate::{
    alignment::CrossAxisAlignment,
    checkmark::checkmark,
    column::column,
    icon::icon,
    material_icons::filled,
    modifier::{ModifierExt as _, with_keyboard_input, with_pointer_input},
    painter::Painter,
    pos_misc::is_position_in_rect,
    row::row,
    scrollable::ScrollableController,
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    text::text,
    theme::{MaterialAlpha, MaterialTheme},
};

pub use appearance::{MenuColorStyle, MenuDefaults, MenuPosition, menu_group};

use host::{MenuHost, MenuPopupLease, MenuPopupRequest, menu_host};
use motion::{MenuShapeHover, MenuShapeMotion};

mod appearance;
mod host;
mod motion;

const MENU_MIN_WIDTH: Dp = Dp(112.0);
const MENU_MAX_WIDTH: Dp = Dp(280.0);
const MENU_MAX_HEIGHT: Dp = Dp(320.0);
const MENU_VERTICAL_GAP: Dp = Dp(4.0);
const MENU_HORIZONTAL_PADDING: Dp = Dp(12.0);
const MENU_LEADING_SIZE: Dp = Dp(20.0);
const MENU_ITEM_HEIGHT: Dp = MenuDefaults::ITEM_HEIGHT;
const MENU_TRAILING_SPACING: Dp = Dp(12.0);
const MENU_HOVER_DELAY_NANOS: u64 = 150_000_000;

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
    /// Top-left corner of the anchor rectangle, relative to the menu container.
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
    /// reused. Otherwise, the menu defaults to anchoring to the provider's
    /// content.
    pub fn open(&mut self) {
        self.is_open = true;
        self.exit_direction = None;
    }

    /// Opens the menu at the provided anchor rectangle.
    ///
    /// An explicit `anchor` prop on the provider takes precedence over this
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

    fn snapshot(&self) -> (bool, Option<MenuAnchor>) {
        (self.is_open, self.anchor)
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

#[derive(Clone, PartialEq)]
struct MenuProviderConfig {
    pub anchor: Option<MenuAnchor>,
    pub precise_anchor: Option<tessera_ui::FloatRect>,
    pub bounds: Option<MenuBoundsHandle>,
    pub focus_scope: Option<FocusScopeNode>,
    pub parent_request: Option<State<MenuPopupRequest>>,
    pub grouped: bool,
    pub color_style: MenuColorStyle,
    pub placement: MenuPlacement,
    pub offset: [Dp; 2],
    pub modifier: Modifier,
    pub max_height: Option<Px>,
    pub shape: Shape,
    pub animate_shape: bool,
    pub elevation: Dp,
    pub container_color: Color,
    pub scrim_color: Color,
    pub close_on_background: bool,
    pub close_on_escape: bool,
    pub on_dismiss: Option<Callback>,
    pub is_open: bool,
    pub controller: Option<State<MenuController>>,
    pub focus_restorer_fallback: Option<FocusRequester>,
    pub main_content: Option<RenderSlot>,
    pub menu_content: Option<RenderSlot>,
}

impl Default for MenuProviderConfig {
    fn default() -> Self {
        Self {
            anchor: None,
            precise_anchor: None,
            bounds: None,
            focus_scope: None,
            parent_request: None,
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
            scrim_color: default_scrim_color(),
            close_on_background: true,
            close_on_escape: true,
            on_dismiss: None,
            is_open: false,
            controller: None,
            focus_restorer_fallback: None,
            main_content: None,
            menu_content: None,
        }
    }
}

#[derive(Clone, PartialEq, Copy)]
struct MenuBounds {
    origin: PxPosition,
    size: ComputedData,
}

#[derive(Clone)]
struct MenuBoundsHandle(Arc<RwLock<Option<MenuBounds>>>);

impl PartialEq for MenuBoundsHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Default for MenuBounds {
    fn default() -> Self {
        Self {
            origin: PxPosition::new(Px::ZERO, Px::ZERO),
            size: ComputedData::ZERO,
        }
    }
}

#[derive(Clone)]
struct MenuLayout {
    is_open: bool,
    has_background: bool,
    placement: MenuPlacement,
    offset: [Dp; 2],
    anchor: Option<tessera_ui::FloatRect>,
    bounds: Arc<RwLock<Option<MenuBounds>>>,
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

impl PartialEq for MenuLayout {
    fn eq(&self, other: &Self) -> bool {
        self.is_open == other.is_open
            && self.has_background == other.has_background
            && self.placement == other.placement
            && self.offset == other.offset
            && self.anchor == other.anchor
    }
}

impl tessera_ui::LayoutModifierNode for MenuLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let child_constraint = input.parent_constraint().without_min();
        let main_content = children
            .first()
            .copied()
            .expect("main content should exist");
        let main_size = main_content.measure(input, &child_constraint)?;
        result.place_child(main_content, PxPosition::new(Px::ZERO, Px::ZERO));

        if !self.is_open {
            *self.bounds.write() = None;
            return Ok(result.with_size(main_size.size()));
        }

        let (menu_index, available) = if self.has_background {
            let background = children[1];
            let background_size = background.measure(input, &child_constraint)?.size();
            result.place_child(background, PxPosition::ZERO);
            let available = if background_size.width > Px::ZERO && background_size.height > Px::ZERO
            {
                background_size
            } else {
                extract_available_size(input.parent_constraint())
            };
            (2, available)
        } else {
            (1, extract_available_size(input.parent_constraint()))
        };
        let menu = children[menu_index];

        let menu_size = menu.measure(input, &child_constraint)?;
        let menu_size = menu_size.size();
        let main_size = main_size.size();
        let anchor = self.anchor.unwrap_or_else(|| {
            tessera_ui::FloatRect::from_position_size(
                PxPosition::ZERO,
                PxSize::new(main_size.width, main_size.height),
            )
        });
        let menu_position =
            resolve_menu_position(anchor, self.placement, menu_size, available, self.offset);
        result.place_child(menu, menu_position);

        *self.bounds.write() = Some(MenuBounds {
            origin: menu_position,
            size: menu_size,
        });

        Ok(result.with_size(main_size))
    }
}

fn resolve_menu_position(
    anchor: tessera_ui::FloatRect,
    placement: MenuPlacement,
    menu_size: ComputedData,
    available: ComputedData,
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
        fit_menu_axis(x, alternate_x, width, available.width.to_f32()),
        fit_menu_axis(y, alternate_y, height, available.height.to_f32()),
    )
}

fn fit_menu_axis(preferred: f32, alternate: f32, extent: f32, available: f32) -> f32 {
    let max = (available - extent).max(0.0);
    if preferred >= 0.0 && preferred <= max {
        preferred
    } else if alternate >= 0.0 && alternate <= max {
        alternate
    } else {
        preferred.clamp(0.0, max)
    }
}

fn extract_available_size(constraint: ParentConstraint<'_>) -> ComputedData {
    let width = constraint.width().resolve_max().unwrap_or(Px::MAX);
    let height = constraint.height().resolve_max().unwrap_or(Px::MAX);

    ComputedData { width, height }
}

fn is_click_outside_menu(cursor_position: Option<PxPosition>, bounds: Option<MenuBounds>) -> bool {
    let Some(bounds) = bounds else {
        return false;
    };

    cursor_position
        .map(|pos| !is_position_in_rect(pos, bounds.origin, bounds.size.width, bounds.size.height))
        .unwrap_or(false)
}

fn apply_close_action(controller: State<MenuController>, on_dismiss: &Option<Callback>) {
    if let Some(callback) = on_dismiss {
        callback.call();
    }
    controller.with_mut(|c| c.close());
}

fn close_for_traversal(
    controller: State<MenuController>,
    on_dismiss: &Option<Callback>,
    reverse: bool,
) {
    apply_close_action(controller, on_dismiss);
    controller.with_mut(|controller| {
        controller.exit_direction = Some(if reverse {
            FocusDirection::Previous
        } else {
            FocusDirection::Next
        });
    });
}

struct MenuKeyboardBoundary {
    controller: State<MenuController>,
    rendered_open: bool,
}

impl MenuKeyboardBoundary {
    fn check(&self) {
        if self.controller.with(MenuController::is_open) != self.rendered_open {
            FocusManager::current().defer_keyboard_until_layout();
        }
    }
}

impl KeyboardInputModifierNode for MenuKeyboardBoundary {
    fn on_keyboard_input(&self, _input: KeyboardInput<'_>) {
        self.check();
    }
}

impl PointerInputModifierNode for MenuKeyboardBoundary {
    fn on_pointer_input(&self, _input: PointerInput<'_>) {
        self.check();
    }
}

/// # menu_provider
///
/// Provides a Material Design 3 menu overlay anchored to a rectangle.
///
/// ## Usage
///
/// Wrap page content and show contextual or overflow actions aligned to a
/// trigger element.
///
/// ## Parameters
///
/// - `anchor` — current anchor relative to the provider; takes precedence over
///   the controller's saved anchor and can follow layout changes
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
/// - `close_on_background` — whether outside clicks dismiss the menu
/// - `close_on_escape` — whether Escape dismisses the menu
/// - `on_dismiss` — optional callback before dismissal
/// - `is_open` — whether the menu is currently visible
/// - `controller` — optional external controller for open state and anchor
/// - `focus_restorer_fallback` — optional fallback focus target on dismiss
/// - `main_content` — optional content rendered behind the menu
/// - `menu_content` — optional menu content slot
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{
///     menus::{MenuAnchor, MenuController, MenuPlacement, menu_item, menu_provider},
///     text::text,
/// };
/// use tessera_ui::{Dp, remember};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// let menu = remember(MenuController::new);
/// menu.with_mut(MenuController::open);
/// assert!(menu.with(MenuController::is_open));
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(move || {
/// menu_provider()
///     .anchor(MenuAnchor::from_dp(
///         (Dp(16.0), Dp(16.0)),
///         (Dp(80.0), Dp(40.0)),
///     ))
///     .placement(MenuPlacement::BelowStart)
///     .controller(menu)
///     .main_content(|| {
///         text().content("Main content");
///     })
///     .menu_content(|| {
///         menu_item().label("Edit").on_click(|| {});
///     });
/// # });
/// # }
/// # component();
/// ```
#[tessera]
pub fn menu_provider(
    anchor: Option<MenuAnchor>,
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
    on_dismiss: Option<Callback>,
    is_open: Option<bool>,
    controller: Option<State<MenuController>>,
    focus_restorer_fallback: Option<FocusRequester>,
    main_content: Option<RenderSlot>,
    menu_content: Option<RenderSlot>,
) {
    let controlled_is_open = is_open;
    let is_open = is_open.unwrap_or(false);
    let grouped = grouped.unwrap_or(false);
    let inherited_style = use_context::<MenuColorStyle>().map(|style| style.get());
    let color_style = color_style.or(inherited_style).unwrap_or_default();
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let mut provider_args = MenuProviderConfig {
        anchor,
        precise_anchor: None,
        bounds: None,
        focus_scope: None,
        parent_request: None,
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
        scrim_color: scrim_color.unwrap_or_else(default_scrim_color),
        close_on_background: close_on_background.unwrap_or(true),
        close_on_escape: close_on_escape.unwrap_or(true),
        on_dismiss,
        is_open,
        controller,
        focus_restorer_fallback,
        main_content,
        menu_content,
    };
    let controller = if let Some(controller) = provider_args.controller {
        controller
    } else {
        remember(MenuController::new)
    };

    // Only a controlled `is_open` prop drives the controller. When the caller
    // manages visibility through the `controller` (the common case), leave the
    // controller untouched so `open_at`/`open`/`close` are not overwritten.
    if let Some(target) = controlled_is_open
        && controller.with(|c| c.is_open()) != target
    {
        controller.with_mut(|c| {
            if target {
                c.open();
            } else {
                c.close();
            }
        });
    }

    provider_args.controller = Some(controller);
    menu_host().provider(provider_args);
}

#[tessera]
fn menu_provider_content(provider: Option<MenuProviderConfig>) {
    let provider_args = provider.unwrap_or_default();
    let controller = provider_args.controller.expect("menu controller");
    let host = use_context::<MenuHost>().expect("menu host").get();

    let main_content = provider_args.main_content.unwrap_or_else(RenderSlot::empty);
    let menu_content = provider_args.menu_content.unwrap_or_else(RenderSlot::empty);
    let menu_open_state = remember(|| false);
    let main_focus_scope = remember(FocusScopeNode::new).get();
    let internal_focus_scope = remember(FocusScopeNode::new).get();
    let menu_focus_scope = provider_args.focus_scope.unwrap_or(internal_focus_scope);
    let pending_focus_restore = remember(|| None::<Option<FocusRequester>>);

    let (is_open, anchor) = controller.with(|c| c.snapshot());
    let anchor = provider_args.precise_anchor.or_else(|| {
        provider_args
            .anchor
            .or(anchor)
            .map(|anchor| tessera_ui::FloatRect::from_position_size(anchor.origin, anchor.size))
    });
    let mut just_opened = false;
    let mut just_closed = false;
    menu_open_state.with_mut(|was_open| {
        just_opened = !*was_open && is_open;
        just_closed = *was_open && !is_open;
        *was_open = is_open;
    });
    if just_opened || just_closed {
        FocusManager::current().defer_keyboard_until_layout();
    }
    let root_restore = just_closed
        && provider_args.parent_request.is_none()
        && (host.take_root_restore() || host.popup_has_focus());
    let can_restore =
        provider_args.parent_request.is_none() || host.root.with(MenuController::is_open);
    if just_closed && can_restore && (menu_focus_scope.has_focus() || root_restore) {
        pending_focus_restore.set(Some(provider_args.focus_restorer_fallback));
    }
    let internal_bounds = remember(|| Arc::new(RwLock::new(None))).get();
    let bounds = provider_args
        .bounds
        .clone()
        .map(|bounds| bounds.0)
        .unwrap_or(internal_bounds);

    // Parent pointer handler: block propagation and close on background click.
    let bounds = bounds.clone();
    let on_dismiss = provider_args.on_dismiss;
    let close_on_escape = provider_args.close_on_escape;
    let close_on_background = provider_args.close_on_background;
    let parent_request = provider_args.parent_request;
    let mut modifier = Modifier::new();
    if pending_focus_restore.get().is_some() {
        // Restore against the rebuilt tree so a newly disabled trigger cannot
        // leave a request waiting to steal focus when it is enabled again.
        modifier = modifier.on_globally_positioned(move |_| {
            if let Some(fallback) = pending_focus_restore.with_mut(Option::take)
                && !controller.with(MenuController::is_open)
            {
                if let Some(fallback) = fallback {
                    fallback.request_focus();
                } else {
                    main_focus_scope.restore_focus();
                }
                if let Some(direction) = controller.with_mut(|menu| menu.exit_direction.take()) {
                    FocusManager::current().request_move_focus(direction);
                }
            }
        });
    }
    if is_open && parent_request.is_none() {
        modifier = with_pointer_input(modifier, {
            let bounds = bounds.clone();
            move |mut input| {
                let cursor_position = input.cursor_position_rel;
                let menu_bounds = *bounds.read();
                let should_close_click = close_on_background
                    && input.has_unconsumed_release()
                    && is_click_outside_menu(cursor_position, menu_bounds);

                input.block_all();

                if should_close_click {
                    apply_close_action(controller, &on_dismiss);
                }
            }
        });
    }

    if is_open && close_on_escape {
        let on_dismiss = provider_args.on_dismiss;
        modifier = with_keyboard_input(modifier, move |mut input| {
            let should_close_escape = input.keyboard_events.iter().any(|event| {
                event.state == winit::event::ElementState::Pressed
                    && matches!(
                        event.physical_key,
                        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape)
                    )
            });
            if should_close_escape {
                apply_close_action(controller, &on_dismiss);
                input.block_keyboard();
            }
        });
    }

    if parent_request.is_none() {
        modifier = modifier.push_pointer_input(MenuKeyboardBoundary {
            controller,
            rendered_open: is_open,
        });
    }
    modifier = modifier.push_keyboard_input(MenuKeyboardBoundary {
        controller,
        rendered_open: is_open,
    });
    layout()
        .modifier(modifier)
        .policy(MenuLayout {
            is_open,
            has_background: parent_request.is_none(),
            placement: provider_args.placement,
            offset: provider_args.offset,
            anchor,
            bounds: bounds.clone(),
        })
        .child(move || {
            layout()
                .modifier(Modifier::new().focus_scope_with(main_focus_scope))
                .child_shared(main_content);
            if is_open {
                let bounds = bounds.clone();
                if parent_request.is_none() {
                    surface()
                        .style(SurfaceStyle::Filled {
                            color: provider_args.scrim_color,
                        })
                        .modifier(with_pointer_input(
                            Modifier::new().fill_max_size(),
                            move |mut input| {
                                let should_close = close_on_background
                                    && input.has_unconsumed_release()
                                    && is_click_outside_menu(
                                        input.cursor_position_rel,
                                        *bounds.read(),
                                    );
                                input.block_all();
                                if should_close {
                                    apply_close_action(controller, &on_dismiss);
                                }
                            },
                        ))
                        .child(|| {});
                }

                menu_panel()
                    .provider(provider_args.clone())
                    .controller(controller)
                    .focus_scope(menu_focus_scope)
                    .menu_content_shared(menu_content)
                    .just_opened(just_opened);
            }
        });
}

#[tessera]
fn menu_panel(
    provider: Option<MenuProviderConfig>,
    controller: Option<State<MenuController>>,
    focus_scope: Option<FocusScopeNode>,
    menu_content: Option<RenderSlot>,
    just_opened: Option<bool>,
) {
    let provider = provider.unwrap_or_default();
    let just_opened = just_opened.unwrap_or(false);
    let controller = controller.expect("menu_panel requires controller");
    let menu_content = menu_content.expect("menu_panel requires menu content");
    let focus_scope = focus_scope.expect("menu_panel requires focus scope");
    let on_dismiss = provider.on_dismiss;
    let placement = provider.placement;
    let scroll = remember(ScrollableController::new);
    let color_style = provider.color_style;
    let grouped = provider.grouped;
    let host = use_context::<MenuHost>().expect("menu host").get();
    let frame = current_frame_nanos();
    let motion = remember(|| MenuShapeMotion::new(MenuPosition::Standalone, frame));
    let focused = focus_scope.has_focus();
    if !grouped && provider.animate_shape {
        if motion.with(|motion| motion.needs_update(MenuPosition::Standalone, focused)) {
            motion.with_mut(|motion| motion.update(MenuPosition::Standalone, focused, frame));
        }
        if !motion.with(MenuShapeMotion::is_idle) {
            receive_frame_nanos(move |frame| motion::advance(motion, frame));
        }
    }
    let hover = MenuShapeHover {
        motion,
        host: Some(host.clone()),
        controller: Some(controller),
    };

    let mut modifier = with_keyboard_input(
        Modifier::new()
            .focus_scope_with(focus_scope)
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

                handled = match event.logical_key {
                    winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab) => {
                        host.exit(input.key_modifiers.shift_key());
                        true
                    }
                    winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft)
                        if placement == MenuPlacement::RightStart =>
                    {
                        apply_close_action(controller, &on_dismiss);
                        true
                    }
                    winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight)
                        if placement == MenuPlacement::LeftStart =>
                    {
                        apply_close_action(controller, &on_dismiss);
                        true
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
    if just_opened {
        modifier = modifier.on_globally_positioned(move |_| {
            if controller.with(MenuController::is_open) {
                focus_scope.request_focus();
            }
        });
    }
    if !grouped && provider.animate_shape {
        modifier = modifier.push_pointer_preview_input(hover);
    }

    layout().modifier(modifier).child(move || {
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
                provide_context(
                    || controller,
                    move || {
                        let inherited =
                            use_context::<State<MenuController>>().expect("menu controller");
                        if inherited.get() != controller {
                            inherited.set(controller);
                        }
                        provide_context(
                            || color_style,
                            move || {
                                let inherited =
                                    use_context::<MenuColorStyle>().expect("menu colors");
                                if inherited.get() != color_style {
                                    inherited.set(color_style);
                                }
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
                                    .modifier(modifier.vertical_scroll(scroll).padding_symmetric(
                                        Dp::ZERO,
                                        if grouped { Dp::ZERO } else { Dp(2.0) },
                                    ))
                                    .cross_axis_alignment(CrossAxisAlignment::Start)
                                    .children_shared(menu_content);
                            },
                        );
                    },
                );
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
    pub label: String,
    pub supporting_text: Option<String>,
    pub trailing_text: Option<String>,
    pub leading_icon: Option<Painter>,
    pub trailing_icon: Option<Painter>,
    pub submenu_content: Option<RenderSlot>,
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
/// Material Design 3 menu item for use inside [`menu_provider`].
///
/// ## Usage
///
/// Build contextual or overflow menu actions inside
/// `menu_provider().menu_content(...)`.
///
/// ## Parameters
///
/// - `label` — primary text shown for the menu action
/// - `supporting_text` — optional supporting text below the label
/// - `trailing_text` — optional trailing text such as a shortcut hint
/// - `leading_icon` — optional icon shown on the leading side
/// - `trailing_icon` — optional icon shown on the trailing side
/// - `submenu_content` — optional submenu opened by activation or mouse dwell
/// - `submenu_placement` — optional submenu placement override
/// - `selected` — whether the item renders as selected
/// - `selection_mode` — optional radio or checkbox semantics; omitted for an
///   action
/// - `position` — item position within its group, controlling corner shapes
/// - `shape` — optional fixed item shape override
/// - `container_color` — optional item background override
/// - `color_style` — optional standard or vibrant color override
/// - `enabled` — whether the item can be activated
/// - `close_on_click` — whether activation closes the surrounding menu
/// - `height` — optional item height override
/// - `label_color` — optional label text color override
/// - `supporting_color` — optional supporting or trailing text color override
/// - `disabled_color` — optional disabled content color override
/// - `on_click` — optional activation callback
///
/// ## Examples
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{
///     menus::{menu_item, menu_provider},
///     text::text,
/// };
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// menu_provider()
///     .is_open(true)
///     .menu_content(|| {
///         menu_item().label("Open");
///     })
///     .main_content(|| {
///         text().content("Main content");
///     });
/// # });
/// # }
/// # component();
/// ```
#[tessera]
pub fn menu_item(
    #[prop(into)] label: Option<String>,
    #[prop(into)] supporting_text: Option<String>,
    #[prop(into)] trailing_text: Option<String>,
    #[prop(into)] leading_icon: Option<Painter>,
    #[prop(into)] trailing_icon: Option<Painter>,
    submenu_content: Option<RenderSlot>,
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
        && let Some(controller_context) = use_context::<State<MenuController>>()
    {
        let controller = controller_context.get();
        let host = use_context::<MenuHost>();
        let previous = args.on_click;
        args.on_click = Some(Callback::new(move || {
            if let Some(callback) = previous {
                callback.call();
            }
            if let Some(host) = host {
                host.get().close_all();
            } else {
                controller.with_mut(MenuController::close);
            }
        }));
    }

    menu_item_inner().args(args);
}

#[tessera]
fn menu_item_surface(
    item: Option<MenuItemConfig>,
    submenu_controller: Option<State<MenuController>>,
    submenu_request: Option<State<MenuPopupRequest>>,
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
    let hover = remember(MenuHoverIntent::default);
    let host = use_context::<MenuHost>().map(|host| host.get());
    let parent = use_context::<State<MenuController>>().map(|controller| controller.get());
    if enabled && hover.with(|hover| hover.deadline.is_some()) {
        let host = host.clone();
        receive_frame_nanos(move |frame| {
            let Some(deadline) = hover.with(|hover| hover.deadline) else {
                return FrameNanosControl::Stop;
            };
            if !hover.with(|hover| hover.inside)
                || !focus_requester.is_focused()
                || parent.is_none_or(|parent| !parent.with(MenuController::is_open))
            {
                hover.with_mut(|hover| hover.deadline = None);
                return FrameNanosControl::Stop;
            }
            if frame < deadline {
                return FrameNanosControl::Continue;
            }
            hover.with_mut(|hover| hover.deadline = None);
            if let Some((host, parent)) = host.as_ref().zip(parent) {
                host.apply_hover(parent, submenu_request);
            }
            FrameNanosControl::Stop
        });
    }
    let modifier = if let Some(request) = submenu_request {
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
            let opening = if value.placement == MenuPlacement::LeftStart {
                winit::keyboard::NamedKey::ArrowLeft
            } else {
                winit::keyboard::NamedKey::ArrowRight
            };
            let should_open = input.keyboard_events.iter().any(|event| {
                event.state == winit::event::ElementState::Pressed
                    && event.logical_key == winit::keyboard::Key::Named(opening)
            });
            if should_open {
                MenuPopupRequest::open(request);
                input.block_keyboard();
            }
        })
        .on_layout_coordinates(move |bounds| {
            let anchor = request.get().anchor;
            if anchor.get() != bounds {
                anchor.set(bounds);
            }
        })
    } else {
        Modifier::new()
    };

    layout().modifier(modifier).child(move || {
        let on_click = if has_submenu {
            let request = submenu_request.expect("submenu request");
            Some(Callback::new(move || {
                MenuPopupRequest::open(request);
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
        let expanded =
            submenu_controller.map(|controller| controller.with(MenuController::is_open));
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
                        host: host.clone(),
                        controller: parent,
                        request: submenu_request,
                        hover,
                    })
                    .push_semantics(MenuItemDisclosure(expanded)),
            )
            .accessibility_role(if !has_submenu {
                match item.selection_mode {
                    Some(MenuItemSelectionMode::Radio) => Role::MenuItemRadio,
                    Some(MenuItemSelectionMode::Checkbox) => Role::MenuItemCheckBox,
                    None => Role::MenuItem,
                }
            } else {
                Role::MenuItem
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
                                    text()
                                        .content(label.clone())
                                        .style(theme.typography.body_large)
                                        .color(label_color);
                                    if let Some(supporting) = supporting.as_ref() {
                                        text()
                                            .content(supporting.clone())
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
                                    text()
                                        .content(trailing)
                                        .style(theme.typography.label_small)
                                        .color(supporting_color);
                                } else {
                                    icon()
                                        .painter(filled::CHEVRON_RIGHT_SVG)
                                        .size(MENU_LEADING_SIZE)
                                        .tint(supporting_color);
                                }
                            }
                        });
                }
            });
    });
}

struct MenuItemHoverFocus {
    enabled: bool,
    requester: FocusRequester,
    host: Option<MenuHost>,
    controller: Option<State<MenuController>>,
    request: Option<State<MenuPopupRequest>>,
    hover: State<MenuHoverIntent>,
}

#[derive(Default)]
struct MenuHoverIntent {
    inside: bool,
    mouse: bool,
    deadline: Option<u64>,
}

impl PointerInputModifierNode for MenuItemHoverFocus {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        let moved = input.pointer_changes.iter().any(|change| {
            change.pointer_id == MOUSE_POINTER_ID
                && matches!(change.content, CursorEventContent::Moved(_))
        });
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
                    hover.deadline =
                        Some(current_frame_nanos().saturating_add(MENU_HOVER_DELAY_NANOS));
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

    if let Some(submenu_content) = submenu_content {
        let submenu_controller = remember(MenuController::new);
        let focus_requester = remember(FocusRequester::new).get();
        let host = use_context::<MenuHost>()
            .expect("submenu items require a menu provider")
            .get();
        let parent = use_context::<State<MenuController>>()
            .expect("submenu parent")
            .get();
        let value = MenuPopupRequest {
            host: host.clone(),
            parent,
            controller: submenu_controller,
            anchor: remember(tessera_ui::LayoutCoordinates::default),
            scope: remember(FocusScopeNode::new).get(),
            fallback: focus_requester,
            content: submenu_content,
            placement: args.submenu_placement,
            color_style: use_context::<MenuColorStyle>()
                .map(|style| style.get())
                .unwrap_or_default(),
            enabled: args.enabled,
            bounds: remember(|| Arc::new(RwLock::new(None))).get(),
        };
        let request = remember(|| value.clone());
        if request.get() != value {
            request.set(value);
        }
        let scope = request.get().scope;
        let _lease = remember(|| MenuPopupLease {
            host,
            request,
            parent,
            scope,
        });
        if !args.enabled {
            request.get().host.close_disabled_branch(request);
        }
        menu_item_surface()
            .item(args)
            .submenu_controller(submenu_controller)
            .submenu_request(request)
            .focus_requester(focus_requester);
    } else {
        menu_item_surface().item(args);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use tessera_ui::testing::{
        layout_test, pump_layout_test_session, pump_layout_test_session_with_input,
        start_layout_test_session,
    };
    use tessera_ui::{
        CursorEventContent, GestureState, Modifier, PointerChange, PressKeyEventType, Px,
        PxPosition, State, remember, tessera, testing::LayoutSnapshot,
    };

    use crate::{
        button::button,
        column::column,
        modifier::{ModifierExt as _, SemanticsArgs},
        theme::material_theme,
    };

    use super::{MenuAnchor, MenuController, menu_item, menu_provider};

    thread_local! {
        static MENU_CONTROLLER: RefCell<Option<State<MenuController>>> =
            const { RefCell::new(None) };
        static MAIN_STATE: RefCell<Option<State<u32>>> = const { RefCell::new(None) };
    }

    #[tessera]
    fn menu_controller_fixture() {
        let controller = remember(MenuController::new);
        MENU_CONTROLLER.with(|slot| *slot.borrow_mut() = Some(controller));
        material_theme().child(move || {
            menu_provider()
                .controller(controller)
                .main_content(|| {
                    let state = remember(|| 0_u32);
                    MAIN_STATE.with(|slot| *slot.borrow_mut() = Some(state));
                    column().children(|| {});
                })
                .menu_content(|| {
                    column()
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            test_tag: Some("menu_content_tag".to_string()),
                            ..Default::default()
                        }))
                        .children(|| {
                            menu_item().label("Edit");
                        });
                });
        });
    }

    #[tessera]
    fn menu_background_fixture(close_on_background: bool) {
        let controller = remember(MenuController::new);
        let state = remember(|| 0_u32);
        MENU_CONTROLLER.with(|slot| *slot.borrow_mut() = Some(controller));
        MAIN_STATE.with(|slot| *slot.borrow_mut() = Some(state));
        material_theme().child(move || {
            menu_provider()
                .controller(controller)
                .close_on_background(close_on_background)
                .main_content(move || {
                    button()
                        .modifier(Modifier::new().fill_max_size())
                        .on_click(move || state.with_mut(|count| *count += 1));
                })
                .menu_content(|| {
                    column()
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            test_tag: Some("menu_background_content".to_string()),
                            ..Default::default()
                        }))
                        .children(|| {
                            menu_item().label("Edit").enabled(false);
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
            let mut session = start_layout_test_session(
                layout_test(|| {
                    menu_background_fixture(close_on_background);
                })
                .viewport_px(400, 300),
            );
            let _ = pump_layout_test_session(&mut session, 0);
            let controller = MENU_CONTROLLER.with(|slot| slot.borrow().expect("menu controller"));
            controller.with_mut(|menu| {
                menu.open_at(MenuAnchor::at(PxPosition::new(Px(10), Px(10))));
            });
            let opened = pump_layout_test_session(&mut session, 16_666_667);
            let bounds = opened.node("menu_background_content").bounds();
            let now = tessera_ui::time::Instant::now();
            let cursor = Some(if click_inside {
                PxPosition::new(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2)
            } else {
                PxPosition::new(Px(350), Px(250))
            });
            for (index, content) in [
                CursorEventContent::Pressed(PressKeyEventType::Left),
                CursorEventContent::Released(PressKeyEventType::Left),
            ]
            .into_iter()
            .enumerate()
            {
                let change = PointerChange::new(now, 1, content, GestureState::TapCandidate);
                let _ = pump_layout_test_session_with_input(
                    &mut session,
                    33_333_334 + index as u64 * 16_666_667,
                    vec![change],
                    cursor,
                );
            }
            assert_eq!(
                controller.with(MenuController::is_open),
                expected_open,
                "close_on_background={close_on_background}, click_inside={click_inside}"
            );
            assert_eq!(MAIN_STATE.with(|slot| slot.borrow().unwrap().get()), 0);
        }
    }

    #[test]
    fn menu_provider_renders_menu_items_after_controller_opens() {
        let mut session = start_layout_test_session(
            layout_test(|| menu_controller_fixture()).viewport_px(400, 300),
        );
        let _ = pump_layout_test_session(&mut session, 0);

        let controller =
            MENU_CONTROLLER.with(|slot| slot.borrow().expect("menu controller available"));
        controller.with_mut(|menu| {
            menu.open_at(MenuAnchor::at(PxPosition::new(Px(10), Px(10))));
        });

        let opened: LayoutSnapshot = pump_layout_test_session(&mut session, 16_666_667);
        opened.node("menu_content_tag").exists();
    }

    #[test]
    fn menu_visibility_preserves_main_content_state() {
        let mut session = start_layout_test_session(
            layout_test(|| menu_controller_fixture()).viewport_px(400, 300),
        );
        let _ = pump_layout_test_session(&mut session, 0);
        let main_state = MAIN_STATE.with(|slot| slot.borrow().expect("main content state"));
        main_state.set(7);
        let controller = MENU_CONTROLLER.with(|slot| slot.borrow().expect("menu controller"));
        controller.with_mut(|menu| {
            menu.open_at(MenuAnchor::at(PxPosition::new(Px(10), Px(10))));
        });
        let _ = pump_layout_test_session(&mut session, 16_666_667);
        assert!(MAIN_STATE.with(|slot| slot.borrow().unwrap()) == main_state);
        assert_eq!(main_state.get(), 7);

        controller.with_mut(MenuController::close);
        let _ = pump_layout_test_session(&mut session, 33_333_334);
        assert!(MAIN_STATE.with(|slot| slot.borrow().unwrap()) == main_state);
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
