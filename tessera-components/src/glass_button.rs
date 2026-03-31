//! An interactive button with a glassmorphic background.
//!
//! ## Usage
//!
//! Use for visually distinctive actions in layered or modern UIs.
use tessera_ui::{Callback, Color, Dp, Modifier, RenderSlot, layout::layout_primitive, tessera};

use crate::{
    button::ButtonDefaults,
    fluid_glass::{GlassBorder, fluid_glass},
    modifier::ModifierExt as _,
    shape_def::{RoundedCorner, Shape},
};

#[derive(Clone)]
struct GlassButtonResolvedArgs {
    modifier: Modifier,
    on_click: Option<Callback>,
    padding: Dp,
    tint_color: Color,
    shape: Shape,
    blur_radius: Dp,
    dispersion_height: Dp,
    chroma_multiplier: f32,
    refraction_height: Dp,
    refraction_amount: f32,
    noise_amount: f32,
    noise_scale: f32,
    time: f32,
    contrast: Option<f32>,
    border: Option<GlassBorder>,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
    accessibility_focusable: bool,
    child: Option<RenderSlot>,
}

impl GlassButtonBuilder {
    /// Sets the child content slot.
    pub fn with_child(self, child: impl Fn() + Send + Sync + 'static) -> Self {
        self.child(child)
    }

    /// Applies the primary glass button preset.
    pub fn primary(self) -> Self {
        self.tint_color(Color::new(0.2, 0.5, 0.8, 0.2))
            .border(GlassBorder::new(Dp(1.0).into()))
    }

    /// Applies the secondary glass button preset.
    pub fn secondary(self) -> Self {
        self.tint_color(Color::new(0.6, 0.6, 0.6, 0.2))
            .border(GlassBorder::new(Dp(1.0).into()))
    }

    /// Applies the success glass button preset.
    pub fn success(self) -> Self {
        self.tint_color(Color::new(0.1, 0.7, 0.3, 0.2))
            .border(GlassBorder::new(Dp(1.0).into()))
    }

    /// Applies the danger glass button preset.
    pub fn danger(self) -> Self {
        self.tint_color(Color::new(0.8, 0.2, 0.2, 0.2))
            .border(GlassBorder::new(Dp(1.0).into()))
    }
}

/// # glass_button
///
/// Renders an interactive button with a customizable glass effect and ripple
/// animation.
///
/// ## Usage
///
/// Use as a primary action button where a modern, layered look is desired.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the button subtree.
/// - `on_click` — optional click callback.
/// - `padding` — optional inner padding.
/// - `tint_color` — optional glass tint color.
/// - `shape` — optional shape override.
/// - `blur_radius` — optional blur radius.
/// - `dispersion_height` — optional chromatic dispersion height.
/// - `chroma_multiplier` — optional chromatic multiplier.
/// - `refraction_height` — optional refraction height.
/// - `refraction_amount` — optional refraction amount.
/// - `noise_amount` — optional noise amount.
/// - `noise_scale` — optional noise scale.
/// - `time` — optional animated time input.
/// - `contrast` — optional contrast override.
/// - `border` — optional glass border override.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
/// - `accessibility_focusable` — optional accessibility focusable flag.
/// - `child` — optional child render slot.
///
/// ## Examples
///
/// ```
/// use tessera_components::{glass_button::glass_button, text::text};
/// use tessera_ui::tessera;
///
/// #[tessera]
/// fn component() {
///     glass_button()
///         .primary()
///         .on_click(|| println!("Button clicked!"))
///         .with_child(|| {
///             text().content("Click Me");
///         });
/// }
///
/// component();
/// ```
#[tessera]
pub fn glass_button(
    modifier: Modifier,
    on_click: Option<Callback>,
    padding: Option<Dp>,
    tint_color: Option<Color>,
    shape: Option<Shape>,
    blur_radius: Option<Dp>,
    dispersion_height: Option<Dp>,
    chroma_multiplier: Option<f32>,
    refraction_height: Option<Dp>,
    refraction_amount: Option<f32>,
    noise_amount: Option<f32>,
    noise_scale: Option<f32>,
    time: Option<f32>,
    contrast: Option<f32>,
    border: Option<GlassBorder>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    accessibility_focusable: Option<bool>,
    child: Option<RenderSlot>,
) {
    let button_args = GlassButtonResolvedArgs {
        modifier,
        on_click,
        padding: padding.unwrap_or(Dp(12.0)),
        tint_color: tint_color.unwrap_or(Color::new(0.5, 0.5, 0.5, 0.1)),
        shape: shape.unwrap_or(Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(Dp(25.0), 3.0),
            top_right: RoundedCorner::manual(Dp(25.0), 3.0),
            bottom_right: RoundedCorner::manual(Dp(25.0), 3.0),
            bottom_left: RoundedCorner::manual(Dp(25.0), 3.0),
        }),
        blur_radius: blur_radius.unwrap_or(Dp(0.0)),
        dispersion_height: dispersion_height.unwrap_or(Dp(25.0)),
        chroma_multiplier: chroma_multiplier.unwrap_or(1.1),
        refraction_height: refraction_height.unwrap_or(Dp(24.0)),
        refraction_amount: refraction_amount.unwrap_or(32.0),
        noise_amount: noise_amount.unwrap_or(0.0),
        noise_scale: noise_scale.unwrap_or(1.0),
        time: time.unwrap_or(0.0),
        contrast,
        border,
        accessibility_label,
        accessibility_description,
        accessibility_focusable: accessibility_focusable.unwrap_or(false),
        child,
    };

    let child = button_args.child.unwrap_or_else(RenderSlot::empty);
    let outer_modifier = button_args.modifier.clone().size_in(
        Some(ButtonDefaults::MIN_WIDTH),
        None,
        Some(ButtonDefaults::MIN_HEIGHT),
        None,
    );

    layout_primitive().modifier(outer_modifier).child(move || {
        let mut builder = fluid_glass()
            .modifier(Modifier::new())
            .tint_color(button_args.tint_color)
            .shape(button_args.shape)
            .blur_radius(button_args.blur_radius)
            .dispersion_height(button_args.dispersion_height)
            .chroma_multiplier(button_args.chroma_multiplier)
            .refraction_height(button_args.refraction_height)
            .refraction_amount(button_args.refraction_amount)
            .noise_amount(button_args.noise_amount)
            .noise_scale(button_args.noise_scale)
            .time(button_args.time)
            .padding(button_args.padding)
            .child_shared(child);

        if let Some(contrast) = button_args.contrast {
            builder = builder.contrast(contrast);
        }
        if let Some(on_click) = button_args.on_click {
            builder = builder.on_click_shared(on_click);
        }
        if let Some(border) = button_args.border {
            builder = builder.border(border);
        }
        if let Some(label) = button_args.accessibility_label.clone() {
            builder = builder.accessibility_label(label);
        }
        if let Some(description) = button_args.accessibility_description.clone() {
            builder = builder.accessibility_description(description);
        }
        if button_args.accessibility_focusable {
            builder = builder.accessibility_focusable(true);
        }

        drop(builder);
    });
}
