//! Material Design 3 menus for contextual action lists.
//! ## Usage Present anchored overflow or context actions as surfaced menus.
use std::sync::Arc;

use closure::closure;
use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEvent, CursorEventContent, DimensionValue, Dp, Px,
    PxPosition, PxSize, accesskit::Role, tessera, winit,
};

use crate::{
    ShadowProps,
    alignment::CrossAxisAlignment,
    checkmark::{CheckmarkArgsBuilder, checkmark},
    column::{ColumnArgsBuilder, column},
    material_color::{blend_over, global_material_scheme},
    pos_misc::is_position_in_rect,
    row::{RowArgsBuilder, row},
    shape_def::Shape,
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, SurfaceStyle, surface},
    text::{TextArgsBuilder, text},
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

fn default_max_height() -> Option<Px> {
    Some(Px::from(MENU_MAX_HEIGHT))
}

fn default_menu_shape() -> Shape {
    Shape::rounded_rectangle(Dp(4.0))
}

fn default_menu_shadow() -> Option<ShadowProps> {
    let scheme = global_material_scheme();
    Some(ShadowProps {
        color: scheme.shadow.with_alpha(0.12),
        offset: [0.0, 3.0],
        smoothness: 8.0,
    })
}

fn default_menu_color() -> Color {
    global_material_scheme().surface
}

fn default_hover_color() -> Color {
    let scheme = global_material_scheme();
    blend_over(scheme.surface, scheme.on_surface, 0.08)
}

fn default_scrim_color() -> Color {
    Color::new(0.0, 0.0, 0.0, 0.0)
}

/// Scope for adding items inside a [`menu`].
pub struct MenuScope<'a, 'b> {
    scope: &'a mut crate::column::ColumnScope<'b>,
}

