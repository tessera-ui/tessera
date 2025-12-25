//! Material Design 3 menus for contextual action lists.
//!
//! ## Usage
//!
//! Present anchored overflow or context actions as surfaced menus.
use std::sync::Arc;

use derive_setters::Setters;
use parking_lot::RwLock;
use tessera_ui::{
    Color, ComputedData, CursorEvent, CursorEventContent, DimensionValue, Dp, Modifier,
    ParentConstraint, Px, PxPosition, PxSize, State, accesskit::Role, remember, tessera,
    use_context, winit,
};

use crate::{
    alignment::CrossAxisAlignment,
    checkmark::{CheckmarkArgs, checkmark},
    column::{ColumnArgs, column},
    modifier::ModifierExt as _,
    pos_misc::is_position_in_rect,
    row::{RowArgs, row},
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceArgs, SurfaceStyle, surface},
    text::{TextArgs, text},
    theme::{MaterialAlpha, MaterialTheme},
};

const MENU_MIN_WIDTH: Dp = Dp(112.0);
const MENU_MAX_WIDTH: Dp = Dp(280.0);
const MENU_MAX_HEIGHT: Dp = Dp(320.0);
const MENU_VERTICAL_GAP: Dp = Dp(4.0);
const MENU_HORIZONTAL_PADDING: Dp = Dp(16.0);
const MENU_LEADING_SIZE: Dp = Dp(20.0);
const MENU_ITEM_HEIGHT: Dp = Dp(48.0);
const MENU_TRAILING_SPACING: Dp = Dp(16.0);

fn default_menu_width() -> DimensionValue {
    DimensionValue::Wrap {
        min: Some(Px::from(MENU_MIN_WIDTH)),
        max: Some(Px::from(MENU_MAX_WIDTH)),
    }
}

fn default_menu_modifier() -> Modifier {
    Modifier::new().constrain(Some(default_menu_width()), None)
}

fn default_max_height() -> Option<Px> {
    Some(Px::from(MENU_MAX_HEIGHT))
}

fn default_menu_shape() -> Shape {
    Shape::rounded_rectangle(Dp(4.0))
}

fn default_menu_color() -> Color {
    use_context::<MaterialTheme>().get().color_scheme.surface
}

fn default_scrim_color() -> Color {
    Color::new(0.0, 0.0, 0.0, 0.0)
}

/// Scope for adding items inside a [`menu`].
pub struct MenuScope<'a, 'b> {
    scope: &'a mut crate::column::ColumnScope<'b>,
    controller: State<MenuController>,
}

impl<'a, 'b> MenuScope<'a, 'b> {
    /// Adds a menu child (typically a menu item).
    pub fn item<F>(&mut self, child: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        self.scope.child(child);
    }

    /// Adds a menu item to the menu.
    pub fn menu_item(&mut self, args: impl Into<MenuItemArgs>) {
        let mut args = args.into();
        if args.close_on_click {
            let prev = args.on_click;
            let controller = self.controller;
            args.on_click = Some(Arc::new(move || {
                if let Some(f) = &prev {
                    f();
                }
                controller.with_mut(|c| c.close());
            }));
        }
        self.scope.child(move || {
            menu_item(args);
        });
    }
}

/// Describes the anchor rectangle used to position a menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Default, Clone)]
pub struct MenuController {
    is_open: bool,
    anchor: Option<MenuAnchor>,
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
    }

    /// Opens the menu at the provided anchor rectangle.
    pub fn open_at(&mut self, anchor: MenuAnchor) {
        self.anchor = Some(anchor);
        self.is_open = true;
    }

    /// Closes the menu.
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Toggles the open state, keeping the current anchor.
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
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
}

