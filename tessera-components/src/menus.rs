//! Material Design 3 menus for contextual action lists.
//!
//! ## Usage
//!
//! Present anchored overflow or context actions as surfaced menus.
use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui::{
    Callback, Color, ComputedData, DimensionValue, Dp, FocusRequester, FocusScopeNode,
    FocusTraversalPolicy, MeasurementError, Modifier, ParentConstraint, Px, PxPosition, PxSize,
    RenderSlot, State,
    accesskit::Role,
    layout::{LayoutInput, LayoutOutput, LayoutPolicy, layout_primitive},
    modifier::FocusModifierExt as _,
    provide_context, remember, tessera, use_context, winit,
};

use crate::{
    alignment::CrossAxisAlignment,
    checkmark::checkmark,
    column::column,
    icon::{IconContent, icon},
    modifier::{ModifierExt as _, with_keyboard_input, with_pointer_input},
    pos_misc::is_position_in_rect,
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    text::text,
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
    use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme
        .surface
}

fn default_scrim_color() -> Color {
    Color::new(0.0, 0.0, 0.0, 0.0)
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
#[derive(Default, Clone, PartialEq)]
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
    /// Align to the anchor's end edge and expand rightward.
    RightStart,
    /// Align to the anchor's start edge and expand leftward.
    LeftStart,
}

#[derive(Clone, PartialEq)]
struct MenuProviderConfig {
    pub placement: MenuPlacement,
    pub offset: [Dp; 2],
    pub modifier: Modifier,
    pub max_height: Option<Px>,
    pub shape: Shape,
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
    placement: MenuPlacement,
    offset: [Dp; 2],
    anchor: Option<MenuAnchor>,
    bounds: Arc<RwLock<Option<MenuBounds>>>,
}

impl PartialEq for MenuLayout {
    fn eq(&self, other: &Self) -> bool {
        self.placement == other.placement
            && self.offset == other.offset
            && self.anchor == other.anchor
    }
}