impl<'a, 'b> MenuScope<'a, 'b> {
    /// Adds a menu child (typically a [`menu_item`]).
    pub fn item<F>(&mut self, child: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        self.scope.child(child);
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

#[derive(Default)]
struct MenuStateInner {
    is_open: bool,
    anchor: MenuAnchor,
}

/// Shared state for controlling menu visibility and anchor placement.
#[derive(Clone, Default)]
pub struct MenuState {
    inner: Arc<RwLock<MenuStateInner>>,
}

impl MenuState {
    /// Creates a new closed menu state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens the menu using the previously remembered anchor.
    pub fn open(&self) {
        self.inner.write().is_open = true;
    }

    /// Opens the menu at the provided anchor rectangle.
    pub fn open_at(&self, anchor: MenuAnchor) {
        let mut inner = self.inner.write();
        inner.anchor = anchor;
        inner.is_open = true;
    }

    /// Closes the menu.
    pub fn close(&self) {
        self.inner.write().is_open = false;
    }

    /// Toggles the open state, keeping the current anchor.
    pub fn toggle(&self) {
        let mut inner = self.inner.write();
        inner.is_open = !inner.is_open;
    }

    /// Returns whether the menu is currently open.
    pub fn is_open(&self) -> bool {
        self.inner.read().is_open
    }

    fn snapshot(&self) -> (bool, MenuAnchor) {
        let inner = self.inner.read();
        (inner.is_open, inner.anchor)
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
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct MenuProviderArgs {
    /// How the menu is aligned relative to the provided anchor.
    #[builder(default)]
    pub placement: MenuPlacement,
    /// Additional x/y offset applied after placement relative to the anchor.
    #[builder(default = "[Dp(0.0), MENU_VERTICAL_GAP]")]
    pub offset: [Dp; 2],
    /// Width behavior of the menu container. Defaults to the Material 112–280 dp range.
    #[builder(default = "default_menu_width()")]
    pub width: DimensionValue,
    /// Maximum height of the menu before scrolling is required.
    #[builder(default = "default_max_height()")]
    pub max_height: Option<Px>,
    /// Shape of the menu container.
    #[builder(default = "default_menu_shape()")]
    pub shape: Shape,
    /// Optional shadow representing elevation. Defaults to a soft Material shadow.
    #[builder(default = "default_menu_shadow()", setter(strip_option))]
    pub shadow: Option<ShadowProps>,
    /// Background color of the menu container.
    #[builder(default = "default_menu_color()")]
    pub container_color: Color,
    /// Color of the invisible background layer. Defaults to transparent (menus do not dim content).
    #[builder(default = "default_scrim_color()")]
    pub scrim_color: Color,
    /// Whether a background click should dismiss the menu.
    #[builder(default = "true")]
    pub close_on_background: bool,
    /// Whether pressing Escape dismisses the menu.
    #[builder(default = "true")]
    pub close_on_escape: bool,
    /// Optional callback invoked before the menu closes (background or Escape).
    #[builder(default, setter(strip_option))]
    pub on_dismiss: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl Default for MenuProviderArgs {
    fn default() -> Self {
        MenuProviderArgsBuilder::default()
            .build()
            .expect("MenuArgsBuilder default build should succeed")
    }
}

/// Backward compatibility alias for earlier menu args naming.
pub type MenuArgs = MenuProviderArgs;
/// Backward compatibility alias for builder.
pub type MenuArgsBuilder = MenuProviderArgsBuilder;

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

fn extract_available_size(constraint: &Constraint) -> ComputedData {
    let width = match constraint.width {
        DimensionValue::Fixed(px) => px,
        DimensionValue::Wrap { max, .. } | DimensionValue::Fill { max, .. } => {
            max.unwrap_or(Px::MAX)
        }
    };
    let height = match constraint.height {
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

fn apply_close_action(state: &MenuState, on_dismiss: &Option<Arc<dyn Fn() + Send + Sync>>) {
    if let Some(callback) = on_dismiss {
        callback();
    }
    state.close();
}

/// # menu_provider
///
/// Provides a Material Design 3 menu overlay anchored to a rectangle.
///
/// ## Usage
///
/// Wrap page content and show contextual or overflow actions aligned to a trigger element.
///
/// ## Parameters
///
/// - `args` — configures placement, styling, and dismissal behavior; see [`MenuProviderArgs`].
/// - `state` — a clonable [`MenuState`] controlling open/close and anchor position.
/// - `main_content` — closure rendering the underlying page UI.
/// - `menu_content` — closure that receives a [`MenuScope`] to register menu items.
///
/// ## Examples
///
/// ```
/// use std::sync::Arc;
/// use tessera_ui::Dp;
/// use tessera_ui_basic_components::{
///     menus::{
///         menu_item, menu_provider, MenuAnchor, MenuItemArgsBuilder, MenuPlacement,
///         MenuProviderArgsBuilder, MenuScope, MenuState,
///     },
///     text::text,
/// };
///
/// let state = MenuState::new();
/// state.open_at(MenuAnchor::from_dp((Dp(8.0), Dp(24.0)), (Dp(120.0), Dp(36.0))));
/// let state_for_menu = state.clone();
///
/// let args = MenuProviderArgsBuilder::default()
///     .placement(MenuPlacement::BelowStart)
///     .build()
///     .unwrap();
///
/// menu_provider(
///     args,
///     state.clone(),
///     || {
///         text("Main content");
///     },
///     move |menu_scope: &mut MenuScope<'_, '_>| {
///         let menu_state = state_for_menu.clone();
///         menu_scope.item(move || {
///             menu_item(
///                 MenuItemArgsBuilder::default()
///                     .label("Edit")
///                     .on_click(Arc::new(|| {}))
///                     .build()
///                     .unwrap(),
///                 Some(menu_state.clone()),
///             );
///         });
///     },
/// );
///
/// assert!(state.is_open());
/// state.close();
/// assert!(!state.is_open());
/// ```
#[tessera]
pub fn menu_provider(
    args: impl Into<MenuProviderArgs>,
    state: MenuState,
    main_content: impl FnOnce() + Send + Sync + 'static,
    menu_content: impl FnOnce(&mut MenuScope<'_, '_>) + Send + Sync + 'static,
) {
    let args: MenuProviderArgs = args.into();

    // Render underlying content first.
    main_content();

    let (is_open, anchor) = state.snapshot();
    if !is_open {
        return;
    }

    // Track menu bounds for outside-click detection.
    let bounds: Arc<RwLock<Option<MenuBounds>>> = Arc::new(RwLock::new(None));

    // Background layer (non-dimming by default).
    surface(
        SurfaceArgsBuilder::default()
            .style(SurfaceStyle::Filled {
                color: args.scrim_color,
            })
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .block_input(true)
            .build()
            .expect("builder construction failed"),
        || {},
    );

    // Menu panel.
    surface(
        {
            let mut builder = SurfaceArgsBuilder::default()
                .style(SurfaceStyle::Filled {
                    color: args.container_color,
                })
                .shape(args.shape)
                .padding(Dp(0.0))
                .width(args.width)
                .height(DimensionValue::Wrap {
                    min: None,
                    max: args.max_height,
                })
                .accessibility_role(Role::Menu)
                .block_input(true);

            if let Some(shadow) = args.shadow {
                builder = builder.shadow(shadow);
            }

            builder.build().expect("builder construction failed")
        },
        || {
            column(
                ColumnArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .build()
                    .expect("builder construction failed"),
                |scope| {
                    let mut menu_scope = MenuScope { scope };
                    menu_content(&mut menu_scope);
                },
            );
        },
    );

    // Parent input handler: block propagation and close on background click.
    let bounds_for_handler = bounds.clone();
    let on_dismiss_for_handler = args.on_dismiss.clone();
    let close_on_escape = args.close_on_escape;
    let close_on_background = args.close_on_background;
    let state_for_handler = state.clone();
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
            apply_close_action(&state_for_handler, &on_dismiss_for_handler);
        }
    }));

    // Measurement: place main content, background, and menu based on anchor.
    let bounds_for_measure = bounds;
    let args_for_measure = args.clone();
    measure(Box::new(move |input| {
        let main_content_id = input
            .children_ids
            .first()
            .copied()
            .expect("main content should exist");
        let main_size = input.measure_child(main_content_id, input.parent_constraint)?;
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

        let background_size = input.measure_child(background_id, input.parent_constraint)?;
        input.place_child(background_id, PxPosition::new(Px::ZERO, Px::ZERO));

        let menu_size = input.measure_child(menu_id, input.parent_constraint)?;
        let available = if background_size.width > Px::ZERO && background_size.height > Px::ZERO {
            background_size
        } else {
            extract_available_size(input.parent_constraint)
        };
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

        *bounds_for_measure.write() = Some(MenuBounds {
            origin: menu_position,
            size: menu_size,
        });

        Ok(main_size)
    }));
}

/// Convenience wrapper for rendering only the menu overlay without extra main content.
#[tessera]
pub fn menu(
    args: impl Into<MenuProviderArgs>,
    state: MenuState,
    content: impl FnOnce(&mut MenuScope<'_, '_>) + Send + Sync + 'static,
) {
    menu_provider(args, state, || {}, content);
}

/// Defines the configuration for an individual menu item.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct MenuItemArgs {
    /// Primary label text for the item.
    #[builder(setter(into))]
    pub label: String,
    /// Optional supporting text displayed under the label.
    #[builder(default, setter(strip_option, into))]
    pub supporting_text: Option<String>,
    /// Optional trailing text (e.g., keyboard shortcut).
    #[builder(default, setter(strip_option, into))]
    pub trailing_text: Option<String>,
    /// Leading icon displayed when the item is not selected.
    #[builder(default, setter(strip_option))]
    pub leading_icon: Option<crate::icon::IconArgs>,
    /// Trailing icon displayed on the right edge.
    #[builder(default, setter(strip_option))]
    pub trailing_icon: Option<crate::icon::IconArgs>,
    /// Whether the item is currently selected (renders a checkmark instead of a leading icon).
    #[builder(default)]
    pub selected: bool,
    /// Whether the item can be interacted with.
    #[builder(default = "true")]
    pub enabled: bool,
    /// Whether the menu should close after the item is activated.
    #[builder(default = "true")]
    pub close_on_click: bool,
    /// Height of the item row.
    #[builder(default = "MENU_ITEM_HEIGHT")]
    pub height: Dp,
    /// Tint applied to the label text.
    #[builder(default = "crate::material_color::global_material_scheme().on_surface")]
    pub label_color: Color,
    /// Tint applied to supporting or trailing text.
    #[builder(default = "crate::material_color::global_material_scheme().on_surface_variant")]
    pub supporting_color: Color,
    /// Tint applied when the item is disabled.
    #[builder(
        default = "crate::material_color::global_material_scheme().on_surface.with_alpha(0.38)"
    )]
    pub disabled_color: Color,
    /// Callback invoked when the item is activated.
    #[builder(default, setter(strip_option))]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl Default for MenuItemArgs {
    fn default() -> Self {
        MenuItemArgsBuilder::default()
            .label("")
            .build()
            .expect("MenuItemArgsBuilder default build should succeed")
    }
}

