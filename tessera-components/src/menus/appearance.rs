//! Shapes and color styles for expressive vertical menus.
//!
//! ## Usage
//!
//! Organize contextual actions into distinct groups with optional selection.

use tessera_ui::{
    Color, Dp, Modifier, RenderSlot, accesskit::Role, layout::layout, provide_context, tessera,
    use_context,
};

use crate::{
    alignment::CrossAxisAlignment,
    column::column,
    modifier::{ModifierExt as _, Padding},
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceStyle, surface},
    theme::{MaterialColorScheme, MaterialTheme},
};

/// Color emphasis for vertical menus and their items.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuColorStyle {
    /// Surface-based containers with tertiary selection.
    #[default]
    Standard,
    /// Tertiary containers with stronger selected emphasis.
    Vibrant,
}

impl MenuColorStyle {
    pub(super) fn container(self, scheme: &MaterialColorScheme) -> Color {
        match self {
            Self::Standard => scheme.surface_container_low,
            Self::Vibrant => scheme.tertiary_container,
        }
    }

    pub(super) fn selected_container(self, scheme: &MaterialColorScheme) -> Color {
        match self {
            Self::Standard => scheme.tertiary_container,
            Self::Vibrant => scheme.tertiary,
        }
    }

    pub(super) fn content(self, scheme: &MaterialColorScheme, selected: bool) -> Color {
        match (self, selected) {
            (Self::Standard, false) => scheme.on_surface,
            (Self::Standard, true) | (Self::Vibrant, false) => scheme.on_tertiary_container,
            (Self::Vibrant, true) => scheme.on_tertiary,
        }
    }

    pub(super) fn supporting(self, scheme: &MaterialColorScheme, selected: bool) -> Color {
        if self == Self::Standard && !selected {
            scheme.on_surface_variant
        } else {
            self.content(scheme, selected)
        }
    }
}

/// Position-dependent corners for menu groups and items.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuPosition {
    /// A single group or item.
    #[default]
    Standalone,
    /// The first group or item in a sequence.
    First,
    /// A group or item between the first and last.
    Middle,
    /// The final group or item in a sequence.
    Last,
}

/// Default dimensions and shapes for vertical menus.
pub struct MenuDefaults;

impl MenuDefaults {
    /// Gap between separately surfaced groups.
    pub const GROUP_SPACING: Dp = Dp(2.0);
    /// Minimum visible item height, before minimum touch-target expansion.
    pub const ITEM_HEIGHT: Dp = Dp(44.0);

    /// Returns the resting shape for a group at the specified position.
    pub fn group_shape(position: MenuPosition) -> Shape {
        let (top, bottom) = match position {
            MenuPosition::Standalone => (16.0, 16.0),
            MenuPosition::First => (16.0, 8.0),
            MenuPosition::Middle => (8.0, 8.0),
            MenuPosition::Last => (8.0, 16.0),
        };
        corners(top, bottom)
    }

    /// Returns a menu item's shape, including its selected state.
    pub fn item_shape(position: MenuPosition, selected: bool) -> Shape {
        let (top, bottom) = if selected {
            (12.0, 12.0)
        } else {
            match position {
                MenuPosition::First => (12.0, 4.0),
                MenuPosition::Last => (4.0, 12.0),
                MenuPosition::Standalone | MenuPosition::Middle => (4.0, 4.0),
            }
        };
        corners(top, bottom)
    }
}

fn corners(top: f64, bottom: f64) -> Shape {
    Shape::RoundedRectangle {
        top_left: RoundedCorner::manual(Dp(top), 2.0),
        top_right: RoundedCorner::manual(Dp(top), 2.0),
        bottom_right: RoundedCorner::manual(Dp(bottom), 2.0),
        bottom_left: RoundedCorner::manual(Dp(bottom), 2.0),
    }
}

/// # menu_group
///
/// Surface a group of related vertical-menu actions.
///
/// ## Usage
///
/// Separate related actions inside a grouped context menu.
///
/// ## Parameters
///
/// - `position` — outside corners; middle and last groups include a 2dp gap
///   above.
/// - `color_style` — color emphasis; inherits the enclosing menu by default.
/// - `modifier` — optional layout and semantics modifiers.
/// - `shape` — optional fixed shape override.
/// - `container_color` — optional group background override.
/// - `accessibility_label` — optional name describing the group of actions.
/// - `children` — menu items or other group content.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     menus::{MenuController, menu_group, menu_item, menu_provider},
///     theme::material_theme,
/// };
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn component() {
///     let menu = remember(MenuController::new);
///     menu.with_mut(MenuController::open);
///     assert!(menu.with(MenuController::is_open));
///     material_theme().child(move || {
///         menu_provider()
///             .controller(menu)
///             .grouped(true)
///             .menu_content(move || {
///                 menu_group().children(move || {
///                     menu_item().label("Save").on_click(move || {
///                         menu.with_mut(MenuController::close);
///                     });
///                 });
///             });
///     });
/// }
/// component();
/// ```
#[tessera]
pub fn menu_group(
    position: Option<MenuPosition>,
    color_style: Option<MenuColorStyle>,
    modifier: Option<Modifier>,
    shape: Option<Shape>,
    container_color: Option<Color>,
    #[prop(into)] accessibility_label: Option<String>,
    children: Option<RenderSlot>,
) {
    let inherited_style = use_context::<MenuColorStyle>().map(|style| style.get());
    let color_style = color_style.or(inherited_style).unwrap_or_default();
    let position = position.unwrap_or_default();
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let children = children.unwrap_or_else(RenderSlot::empty);
    let modifier = modifier.unwrap_or_default();
    let shape = shape.unwrap_or_else(|| MenuDefaults::group_shape(position));
    layout()
        .modifier(Modifier::new().padding(Padding::top(
            if matches!(position, MenuPosition::Middle | MenuPosition::Last) {
                MenuDefaults::GROUP_SPACING
            } else {
                Dp::ZERO
            },
        )))
        .child(move || {
            surface()
                .modifier(modifier.clone().fill_max_width())
                .shape(shape)
                .style(SurfaceStyle::Filled {
                    color: container_color.unwrap_or_else(|| color_style.container(&scheme)),
                })
                .accessibility_role(Role::Group)
                .accessibility_label_optional(accessibility_label.clone())
                .elevation(Dp(3.0))
                .child(move || {
                    provide_context(
                        || color_style,
                        move || {
                            let inherited = use_context::<MenuColorStyle>().expect("menu colors");
                            if inherited.get() != color_style {
                                inherited.set(color_style);
                            }
                            column()
                                .modifier(
                                    Modifier::new()
                                        .fill_max_width()
                                        .clip_shape(shape)
                                        .padding_symmetric(Dp::ZERO, Dp(2.0)),
                                )
                                .cross_axis_alignment(CrossAxisAlignment::Start)
                                .children_shared(children);
                        },
                    );
                });
        });
}