impl LayoutPolicy for MenuLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let main_content_id = input
            .children_ids()
            .first()
            .copied()
            .expect("main content should exist");
        let main_size = input.measure_child_in_parent_constraint(main_content_id)?;
        output.place_child(main_content_id, PxPosition::new(Px::ZERO, Px::ZERO));

        let background_id = input
            .children_ids()
            .get(1)
            .copied()
            .expect("menu background should exist");
        let menu_id = input
            .children_ids()
            .get(2)
            .copied()
            .expect("menu surface should exist");

        let background_size = input.measure_child_in_parent_constraint(background_id)?;
        output.place_child(background_id, PxPosition::new(Px::ZERO, Px::ZERO));

        let menu_size = input.measure_child_in_parent_constraint(menu_id)?;
        let available = if background_size.width > Px::ZERO && background_size.height > Px::ZERO {
            background_size
        } else {
            extract_available_size(input.parent_constraint())
        };
        let anchor = self.anchor.unwrap_or_else(|| {
            MenuAnchor::new(
                PxPosition::ZERO,
                PxSize::new(main_size.width, main_size.height),
            )
        });
        let menu_position =
            resolve_menu_position(anchor, self.placement, menu_size, available, self.offset);
        output.place_child(menu_id, menu_position);

        *self.bounds.write() = Some(MenuBounds {
            origin: menu_position,
            size: menu_size,
        });

        Ok(main_size)
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
        MenuPlacement::RightStart => anchor_end_x,
        MenuPlacement::LeftStart => anchor.origin.x - menu_size.width,
    };

    let mut y = match placement {
        MenuPlacement::BelowStart | MenuPlacement::BelowEnd => anchor_end_y,
        MenuPlacement::AboveStart | MenuPlacement::AboveEnd => anchor.origin.y - menu_size.height,
        MenuPlacement::RightStart | MenuPlacement::LeftStart => anchor.origin.y,
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
/// - `placement` — how the menu aligns relative to its anchor
/// - `offset` — additional x/y offset after placement
/// - `modifier` — extra layout modifiers for the menu container
/// - `max_height` — optional maximum menu height before scrolling
/// - `shape` — shape of the surfaced menu panel
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
///     menus::{MenuPlacement, menu_item, menu_provider},
///     text::text,
/// };
/// use tessera_ui::{Callback, Dp};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// menu_provider()
///     .placement(MenuPlacement::BelowStart)
///     .is_open(true)
///     .main_content(|| {
///         text().content("Main content");
///     })
///     .menu_content(|| {
///         menu_item().label("Edit").on_click(Callback::noop());
///     });
/// # });
/// # }
/// # component();
/// ```
#[tessera]
pub fn menu_provider(
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
    is_open: bool,
    controller: Option<State<MenuController>>,
    focus_restorer_fallback: Option<FocusRequester>,
    main_content: Option<RenderSlot>,
    menu_content: Option<RenderSlot>,
) {
    let provider_args = MenuProviderConfig {
        placement: placement.unwrap_or_default(),
        offset: offset.unwrap_or([Dp(0.0), MENU_VERTICAL_GAP]),
        modifier: modifier.unwrap_or_else(default_menu_modifier),
        max_height: max_height.or_else(default_max_height),
        shape: shape.unwrap_or_else(default_menu_shape),
        elevation: elevation.unwrap_or(Dp(3.0)),
        container_color: container_color.unwrap_or_else(default_menu_color),
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
    let controller = provider_args
        .controller
        .unwrap_or_else(|| remember(MenuController::new));

    if controller.with(|c| c.is_open()) != provider_args.is_open {
        if provider_args.is_open {
            controller.with_mut(|c| c.open());
        } else {
            controller.with_mut(|c| c.close());
        }
    }

    let main_content = provider_args.main_content.unwrap_or_else(RenderSlot::empty);
    let menu_content = provider_args.menu_content.unwrap_or_else(RenderSlot::empty);
    let menu_open_state = remember(|| false);

    let (is_open, anchor) = controller.with(|c| c.snapshot());
    let mut just_opened = false;
    menu_open_state.with_mut(|was_open| {
        just_opened = !*was_open && is_open;
        *was_open = is_open;
    });
    if !is_open {
        return;
    }

    // Track menu bounds for outside-click detection.
    let bounds: Arc<RwLock<Option<MenuBounds>>> = Arc::new(RwLock::new(None));

    // Parent pointer handler: block propagation and close on background click.
    let bounds = bounds.clone();
    let on_dismiss = provider_args.on_dismiss;
    let close_on_escape = provider_args.close_on_escape;
    let close_on_background = provider_args.close_on_background;
    let mut modifier = with_pointer_input(Modifier::new(), {
        let bounds = bounds.clone();
        move |mut input| {
            let cursor_position = input.cursor_position_rel;
            let menu_bounds = *bounds.read();
            let should_close_click = close_on_background
                && input.has_unconsumed_release()
                && is_click_outside_menu(cursor_position, menu_bounds);

            // Prevent underlying content from receiving input while menu is open.
            input.block_all();

            if should_close_click {
                apply_close_action(controller, &on_dismiss);
            }
        }
    });

    if close_on_escape {
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

    // Measurement: place main content, background, and menu based on anchor.
    layout_primitive()
        .modifier(modifier)
        .layout_policy(MenuLayout {
            placement: provider_args.placement,
            offset: provider_args.offset,
            anchor,
            bounds,
        })
        .child(move || {
            let menu_content = menu_content;
            main_content.render();

            surface()
                .style(SurfaceStyle::Filled {
                    color: provider_args.scrim_color,
                })
                .modifier(Modifier::new().fill_max_size())
                .block_input(true)
                .with_child(|| {});

            menu_panel()
                .provider(provider_args.clone())
                .controller(controller)
                .menu_content_shared(menu_content)
                .just_opened(just_opened);
        });
}

#[tessera]
fn menu_panel(
    provider: MenuProviderConfig,
    controller: Option<State<MenuController>>,
    menu_content: Option<RenderSlot>,
    just_opened: bool,
) {
    let controller = controller.expect("menu_panel requires controller");
    let menu_content = menu_content.expect("menu_panel requires menu content");
    let focus_scope = remember(FocusScopeNode::new).get();
    let on_dismiss = provider.on_dismiss;
    let placement = provider.placement;

    let modifier = with_keyboard_input(
        Modifier::new()
            .focus_restorer_with(focus_scope, provider.focus_restorer_fallback)
            .focus_traversal_policy(
                FocusTraversalPolicy::vertical()
                    .wrap(true)
                    .tab_navigation(true),
            ),
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
        focus_scope.restore_focus();
    }

    layout_primitive().modifier(modifier).child(move || {
        let menu_content = menu_content;
        surface()
            .style(SurfaceStyle::Filled {
                color: provider.container_color,
            })
            .shape(provider.shape)
            .modifier(
                provider
                    .modifier
                    .clone()
                    .constrain(
                        None,
                        Some(DimensionValue::Wrap {
                            min: None,
                            max: provider.max_height,
                        }),
                    )
                    .clip_to_bounds(),
            )
            .accessibility_role(Role::Menu)
            .block_input(true)
            .elevation(provider.elevation)
            .with_child(move || {
                let menu_content = menu_content;
                provide_context(
                    || controller,
                    move || {
                        column()
                            .modifier(Modifier::new().fill_max_width())
                            .cross_axis_alignment(CrossAxisAlignment::Start)
                            .children(move || {
                                menu_content.render();
                            });
                    },
                );
            });
    });
}

#[derive(Clone, PartialEq)]
struct MenuItemConfig {
    pub label: String,
    pub supporting_text: Option<String>,
    pub trailing_text: Option<String>,
    pub leading_icon: Option<IconContent>,
    pub trailing_icon: Option<IconContent>,
    pub submenu_content: Option<RenderSlot>,
    pub submenu_placement: MenuPlacement,
    pub selected: bool,
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
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        Self {
            label: label.into(),
            supporting_text: None,
            trailing_text: None,
            leading_icon: None,
            trailing_icon: None,
            submenu_content: None,
            submenu_placement: MenuPlacement::RightStart,
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
/// - `submenu_content` — optional submenu content opened from this item
/// - `submenu_placement` — optional submenu placement override
/// - `selected` — whether the item renders as selected
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
///
/// menu_provider()
///     .is_open(true)
///     .menu_content(|| {
///         menu_item().label("Open");
///     })
///     .main_content(|| {
///         text().content("Main content");
///     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn menu_item(
    #[prop(into)] label: String,
    #[prop(into)] supporting_text: Option<String>,
    #[prop(into)] trailing_text: Option<String>,
    #[prop(into)] leading_icon: Option<IconContent>,
    #[prop(into)] trailing_icon: Option<IconContent>,
    submenu_content: Option<RenderSlot>,
    submenu_placement: Option<MenuPlacement>,
    selected: bool,
    enabled: Option<bool>,
    close_on_click: Option<bool>,
    height: Option<Dp>,
    label_color: Option<Color>,
    supporting_color: Option<Color>,
    disabled_color: Option<Color>,
    on_click: Option<Callback>,
) {
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
        enabled: enabled.unwrap_or(true),
        close_on_click: close_on_click.unwrap_or(true),
        height: height.unwrap_or(MENU_ITEM_HEIGHT),
        label_color: label_color.unwrap_or(scheme.on_surface),
        supporting_color: supporting_color.unwrap_or(scheme.on_surface_variant),
        disabled_color: disabled_color.unwrap_or(
            scheme
                .on_surface
                .with_alpha(MaterialAlpha::DISABLED_CONTENT),
        ),
        on_click,
    };

    if args.close_on_click
        && args.submenu_content.is_none()
        && let Some(controller_context) = use_context::<State<MenuController>>()
    {
        let controller = controller_context.get();
        let previous = args.on_click;
        args.on_click = Some(Callback::new(move || {
            if let Some(callback) = previous {
                callback.call();
            }
            controller.with_mut(|menu| menu.close());
        }));
    }

    menu_item_inner().args(args);
}

fn render_leading(args: &MenuItemConfig, enabled: bool) {
    if args.selected {
        checkmark()
            .color(if enabled {
                args.label_color
            } else {
                args.disabled_color
            })
            .size(MENU_LEADING_SIZE)
            .padding([2.0, 2.0]);
    } else if let Some(icon) = args.leading_icon.clone() {
        render_menu_icon(
            icon,
            if enabled {
                args.supporting_color
            } else {
                args.disabled_color
            },
        );
    } else {
        spacer().modifier(Modifier::new().size(MENU_LEADING_SIZE, MENU_LEADING_SIZE));
    }
}

fn render_labels(args: &MenuItemConfig, enabled: bool) {
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

    column()
        .modifier(Modifier::new().constrain(Some(DimensionValue::WRAP), None))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .children(move || {
            {
                let text_value = label_text.clone();
                let color = label_color;
                text().content(text_value).size(Dp(16.0)).color(color);
            };
            if let Some(supporting) = supporting_text.as_ref() {
                {
                    let supporting_value = supporting.clone();
                    let color = supporting_color;
                    text().content(supporting_value).size(Dp(14.0)).color(color);
                };
            }
        });
}

fn render_trailing(args: &MenuItemConfig, enabled: bool) {
    if let Some(trailing_icon) = args.trailing_icon.clone() {
        render_menu_icon(
            trailing_icon,
            if enabled {
                args.supporting_color
            } else {
                args.disabled_color
            },
        );
    } else if let Some(trailing_text) = args.trailing_text.clone() {
        text()
            .content(trailing_text)
            .size(Dp(14.0))
            .color(if enabled {
                args.supporting_color
            } else {
                args.disabled_color
            });
    } else if args.submenu_content.is_some() {
        text().content(">").size(Dp(14.0)).color(if enabled {
            args.supporting_color
        } else {
            args.disabled_color
        });
    }
}

fn render_menu_icon(content: IconContent, tint: Color) {
    match content {
        IconContent::Vector(data) => {
            icon().vector(data).size(MENU_LEADING_SIZE).tint(tint);
        }
        IconContent::Raster(data) => {
            icon().raster(data).size(MENU_LEADING_SIZE).tint(tint);
        }
    }
}

#[tessera]
fn menu_item_surface(
    item: MenuItemConfig,
    submenu_controller: Option<State<MenuController>>,
    focus_requester: Option<FocusRequester>,
) {
    let enabled = item.enabled && (item.on_click.is_some() || item.submenu_content.is_some());
    let has_submenu = item.submenu_content.is_some();
    let keyboard_submenu_controller = submenu_controller;
    let click_submenu_controller = submenu_controller;
    let modifier = if let Some(submenu_controller) = keyboard_submenu_controller {
        with_keyboard_input(Modifier::new(), move |mut input| {
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
                    winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight) => {
                        submenu_controller.with_mut(|controller| controller.open());
                        true
                    }
                    winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft)
                        if submenu_controller.with(|controller| controller.is_open()) =>
                    {
                        submenu_controller.with_mut(|controller| controller.close());
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
        })
    } else {
        Modifier::new()
    };

    layout_primitive().modifier(modifier).child(move || {
        let mut surface_args = surface()
            .style(SurfaceStyle::Filled {
                color: Color::TRANSPARENT,
            })
            .enabled(enabled)
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::FILLED),
                Some(DimensionValue::Wrap {
                    min: Some(Px::from(item.height)),
                    max: None,
                }),
            ))
            .accessibility_role(Role::MenuItem)
            .accessibility_label(item.label.clone())
            .block_input(true)
            .ripple_color(
                use_context::<MaterialTheme>()
                    .expect("MaterialTheme must be provided")
                    .get()
                    .color_scheme
                    .on_surface
                    .with_alpha(1.0),
            );

        if let Some(focus_requester) = focus_requester {
            surface_args = surface_args.focus_requester(focus_requester);
        }

        if has_submenu {
            let submenu_controller =
                click_submenu_controller.expect("submenu item requires controller");
            surface_args = surface_args.on_click_shared(move || {
                submenu_controller.with_mut(|controller| controller.open());
            });
        } else if let Some(on_click) = item.on_click {
            surface_args = surface_args.on_click_shared(on_click);
        }

        if let Some(description) = item.supporting_text.clone() {
            surface_args = surface_args.accessibility_description(description);
        }

        let item_for_child = item.clone();
        surface_args.with_child(move || {
            let item_for_row = item_for_child.clone();
            row()
                .modifier(Modifier::new().constrain(
                    Some(DimensionValue::FILLED),
                    Some(DimensionValue::Wrap {
                        min: Some(Px::from(item_for_child.height)),
                        max: None,
                    }),
                ))
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    let item_for_child = item_for_row.clone();
                    {
                        spacer().modifier(Modifier::new().constrain(
                            Some(DimensionValue::Fixed(Px::from(MENU_HORIZONTAL_PADDING))),
                            None,
                        ));
                    };

                    let leading_args = item_for_child.clone();
                    {
                        render_leading(&leading_args, enabled);
                    };

                    {
                        spacer().modifier(Modifier::new().constrain(
                            Some(DimensionValue::Fixed(Px::from(MENU_HORIZONTAL_PADDING))),
                            None,
                        ));
                    };

                    let label_args = item_for_child.clone();
                    {
                        render_labels(&label_args, enabled);
                    };

                    spacer().modifier(Modifier::new().fill_max_width().weight(1.0));

                    if item_for_child.trailing_icon.is_some()
                        || item_for_child.trailing_text.is_some()
                        || item_for_child.submenu_content.is_some()
                    {
                        let trailing_args = item_for_child.clone();
                        {
                            render_trailing(&trailing_args, enabled);
                        };

                        {
                            spacer().modifier(Modifier::new().constrain(
                                Some(DimensionValue::Fixed(Px::from(MENU_TRAILING_SPACING))),
                                None,
                            ));
                        };
                    } else {
                        {
                            spacer().modifier(Modifier::new().constrain(
                                Some(DimensionValue::Fixed(Px::from(MENU_HORIZONTAL_PADDING))),
                                None,
                            ));
                        };
                    }
                });
        });
    });
}

#[tessera]
fn menu_item_inner(args: MenuItemConfig) {
    let submenu_content = args.submenu_content;

    if let Some(submenu_content) = submenu_content {
        let submenu_controller = remember(MenuController::new);
        let focus_requester = remember(FocusRequester::new).get();
        menu_provider()
            .placement(args.submenu_placement)
            .offset([Dp::ZERO, Dp::ZERO])
            .controller(submenu_controller)
            .focus_restorer_fallback(focus_requester)
            .main_content({
                let args = args.clone();
                move || {
                    menu_item_surface()
                        .item(args.clone())
                        .submenu_controller(submenu_controller)
                        .focus_requester(focus_requester);
                }
            })
            .menu_content_shared(submenu_content);
    } else {
        menu_item_surface().item(args);
    }
}