/// Configuration for the menu overlay/provider.
#[derive(Clone, Setters)]
pub struct MenuProviderArgs {
    /// How the menu is aligned relative to the provided anchor.
    pub placement: MenuPlacement,
    /// Additional x/y offset applied after placement relative to the anchor.
    pub offset: [Dp; 2],
    /// Layout modifiers applied to the menu container. Defaults to the Material
    /// 112–280 dp width range.
    pub modifier: Modifier,
    /// Maximum height of the menu before scrolling is required.
    #[setters(strip_option)]
    pub max_height: Option<Px>,
    /// Shape of the menu container.
    pub shape: Shape,
    /// Elevation of the menu. Defaults to level 2 (3.0.dp).
    pub elevation: Dp,
    /// Background color of the menu container.
    pub container_color: Color,
    /// Color of the invisible background layer. Defaults to transparent (menus
    /// do not dim content).
    pub scrim_color: Color,
    /// Whether a background click should dismiss the menu.
    pub close_on_background: bool,
    /// Whether pressing Escape dismisses the menu.
    pub close_on_escape: bool,
    /// Optional callback invoked before the menu closes (background or Escape).
    #[setters(skip)]
    pub on_dismiss: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Whether the menu is currently open.
    pub is_open: bool,
}

impl MenuProviderArgs {
    /// Set the dismiss callback.
    pub fn on_dismiss<F>(mut self, on_dismiss: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_dismiss = Some(Arc::new(on_dismiss));
        self
    }

    /// Set the dismiss callback using a shared callback.
    pub fn on_dismiss_shared(mut self, on_dismiss: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.on_dismiss = Some(on_dismiss);
        self
    }
}

impl Default for MenuProviderArgs {
    fn default() -> Self {
        Self {
            placement: MenuPlacement::default(),
            offset: [Dp(0.0), MENU_VERTICAL_GAP],
            modifier: default_menu_modifier(),
            max_height: default_max_height(),
            shape: default_menu_shape(),
            elevation: Dp(3.0),
            container_color: default_menu_color(),
            scrim_color: default_scrim_color(),
            close_on_background: true,
            close_on_escape: true,
            on_dismiss: None,
            is_open: false,
        }
    }
}

/// Backward compatibility alias for earlier menu args naming.
pub type MenuArgs = MenuProviderArgs;

#[derive(Clone, Copy)]
struct MenuBounds {
    origin: PxPosition,
    size: ComputedData,
}

impl Default for MenuBounds {
    fn default() -> Self {
        Self {
            origin: PxPosition::new(Px::ZERO, Px::ZERO),
            size: ComputedData::ZERO,
        }
    }
}

fn resolve_menu_position(
    anchor: MenuAnchor,
    placement: MenuPlacement,
    menu_size: ComputedData,
    available: ComputedData,
    offset: [Dp; 2],
) -> PxPosition {
    let anchor_end_x = anchor.origin.x + anchor.size.width;
    let anchor_end_y = anchor.origin.y + anchor.size.height;

    let mut x = match placement {
        MenuPlacement::BelowStart | MenuPlacement::AboveStart => anchor.origin.x,
        MenuPlacement::BelowEnd | MenuPlacement::AboveEnd => anchor_end_x - menu_size.width,
    };

    let mut y = match placement {
        MenuPlacement::BelowStart | MenuPlacement::BelowEnd => anchor_end_y,
        MenuPlacement::AboveStart | MenuPlacement::AboveEnd => anchor.origin.y - menu_size.height,
    };

    x += Px::from(offset[0]);
    y += Px::from(offset[1]);

    let max_x = available.width - menu_size.width;
    let max_y = available.height - menu_size.height;
    if x < Px::ZERO {
        x = Px::ZERO;
    }
    if y < Px::ZERO {
        y = Px::ZERO;
    }
    if max_x > Px::ZERO {
        x = x.min(max_x);
    }
    if max_y > Px::ZERO {
        y = y.min(max_y);
    }

    PxPosition::new(x, y)
}

fn extract_available_size(constraint: ParentConstraint<'_>) -> ComputedData {
    let width = match constraint.width() {
        DimensionValue::Fixed(px) => px,
        DimensionValue::Wrap { max, .. } | DimensionValue::Fill { max, .. } => {
            max.unwrap_or(Px::MAX)
        }
    };
    let height = match constraint.height() {
        DimensionValue::Fixed(px) => px,
        DimensionValue::Wrap { max, .. } | DimensionValue::Fill { max, .. } => {
            max.unwrap_or(Px::MAX)
        }
    };

    ComputedData { width, height }
}

fn should_close_on_click(
    cursor_events: &[CursorEvent],
    cursor_position: Option<PxPosition>,
    bounds: Option<MenuBounds>,
) -> bool {
    let Some(bounds) = bounds else {
        return false;
    };

    cursor_events.iter().any(|event| {
        matches!(event.content, CursorEventContent::Released(_))
            && cursor_position
                .map(|pos| {
                    !is_position_in_rect(pos, bounds.origin, bounds.size.width, bounds.size.height)
                })
                .unwrap_or(false)
    })
}

