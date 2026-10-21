//! Material Design card primitives.
//!
//! ## Usage
//!
//! Group related content into a single, elevated or outlined container.

use tessera_ui::{
    Callback, Color, Dp, Modifier, RenderSlot, State, accesskit::Role, receive_frame_nanos,
    remember, tessera, use_context,
};

use crate::{
    animation::Animatable,
    modifier::InteractionState,
    shape_def::Shape,
    surface::{SurfaceStyle, surface},
    theme::{ContentColor, MaterialAlpha, MaterialTheme, content_color_for},
};

const DEFAULT_SPATIAL_DAMPING_RATIO: f32 = 0.9;
const DEFAULT_SPATIAL_STIFFNESS: f32 = 700.0;

#[cfg(test)]
mod tests;

fn composite_over(base: Color, overlay: Color) -> Color {
    let overlay_a = overlay.a.clamp(0.0, 1.0);
    let base_a = base.a.clamp(0.0, 1.0);
    let out_a = overlay_a + base_a * (1.0 - overlay_a);
    if out_a <= 0.0 {
        return Color::TRANSPARENT;
    }

    let r = (overlay.r * overlay_a + base.r * base_a * (1.0 - overlay_a)) / out_a;
    let g = (overlay.g * overlay_a + base.g * base_a * (1.0 - overlay_a)) / out_a;
    let b = (overlay.b * overlay_a + base.b * base_a * (1.0 - overlay_a)) / out_a;
    Color::new(r, g, b, out_a)
}

#[derive(Clone, PartialEq, Debug)]
struct CardElevationSpring {
    last_frame_nanos: Option<u64>,
    spring: Animatable,
}

impl CardElevationSpring {
    fn new(initial: Dp) -> Self {
        Self {
            last_frame_nanos: None,
            spring: Animatable::spring(
                initial.0 as f32,
                DEFAULT_SPATIAL_STIFFNESS.sqrt(),
                DEFAULT_SPATIAL_DAMPING_RATIO,
            ),
        }
    }

    fn set_target(&mut self, target: Dp) {
        if self.spring.target() != target.0 as f32 {
            self.last_frame_nanos = None;
        }
        self.spring.set_target(target.0 as f32);
    }

    fn snap_to(&mut self, target: Dp) {
        *self = Self::new(target);
    }

    fn tick(&mut self, frame_nanos: u64) {
        let dt = if let Some(last_frame_nanos) = self.last_frame_nanos {
            frame_nanos.saturating_sub(last_frame_nanos) as f32 / 1_000_000_000.0
        } else {
            1.0 / 60.0
        };
        self.last_frame_nanos = Some(frame_nanos);
        self.spring.tick(dt);
    }

    fn is_animating(&self) -> bool {
        !self.spring.is_idle()
    }

    fn value_dp(&self) -> Dp {
        Dp(self.spring.value() as f64)
    }
}

/// Visual variants supported by [`card`].
#[derive(Clone, PartialEq, Copy, Debug, Default)]
pub enum CardVariant {
    /// Filled cards provide subtle separation from the background.
    #[default]
    Filled,
    /// Elevated cards provide more emphasis via shadow elevation.
    Elevated,
    /// Outlined cards provide emphasis via a border stroke.
    Outlined,
}

/// Represents the container and content colors used in a card in different
/// states.
#[derive(Clone, PartialEq, Copy, Debug)]
pub struct CardColors {
    /// Container color used when enabled.
    pub container_color: Color,
    /// Content color used when enabled.
    pub content_color: Color,
    /// Container color used when disabled.
    pub disabled_container_color: Color,
    /// Content color used when disabled.
    pub disabled_content_color: Color,
}

impl CardColors {
    fn container_color(self, enabled: bool) -> Color {
        if enabled {
            self.container_color
        } else {
            self.disabled_container_color
        }
    }

    fn content_color(self, enabled: bool) -> Color {
        if enabled {
            self.content_color
        } else {
            self.disabled_content_color
        }
    }
}