fn render_leading(args: &MenuItemArgs, enabled: bool) {
    if args.selected {
        checkmark(
            CheckmarkArgsBuilder::default()
                .color(if enabled {
                    args.label_color
                } else {
                    args.disabled_color
                })
                .size(MENU_LEADING_SIZE)
                .padding([2.0, 2.0])
                .build()
                .expect("builder construction failed"),
        );
    } else if let Some(icon) = args.leading_icon.clone() {
        crate::icon::icon(
            crate::icon::IconArgsBuilder::default()
                .content(icon.content)
                .size(icon.size)
                .width(
                    icon.width
                        .unwrap_or_else(|| DimensionValue::Fixed(Px::from(MENU_LEADING_SIZE))),
                )
                .height(
                    icon.height
                        .unwrap_or_else(|| DimensionValue::Fixed(Px::from(MENU_LEADING_SIZE))),
                )
                .tint(if enabled {
                    args.supporting_color
                } else {
                    args.disabled_color
                })
                .build()
                .expect("builder construction failed"),
        );
    } else {
        spacer(
            SpacerArgsBuilder::default()
                .width(DimensionValue::Fixed(Px::from(MENU_LEADING_SIZE)))
                .height(DimensionValue::Fixed(Px::from(MENU_LEADING_SIZE)))
                .build()
                .expect("builder construction failed"),
        );
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
        ColumnArgsBuilder::default()
            .width(DimensionValue::WRAP)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .build()
            .expect("builder construction failed"),
        |scope| {
            scope.child(move || {
                let text_value = label_text.clone();
                let color = label_color;
                text(
                    TextArgsBuilder::default()
                        .text(text_value)
                        .size(Dp(16.0))
                        .color(color)
                        .build()
                        .expect("builder construction failed"),
                );
            });
            if let Some(supporting) = supporting_text {
                scope.child(move || {
                    let supporting_value = supporting.clone();
                    let color = supporting_color;
                    text(
                        TextArgsBuilder::default()
                            .text(supporting_value)
                            .size(Dp(14.0))
                            .color(color)
                            .build()
                            .expect("builder construction failed"),
                    );
                });
            }
        },
    );
}