fn apply_close_action(
    controller: State<MenuController>,
    on_dismiss: &Option<Arc<dyn Fn() + Send + Sync>>,
) {
    if let Some(callback) = on_dismiss {
        callback();
    }
    controller.with_mut(|c| c.close());
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
/// - `args` — configures placement, styling, and dismissal behavior; see
///   [`MenuProviderArgs`].
/// - `main_content` — closure rendering the underlying page UI.
/// - `menu_content` — closure that receives a [`MenuScope`] to register menu
///   items.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_ui::Dp;
/// use tessera_ui_basic_components::{
///     menus::{
///         MenuAnchor, MenuItemArgs, MenuPlacement, MenuProviderArgs, MenuScope, menu_provider,
///     },
///     text::text,
/// };
///
/// let args = MenuProviderArgs::default()
///     .placement(MenuPlacement::BelowStart)
///     .is_open(true);
///
/// menu_provider(
///     args,
///     || {
///         text("Main content");
///     },
///     move |menu_scope: &mut MenuScope<'_, '_>| {
///         menu_scope.menu_item(MenuItemArgs::default().label("Edit").on_click(|| {}));
///     },
/// );
/// # }
/// # component();
/// ```
#[tessera]
pub fn menu_provider(
    args: impl Into<MenuProviderArgs>,
    main_content: impl FnOnce() + Send + Sync + 'static,
    menu_content: impl FnOnce(&mut MenuScope<'_, '_>) + Send + Sync + 'static,
) {
    let args: MenuProviderArgs = args.into();
    let controller = remember(MenuController::new);

    if controller.with(|c| c.is_open()) != args.is_open {
        if args.is_open {
            controller.with_mut(|c| c.open());
        } else {
            controller.with_mut(|c| c.close());
        }
    }

    menu_provider_with_controller(args, controller, main_content, menu_content);
}

/// # menu_provider_with_controller
///
/// Provides a Material Design 3 menu overlay anchored to a rectangle with an
/// external controller.
///
/// ## Usage
///
/// Wrap page content and show contextual or overflow actions aligned to a
/// trigger element, controlled via a shared [`MenuController`].
///
/// ## Parameters
///
/// - `args` — configures placement, styling, and dismissal behavior; see
///   [`MenuProviderArgs`].
/// - `controller` — A [`MenuController`] controlling open/close and anchor
///   position.
/// - `main_content` — closure rendering the underlying page UI.
/// - `menu_content` — closure that receives a [`MenuScope`] to register menu
///   items.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_ui::{Dp, remember, tessera};
/// use tessera_ui_basic_components::{
///     menus::{
///         MenuAnchor, MenuController, MenuItemArgs, MenuPlacement, MenuProviderArgs, MenuScope,
///         menu_provider_with_controller,
///     },
///     text::text,
/// };
///
/// #[tessera]
/// fn foo() {
///     let menu_controller = remember(MenuController::new);
///     let args = MenuProviderArgs::default().placement(MenuPlacement::BelowStart);
///     menu_provider_with_controller(
///         args,
///         menu_controller,
///         || { /* Main content */ },
///         move |menu_scope| {
///             menu_scope.menu_item(MenuItemArgs::default().label("Edit").on_click(|| {
///                 // Handle edit action
///             }));
///         },
///     );
/// }
/// # }
/// # component();
/// ```
#[tessera]
pub fn menu_provider_with_controller(
    args: impl Into<MenuProviderArgs>,
    controller: State<MenuController>,
    main_content: impl FnOnce() + Send + Sync + 'static,
    menu_content: impl FnOnce(&mut MenuScope<'_, '_>) + Send + Sync + 'static,
) {
    let args: MenuProviderArgs = args.into();

    // Render underlying content first.
    main_content();

    let (is_open, anchor) = controller.with(|c| c.snapshot());
    if !is_open {
        return;
    }

    // Track menu bounds for outside-click detection.
    let bounds: Arc<RwLock<Option<MenuBounds>>> = Arc::new(RwLock::new(None));

    // Background layer (non-dimming by default).
    surface(
        SurfaceArgs::default()
            .style(SurfaceStyle::Filled {
                color: args.scrim_color,
            })
            .modifier(Modifier::new().fill_max_size())
            .block_input(true),
        || {},
    );

    // Menu panel.
    surface(
        {
            SurfaceArgs::default()
                .style(SurfaceStyle::Filled {
                    color: args.container_color,
                })
                .shape(args.shape)
                .modifier(args.modifier.constrain(
                    None,
                    Some(DimensionValue::Wrap {
                        min: None,
                        max: args.max_height,
                    }),
                ))
                .accessibility_role(Role::Menu)
                .block_input(true)
                .elevation(args.elevation)
        },
        move || {
            column(
                ColumnArgs::default()
                    .modifier(Modifier::new().fill_max_width())
                    .cross_axis_alignment(CrossAxisAlignment::Start),
                {
                    move |scope| {
                        let mut menu_scope = MenuScope { scope, controller };
                        menu_content(&mut menu_scope);
                    }
                },
            );
        },
    );

    // Parent input handler: block propagation and close on background click.
    let bounds_for_handler = bounds.clone();
    let on_dismiss_for_handler = args.on_dismiss.clone();
    let close_on_escape = args.close_on_escape;
    let close_on_background = args.close_on_background;
    input_handler(Box::new(move |mut input| {
        let mut cursor_events: Vec<_> = Vec::new();
        std::mem::swap(&mut cursor_events, input.cursor_events);
        let cursor_position = input.cursor_position_rel;

        let mut keyboard_events: Vec<_> = Vec::new();
        std::mem::swap(&mut keyboard_events, input.keyboard_events);

        // Prevent underlying content from receiving input while menu is open.
        input.block_all();

        let menu_bounds = *bounds_for_handler.read();
        let should_close_click = close_on_background
            && should_close_on_click(&cursor_events, cursor_position, menu_bounds);

        let should_close_escape = close_on_escape
            && keyboard_events.iter().any(|event| {
                event.state == winit::event::ElementState::Pressed
                    && matches!(
                        event.physical_key,
                        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape)
                    )
            });

        if should_close_click || should_close_escape {
            apply_close_action(controller, &on_dismiss_for_handler);
        }
    }));

    // Measurement: place main content, background, and menu based on anchor.
    let args_for_measure = args.clone();
    measure(Box::new(move |input| {
        let main_content_id = input
            .children_ids
            .first()
            .copied()
            .expect("main content should exist");
        let main_size = input.measure_child_in_parent_constraint(main_content_id)?;
        input.place_child(main_content_id, PxPosition::new(Px::ZERO, Px::ZERO));

        let background_id = input
            .children_ids
            .get(1)
            .copied()
            .expect("menu background should exist");
        let menu_id = input
            .children_ids
            .get(2)
            .copied()
            .expect("menu surface should exist");

        let background_size = input.measure_child_in_parent_constraint(background_id)?;
        input.place_child(background_id, PxPosition::new(Px::ZERO, Px::ZERO));

        let menu_size = input.measure_child_in_parent_constraint(menu_id)?;
        let available = if background_size.width > Px::ZERO && background_size.height > Px::ZERO {
            background_size
        } else {
            extract_available_size(input.parent_constraint)
        };
        let anchor = anchor.unwrap_or_else(|| {
            MenuAnchor::new(
                PxPosition::ZERO,
                PxSize::new(main_size.width, main_size.height),
            )
        });
        let menu_position = resolve_menu_position(
            anchor,
            args_for_measure.placement,
            menu_size,
            available,
            args_for_measure.offset,
        );
        input.place_child(menu_id, menu_position);

        if let Some(mut metadata) = input.metadatas.get_mut(&menu_id) {
            metadata.clips_children = true;
        }

        *bounds.write() = Some(MenuBounds {
            origin: menu_position,
            size: menu_size,
        });

        Ok(main_size)
    }));
}