/// Represents a border stroke for card containers.
#[derive(Clone, PartialEq, Copy, Debug)]
pub struct CardBorder {
    /// Border width.
    pub width: Dp,
    /// Border color.
    pub color: Color,
}

/// Represents the elevation for a card in different states.
#[derive(Clone, PartialEq, Copy, Debug)]
pub struct CardElevation {
    default_elevation: Dp,
    pressed_elevation: Dp,
    focused_elevation: Dp,
    hovered_elevation: Dp,
    dragged_elevation: Dp,
    disabled_elevation: Dp,
}

impl CardElevation {
    fn default_elevation(self) -> Dp {
        self.default_elevation
    }

    fn target(self, enabled: bool, interaction_state: Option<State<InteractionState>>) -> Dp {
        if !enabled {
            return self.disabled_elevation;
        }

        let Some(state) = interaction_state else {
            return self.default_elevation;
        };

        state.with(|s| {
            if s.is_dragged() {
                self.dragged_elevation
            } else if s.is_pressed() {
                self.pressed_elevation
            } else if s.is_focused() {
                self.focused_elevation
            } else if s.is_hovered() {
                self.hovered_elevation
            } else {
                self.default_elevation
            }
        })
    }
}

/// Default values for card components.
pub struct CardDefaults;

impl CardDefaults {
    /// Opacity applied to disabled container overlays.
    pub const DISABLED_CONTAINER_OPACITY: f32 = 0.38;
    /// Opacity applied to disabled content.
    pub const DISABLED_CONTENT_ALPHA: f32 = MaterialAlpha::DISABLED_CONTENT;
    /// Border opacity for disabled outlined cards.
    pub const DISABLED_OUTLINE_ALPHA: f32 = 0.12;

    /// Default filled card shape.
    pub fn shape(theme: &MaterialTheme) -> Shape {
        theme.shapes.medium
    }

    /// Default elevated card shape.
    pub fn elevated_shape(theme: &MaterialTheme) -> Shape {
        theme.shapes.medium
    }

    /// Default outlined card shape.
    pub fn outlined_shape(theme: &MaterialTheme) -> Shape {
        theme.shapes.medium
    }

    /// Default elevation values for filled cards.
    pub fn card_elevation() -> CardElevation {
        CardElevation {
            default_elevation: Dp(0.0),
            pressed_elevation: Dp(0.0),
            focused_elevation: Dp(0.0),
            hovered_elevation: Dp(1.0),
            dragged_elevation: Dp(3.0),
            disabled_elevation: Dp(0.0),
        }
    }

    /// Default elevation values for elevated cards.
    pub fn elevated_card_elevation() -> CardElevation {
        CardElevation {
            default_elevation: Dp(1.0),
            pressed_elevation: Dp(1.0),
            focused_elevation: Dp(1.0),
            hovered_elevation: Dp(2.0),
            dragged_elevation: Dp(4.0),
            disabled_elevation: Dp(1.0),
        }
    }

    /// Default elevation values for outlined cards.
    pub fn outlined_card_elevation() -> CardElevation {
        CardElevation {
            default_elevation: Dp(0.0),
            pressed_elevation: Dp(0.0),
            focused_elevation: Dp(0.0),
            hovered_elevation: Dp(1.0),
            dragged_elevation: Dp(3.0),
            disabled_elevation: Dp(0.0),
        }
    }

    /// Default colors for filled cards.
    pub fn card_colors(theme: &MaterialTheme, content_color: Color) -> CardColors {
        let scheme = &theme.color_scheme;
        let container = scheme.surface_container_highest;
        let content = content_color_for(container, scheme).unwrap_or(content_color);
        let disabled_overlay = scheme
            .surface_variant
            .with_alpha(Self::DISABLED_CONTAINER_OPACITY);
        let disabled_container = composite_over(container, disabled_overlay);
        CardColors {
            container_color: container,
            content_color: content,
            disabled_container_color: disabled_container,
            disabled_content_color: content.with_alpha(Self::DISABLED_CONTENT_ALPHA),
        }
    }