fn render_trailing(args: &MenuItemArgs, enabled: bool) {
    if let Some(trailing_icon) = args.trailing_icon.clone() {
        crate::icon::icon(
            crate::icon::IconArgsBuilder::default()
                .content(trailing_icon.content)
                .size(trailing_icon.size)
                .width(trailing_icon.width.unwrap_or(DimensionValue::WRAP))
                .height(trailing_icon.height.unwrap_or(DimensionValue::WRAP))
                .tint(if enabled {
                    args.supporting_color
                } else {
                    args.disabled_color
                })
                .build()
                .expect("builder construction failed"),
        );
    } else if let Some(trailing_text) = args.trailing_text.clone() {
        text(
            TextArgsBuilder::default()
                .text(trailing_text)
                .size(Dp(14.0))
                .color(if enabled {
                    args.supporting_color
                } else {
                    args.disabled_color
                })
                .build()
                .expect("builder construction failed"),
        );
    }
}

/// # menu_item
///
/// Renders a single Material-styled menu item with hover and ripple feedback.
///
/// ## Usage
///
/// Use inside [`menu`] to show actions, shortcuts, or toggles.
///
/// ## Parameters
///
/// - `args` — configures the item label, icons, selection state, and callbacks; see [`MenuItemArgs`].
/// - `menu_state` — optional [`MenuState`] to auto-close the menu when activated.
///
/// ## Examples
///
/// ```
/// use std::sync::Arc;
/// use tessera_ui_basic_components::{
///     menus::{menu_item, MenuItemArgsBuilder, MenuState},
/// };
///
/// let state = MenuState::new();
/// menu_item(
///     MenuItemArgsBuilder::default()
///         .label("Copy")
///         .on_click(Arc::new(|| {}))
///         .build()
///         .unwrap(),
///     Some(state.clone()),
/// );
/// assert!(!state.is_open());
/// ```
#[tessera]
pub fn menu_item(args: MenuItemArgs, menu_state: Option<MenuState>) {
    let is_enabled = args.enabled && args.on_click.is_some();
    let on_click = args.on_click.clone();
    let close_on_click = args.close_on_click;

    let interactive_click = if is_enabled {
        Some(Arc::new(closure!(clone on_click, clone menu_state, || {
            if let Some(handler) = &on_click {
                handler();
            }
            if close_on_click && let Some(state) = &menu_state {
                state.close();
            }
        })) as Arc<dyn Fn() + Send + Sync>)
    } else {
        None
    };

    let mut surface_builder = SurfaceArgsBuilder::default()
        .style(SurfaceStyle::Filled {
            color: Color::TRANSPARENT,
        })
        .hover_style(is_enabled.then(|| SurfaceStyle::Filled {
            color: default_hover_color(),
        }))
        .padding(Dp(0.0))
        .width(DimensionValue::FILLED)
        .height(DimensionValue::Wrap {
            min: Some(Px::from(args.height)),
            max: None,
        })
        .accessibility_role(Role::MenuItem)
        .accessibility_label(args.label.clone())
        .block_input(true)
        .ripple_color(
            crate::material_color::global_material_scheme()
                .on_surface
                .with_alpha(0.12),
        );

    if let Some(click) = interactive_click {
        surface_builder = surface_builder.on_click(click);
    }

    if let Some(description) = args.supporting_text.clone() {
        surface_builder = surface_builder.accessibility_description(description);
    }

    surface(
        surface_builder
            .build()
            .expect("builder construction failed"),
        || {
            row(
                RowArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .height(DimensionValue::Wrap {
                        min: Some(Px::from(args.height)),
                        max: None,
                    })
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .build()
                    .expect("builder construction failed"),
                |row_scope| {
                    // Leading padding
                    row_scope.child(|| {
                        spacer(
                            SpacerArgsBuilder::default()
                                .width(DimensionValue::Fixed(Px::from(MENU_HORIZONTAL_PADDING)))
                                .build()
                                .expect("builder construction failed"),
                        );
                    });

                    // Leading indicator / icon.
                    let leading_args = args.clone();
                    row_scope.child(move || {
                        render_leading(&leading_args, is_enabled);
                    });

                    // Gap after leading.
                    row_scope.child(|| {
                        spacer(
                            SpacerArgsBuilder::default()
                                .width(DimensionValue::Fixed(Px::from(MENU_HORIZONTAL_PADDING)))
                                .build()
                                .expect("builder construction failed"),
                        );
                    });

                    // Labels column.
                    let label_args = args.clone();
                    row_scope.child(move || {
                        render_labels(&label_args, is_enabled);
                    });

                    // Flexible spacer.
                    row_scope.child_weighted(
                        || {
                            spacer(
                                SpacerArgsBuilder::default()
                                    .width(DimensionValue::FILLED)
                                    .build()
                                    .expect("builder construction failed"),
                            );
                        },
                        1.0,
                    );

                    // Trailing text/icon if any.
                    if args.trailing_icon.is_some() || args.trailing_text.is_some() {
                        let trailing_args = args.clone();
                        row_scope.child(move || {
                            render_trailing(&trailing_args, is_enabled);
                        });

                        row_scope.child(|| {
                            spacer(
                                SpacerArgsBuilder::default()
                                    .width(DimensionValue::Fixed(Px::from(MENU_TRAILING_SPACING)))
                                    .build()
                                    .expect("builder construction failed"),
                            );
                        });
                    } else {
                        row_scope.child(|| {
                            spacer(
                                SpacerArgsBuilder::default()
                                    .width(DimensionValue::Fixed(Px::from(MENU_HORIZONTAL_PADDING)))
                                    .build()
                                    .expect("builder construction failed"),
                            );
                        });
                    }
                },
            );
        },
    );
}