/// Convenience wrapper for rendering only the menu overlay without extra main
/// content.
#[tessera]
pub fn menu(
    args: impl Into<MenuProviderArgs>,
    content: impl FnOnce(&mut MenuScope<'_, '_>) + Send + Sync + 'static,
) {
    menu_provider(args, || {}, content);
}

/// Convenience wrapper for rendering only the menu overlay without extra main
/// content with an external controller.
#[tessera]
pub fn menu_with_controller(
    args: impl Into<MenuProviderArgs>,
    controller: State<MenuController>,
    content: impl FnOnce(&mut MenuScope<'_, '_>) + Send + Sync + 'static,
) {
    menu_provider_with_controller(args, controller, || {}, content);
}

/// Defines the configuration for an individual menu item.
#[derive(Clone, Setters)]
pub struct MenuItemArgs {
    /// Primary label text for the item.
    #[setters(into)]
    pub label: String,
    /// Optional supporting text displayed under the label.
    #[setters(strip_option, into)]
    pub supporting_text: Option<String>,
    /// Optional trailing text (e.g., keyboard shortcut).
    #[setters(strip_option, into)]
    pub trailing_text: Option<String>,
    /// Leading icon displayed when the item is not selected.
    #[setters(strip_option, into)]
    pub leading_icon: Option<crate::icon::IconArgs>,
    /// Trailing icon displayed on the right edge.
    #[setters(strip_option, into)]
    pub trailing_icon: Option<crate::icon::IconArgs>,
    /// Whether the item is currently selected (renders a checkmark instead of a
    /// leading icon).
    pub selected: bool,
    /// Whether the item can be interacted with.
    pub enabled: bool,
    /// Whether the menu should close after the item is activated.
    pub close_on_click: bool,
    /// Height of the item row.
    pub height: Dp,
    /// Tint applied to the label text.
    pub label_color: Color,
    /// Tint applied to supporting or trailing text.
    pub supporting_color: Color,
    /// Tint applied when the item is disabled.
    pub disabled_color: Color,
    /// Callback invoked when the item is activated.
    #[setters(skip)]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl MenuItemArgs {
    /// Creates menu item arguments with the required label.
    pub fn new(label: impl Into<String>) -> Self {
        let scheme = use_context::<MaterialTheme>().get().color_scheme;
        Self {
            label: label.into(),
            supporting_text: None,
            trailing_text: None,
            leading_icon: None,
            trailing_icon: None,
            selected: false,
            enabled: true,
            close_on_click: true,
            height: MENU_ITEM_HEIGHT,
            label_color: scheme.on_surface,
            supporting_color: scheme.on_surface_variant,
            disabled_color: scheme
                .on_surface
                .with_alpha(MaterialAlpha::DISABLED_CONTENT),
            on_click: None,
        }
    }