    /// Default colors for elevated cards.
    pub fn elevated_card_colors(theme: &MaterialTheme, content_color: Color) -> CardColors {
        let scheme = &theme.color_scheme;
        let container = scheme.surface_container_low;
        let content = content_color_for(container, scheme).unwrap_or(content_color);
        CardColors {
            container_color: container,
            content_color: content,
            disabled_container_color: scheme.surface,
            disabled_content_color: content.with_alpha(Self::DISABLED_CONTENT_ALPHA),
        }
    }

    /// Default colors for outlined cards.
    pub fn outlined_card_colors(theme: &MaterialTheme, content_color: Color) -> CardColors {
        let scheme = &theme.color_scheme;
        let container = scheme.surface;
        let content = content_color_for(container, scheme).unwrap_or(content_color);
        CardColors {
            container_color: container,
            content_color: content,
            disabled_container_color: container,
            disabled_content_color: content.with_alpha(Self::DISABLED_CONTENT_ALPHA),
        }
    }

    /// Default border stroke for outlined cards.
    pub fn outlined_card_border(theme: &MaterialTheme, enabled: bool) -> CardBorder {
        let scheme = &theme.color_scheme;
        let color = if enabled {
            scheme.outline_variant
        } else {
            composite_over(
                scheme.surface_container_low,
                scheme.outline.with_alpha(Self::DISABLED_OUTLINE_ALPHA),
            )
        };
        CardBorder {
            width: Dp(1.0),
            color,
        }
    }
}

#[derive(Clone)]
struct CardResolvedArgs {
    modifier: Modifier,
    variant: CardVariant,
    enabled: bool,
    on_click: Option<Callback>,
    interaction_state: Option<State<InteractionState>>,
    shape: Option<Shape>,
    colors: Option<CardColors>,
    elevation: Option<CardElevation>,
    border: Option<CardBorder>,
    content: Option<RenderSlot>,
}

