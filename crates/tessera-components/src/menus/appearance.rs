//! Shapes and color styles for expressive vertical menus.
//!
//! ## Usage
//!
//! Organize contextual actions into distinct groups with optional selection.

use tessera_foundation::modifier::{ModifierExt as _, Padding};
use tessera_ui::{
    Color, Dp, FocusScopeNode, Modifier, RenderSlot,
    accesskit::Role,
    current_frame_nanos,
    layout::{Stack, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    provide_context, remember, tessera, use_context,
};

use crate::{
    alignment::CrossAxisAlignment,
    column::column,
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceStyle, surface},
    theme::{MaterialColorScheme, MaterialTheme},
};

use super::motion::{MenuShapeHover, MenuShapeMotion, advance};

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
/// - `menu` — action owner received from the enclosing menu content slot.
/// - `position` — outside corners; middle and last groups include a 2dp gap
///   above. Default corners contract when an activated group loses hover and
///   keyboard focus.
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
///             .menu_content(move |scope| {
///                 menu_group().menu(scope).children(move || {
///                     menu_item().menu(scope).label("Save").on_click(move || {
///                         menu.with_mut(MenuController::close);
///                     });
///                 });
///             });
///     });
/// }
/// # tessera_ui::testing::layout_test(component).run();
/// ```
#[tessera]
pub fn menu_group(
    menu: Option<super::MenuScope>,
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
    let focus_scope = remember(FocusScopeNode::new).get();
    let frame = current_frame_nanos();
    let motion = remember(|| MenuShapeMotion::new(position, frame));
    let focused = focus_scope.has_focus();
    if shape.is_none() && motion.with(|motion| motion.needs_update(position, focused)) {
        motion.with_mut(|motion| motion.update(position, focused, frame));
    }
    if shape.is_none() && !motion.with(MenuShapeMotion::is_idle) {
        // Rebuilds this component on every animated frame.
        let frame_clock = tessera_ui::remember(tessera_ui::current_frame_nanos);
        let _ = frame_clock.get();
        tessera_ui::launch_once(async move {
            let step = move |frame: u64| advance(motion, frame);
            loop {
                let now = tessera_ui::next_frame().await;
                frame_clock.set(now);
                if !step(now) {
                    break;
                }
            }
        });
    }
    let mut modifier = modifier.focus_scope(Some(focus_scope));
    if shape.is_none() {
        modifier = modifier.push_pointer_preview_input(MenuShapeHover {
            motion,
            host: menu.map(|menu| menu.host),
            controller: menu.map(|menu| menu.controller),
        });
    }
    layout(Stack)
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
                .shape(shape.unwrap_or_else(|| motion.with(MenuShapeMotion::shape)))
                .style(SurfaceStyle::Filled {
                    color: container_color.unwrap_or_else(|| color_style.container(&scheme)),
                })
                .accessibility_role(Role::Group)
                .accessibility_label_optional(accessibility_label.clone())
                .elevation(Dp(3.0))
                .child(move || {
                    provide_context(color_style, move || {
                        column()
                            .modifier(
                                Modifier::new()
                                    .fill_max_width()
                                    .clip_shape(
                                        shape
                                            .unwrap_or_else(|| motion.with(MenuShapeMotion::shape)),
                                    )
                                    .padding_symmetric(Dp::ZERO, Dp(2.0)),
                            )
                            .cross_axis_alignment(CrossAxisAlignment::Start)
                            .children_shared(children);
                    });
                });
        });
}