    /// Set the click handler.
    pub fn on_click<F>(mut self, on_click: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Arc::new(on_click));
        self
    }

    /// Set the click handler using a shared callback.
    pub fn on_click_shared(mut self, on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.on_click = Some(on_click);
        self
    }
}

impl Default for MenuItemArgs {
    fn default() -> Self {
        Self::new("")
    }
}

fn render_leading(args: &MenuItemArgs, enabled: bool) {
    if args.selected {
        checkmark(
            CheckmarkArgs::default()
                .color(if enabled {
                    args.label_color
                } else {
                    args.disabled_color
                })
                .size(MENU_LEADING_SIZE)
                .padding([2.0, 2.0]),
        );
    } else if let Some(icon) = args.leading_icon.clone() {
        let width = icon
            .width
            .unwrap_or_else(|| DimensionValue::Fixed(Px::from(MENU_LEADING_SIZE)));
        let height = icon
            .height
            .unwrap_or_else(|| DimensionValue::Fixed(Px::from(MENU_LEADING_SIZE)));
        crate::icon::icon(icon.width(width).height(height).tint(if enabled {
            args.supporting_color
        } else {
            args.disabled_color
        }));
    } else {
        spacer(Modifier::new().size(MENU_LEADING_SIZE, MENU_LEADING_SIZE));
    }
}

fn render_labels(args: &MenuItemArgs, enabled: bool) {
    let label_color = if enabled {
        args.label_color
    } else {
        args.disabled_color
    };
    let supporting_color = if enabled {
        args.supporting_color
    } else {
        args.disabled_color
    };
    let label_text = args.label.clone();
    let supporting_text = args.supporting_text.clone();

    column(
        ColumnArgs::default()
            .modifier(Modifier::new().constrain(Some(DimensionValue::WRAP), None))
            .cross_axis_alignment(CrossAxisAlignment::Start),
        |scope| {
            scope.child(move || {
                let text_value = label_text.clone();
                let color = label_color;
                text(
                    TextArgs::default()
                        .text(text_value)
                        .size(Dp(16.0))
                        .color(color),
                );
            });
            if let Some(supporting) = supporting_text {
                scope.child(move || {
                    let supporting_value = supporting.clone();
                    let color = supporting_color;
                    text(
                        TextArgs::default()
                            .text(supporting_value)
                            .size(Dp(14.0))
                            .color(color),
                    );
                });
            }
        },
    );
}