/// # card
///
/// Renders a Material card container, optionally clickable and with animated
/// elevation.
///
/// ## Usage
///
/// Group related information and actions into a visually distinct container.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the card subtree.
/// - `variant` — optional card variant controlling default tokens.
/// - `enabled` — optional enabled flag.
/// - `on_click` — optional click callback.
/// - `interaction_state` — optional shared interaction state.
/// - `shape` — optional shape override.
/// - `colors` — optional color override set.
/// - `elevation` — optional elevation override.
/// - `border` — optional border override.
/// - `accessibility_label` — optional accessible name for the card's main
///   action.
/// - `content` — optional content render slot.
///
/// ## Examples
///
/// ```
/// use tessera_components::card::card;
/// use tessera_ui::tessera;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// #[tessera]
/// fn component() {
/// #     material_theme()
/// #         .theme(|| MaterialTheme::default())
/// #         .child(|| {
///     card().filled().content(|| {});
/// #         });
/// }
///
/// component();
/// ```
#[tessera]
pub fn card(
    modifier: Option<Modifier>,
    variant: Option<CardVariant>,
    enabled: Option<bool>,
    on_click: Option<Callback>,
    interaction_state: Option<State<InteractionState>>,
    shape: Option<Shape>,
    colors: Option<CardColors>,
    elevation: Option<CardElevation>,
    border: Option<CardBorder>,
    #[prop(into)] accessibility_label: Option<String>,
    content: Option<RenderSlot>,
) {
    let args = CardResolvedArgs {
        modifier: modifier.unwrap_or_default(),
        variant: variant.unwrap_or_default(),
        enabled: enabled.unwrap_or(true),
        on_click,
        interaction_state,
        shape,
        colors,
        elevation,
        border,
        content,
    };
    let content = args.content.unwrap_or_else(RenderSlot::empty);

    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let content_color = use_context::<ContentColor>()
        .map(|c| c.get().current)
        .unwrap_or(ContentColor::default().current);

    let shape = args.shape.unwrap_or_else(|| match args.variant {
        CardVariant::Filled => CardDefaults::shape(&theme),
        CardVariant::Elevated => CardDefaults::elevated_shape(&theme),
        CardVariant::Outlined => CardDefaults::outlined_shape(&theme),
    });

    let colors = args.colors.unwrap_or_else(|| match args.variant {
        CardVariant::Filled => CardDefaults::card_colors(&theme, content_color),
        CardVariant::Elevated => CardDefaults::elevated_card_colors(&theme, content_color),
        CardVariant::Outlined => CardDefaults::outlined_card_colors(&theme, content_color),
    });

    let elevation = args.elevation.unwrap_or_else(|| match args.variant {
        CardVariant::Filled => CardDefaults::card_elevation(),
        CardVariant::Elevated => CardDefaults::elevated_card_elevation(),
        CardVariant::Outlined => CardDefaults::outlined_card_elevation(),
    });

    let clickable = args.on_click.is_some();
    let interaction_state = if clickable {
        Some(if let Some(state) = args.interaction_state {
            state
        } else {
            remember(InteractionState::new)
        })
    } else {
        None
    };
    if !args.enabled
        && let Some(state) = interaction_state
    {
        if state.with(|state| *state != InteractionState::default()) {
            state.set(InteractionState::default());
        }
    }
    let border = match args.border {
        Some(border) => Some(border),
        None if matches!(args.variant, CardVariant::Outlined) => {
            let mut border = CardDefaults::outlined_card_border(&theme, args.enabled);
            if args.enabled
                && interaction_state.is_some_and(|state| {
                    state.with(|state| {
                        state.is_focused() && !state.is_pressed() && !state.is_dragged()
                    })
                })
            {
                border.color = theme.color_scheme.on_surface;
            }
            Some(border)
        }
        None => None,
    };

    let elevation_spring = remember(|| CardElevationSpring::new(elevation.default_elevation()));

    let enabled = args.enabled;
    let target = elevation.target(enabled, interaction_state);
    let should_update_spring = elevation_spring.with(|spring| {
        let mut next = spring.clone();
        next.set_target(target);
        if !enabled || !clickable {
            next.snap_to(target);
        }
        next != *spring
    });
    if should_update_spring {
        elevation_spring.with_mut(|spring| {
            spring.set_target(target);
            if !enabled || !clickable {
                spring.snap_to(target);
            }
        });
    }

    let should_schedule_frame = elevation_spring.with(|spring| spring.is_animating());

    if should_schedule_frame {
        receive_frame_nanos(move |frame_nanos| {
            let is_animating = elevation_spring.with_mut(|spring| {
                spring.tick(frame_nanos);
                spring.is_animating()
            });
            if is_animating {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }

    let shadow_elevation = if clickable {
        elevation_spring.with(|s| s.value_dp())
    } else {
        elevation.default_elevation()
    };

    let container_color = colors.container_color(args.enabled);
    let content_color = colors.content_color(args.enabled);
    let style = match border {
        Some(border) => SurfaceStyle::FilledOutlined {
            fill_color: container_color,
            border_color: border.color,
            border_width: border.width,
        },
        None => SurfaceStyle::Filled {
            color: container_color,
        },
    };

    let has_on_click = args.on_click.is_some();

    surface()
        .shape(shape)
        .clip_content(true)
        .modifier(args.modifier)
        .content_color(content_color)
        .elevation(shadow_elevation)
        .enabled(args.enabled)
        .style(style)
        .interaction_state_optional(interaction_state)
        .on_click_optional(args.on_click)
        .accessibility_role_optional(has_on_click.then_some(Role::Button))
        .accessibility_label_optional(accessibility_label)
        .accessibility_focusable_optional(has_on_click.then_some(args.enabled))
        .child_shared(content);
}

impl CardBuilder {
    /// Applies the filled card preset.
    pub fn filled(self) -> Self {
        self.variant(CardVariant::Filled)
    }

    /// Applies the elevated card preset.
    pub fn elevated(self) -> Self {
        self.variant(CardVariant::Elevated)
    }

    /// Applies the outlined card preset.
    pub fn outlined(self) -> Self {
        self.variant(CardVariant::Outlined)
    }
}