fn render_trailing(args: &MenuItemArgs, enabled: bool) {
    if let Some(trailing_icon) = args.trailing_icon.clone() {
        let width = trailing_icon.width.unwrap_or(DimensionValue::WRAP);
        let height = trailing_icon.height.unwrap_or(DimensionValue::WRAP);
        crate::icon::icon(trailing_icon.width(width).height(height).tint(if enabled {
            args.supporting_color
        } else {
            args.disabled_color
        }));
    } else if let Some(trailing_text) = args.trailing_text.clone() {
        text(
            TextArgs::default()
                .text(trailing_text)
                .size(Dp(14.0))
                .color(if enabled {
                    args.supporting_color
                } else {
                    args.disabled_color
                }),
        );
    }
}

#[tessera]
fn menu_item(args: impl Into<MenuItemArgs>) {
    let args: MenuItemArgs = args.into();
    let enabled = args.enabled && args.on_click.is_some();

    let mut surface_args = SurfaceArgs::default()
        .style(SurfaceStyle::Filled {
            color: Color::TRANSPARENT,
        })
        .enabled(enabled)
        .modifier(Modifier::new().constrain(
            Some(DimensionValue::FILLED),
            Some(DimensionValue::Wrap {
                min: Some(Px::from(args.height)),
                max: None,
            }),
        ))
        .accessibility_role(Role::MenuItem)
        .accessibility_label(args.label.clone())
        .block_input(true)
        .ripple_color(
            use_context::<MaterialTheme>()
                .get()
                .color_scheme
                .on_surface
                .with_alpha(1.0),
        );

    if let Some(on_click) = args.on_click.clone() {
        surface_args = surface_args.on_click_shared(on_click);
    }

    if let Some(description) = args.supporting_text.clone() {
        surface_args = surface_args.accessibility_description(description);
    }

    surface(surface_args, move || {
        row(
            RowArgs::default()
                .modifier(Modifier::new().constrain(
                    Some(DimensionValue::FILLED),
                    Some(DimensionValue::Wrap {
                        min: Some(Px::from(args.height)),
                        max: None,
                    }),
                ))
                .cross_axis_alignment(CrossAxisAlignment::Center),
            |row_scope| {
                // Leading padding
                row_scope.child(|| {
                    spacer(Modifier::new().constrain(
                        Some(DimensionValue::Fixed(Px::from(MENU_HORIZONTAL_PADDING))),
                        None,
                    ));
                });

                // Leading indicator / icon.
                let leading_args = args.clone();
                row_scope.child(move || {
                    render_leading(&leading_args, enabled);
                });

                // Gap after leading.
                row_scope.child(|| {
                    spacer(Modifier::new().constrain(
                        Some(DimensionValue::Fixed(Px::from(MENU_HORIZONTAL_PADDING))),
                        None,
                    ));
                });

                // Labels column.
                let label_args = args.clone();
                row_scope.child(move || {
                    render_labels(&label_args, enabled);
                });

                // Flexible spacer.
                row_scope.child_weighted(
                    || {
                        spacer(Modifier::new().fill_max_width());
                    },
                    1.0,
                );

                // Trailing text/icon if any.
                if args.trailing_icon.is_some() || args.trailing_text.is_some() {
                    let trailing_args = args.clone();
                    row_scope.child(move || {
                        render_trailing(&trailing_args, enabled);
                    });

                    row_scope.child(|| {
                        spacer(Modifier::new().constrain(
                            Some(DimensionValue::Fixed(Px::from(MENU_TRAILING_SPACING))),
                            None,
                        ));
                    });
                } else {
                    row_scope.child(|| {
                        spacer(Modifier::new().constrain(
                            Some(DimensionValue::Fixed(Px::from(MENU_HORIZONTAL_PADDING))),
                            None,
                        ));
                    });
                }
            },
        );
    });
}
