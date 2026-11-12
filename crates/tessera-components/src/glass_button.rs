//! An interactive button with a glassmorphic background.
//!
//! ## Usage
//!
//! Use for visually distinctive actions in layered or modern UIs.
use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Callback, Color, Dp, Modifier, RenderSlot,
    layout::{Stack, layout},
    tessera,
};

use crate::{
    button::ButtonDefaults,
    fluid_glass::{GlassAppearance, GlassBorder, GlassMaterial, fluid_glass},
    shape_def::Shape,
};

impl GlassButtonBuilder {
    /// Applies the primary glass button preset.
    pub fn primary(&self) -> Self {
        self.tint_color(Color::new(0.2, 0.5, 0.8, 0.2))
            .border(GlassBorder::new(Dp(1.0).into()))
    }

    /// Applies the secondary glass button preset.
    pub fn secondary(&self) -> Self {
        self.tint_color(Color::new(0.6, 0.6, 0.6, 0.2))
            .border(GlassBorder::new(Dp(1.0).into()))
    }

    /// Applies the success glass button preset.
    pub fn success(&self) -> Self {
        self.tint_color(Color::new(0.1, 0.7, 0.3, 0.2))
            .border(GlassBorder::new(Dp(1.0).into()))
    }

    /// Applies the danger glass button preset.
    pub fn danger(&self) -> Self {
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
/// - `material` — backdrop treatment, defaults to [`GlassMaterial::Regular`].
/// - `appearance` — optional override for the inherited light or dark appearance.
/// - `interactive` — optional override for optical press feedback.
/// - `blur_radius` — optional blur radius.
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
///         .child(|| {
///             text("Click Me");
///         });
/// }
///
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn glass_button(
    modifier: Option<Modifier>,
    on_click: Option<Callback>,
    padding: Option<Dp>,
    tint_color: Option<Color>,
    shape: Option<Shape>,
    material: Option<GlassMaterial>,
    appearance: Option<GlassAppearance>,
    interactive: Option<bool>,
    blur_radius: Option<Dp>,
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
    let modifier = modifier.unwrap_or_default();
    let padding = padding.unwrap_or(Dp(12.0));
    let noise_amount = noise_amount.unwrap_or(0.0);
    let noise_scale = noise_scale.unwrap_or(1.0);
    let time = time.unwrap_or(0.0);
    let accessibility_focusable = accessibility_focusable.unwrap_or(false);
    let child = child.unwrap_or_else(RenderSlot::empty);
    let outer_modifier =
        modifier.default_min_size(ButtonDefaults::MIN_WIDTH, ButtonDefaults::MIN_HEIGHT);

    layout(Stack).modifier(outer_modifier).child(move || {
        fluid_glass()
            .modifier(Modifier::new())
            .tint_color_optional(tint_color)
            .shape_optional(shape)
            .material_optional(material)
            .appearance_optional(appearance)
            .interactive_optional(interactive)
            .blur_radius_optional(blur_radius)
            .noise_amount(noise_amount)
            .noise_scale(noise_scale)
            .time(time)
            .padding(padding)
            .contrast_optional(contrast)
            .on_click_optional(on_click)
            .border_optional(border)
            .accessibility_label_optional(accessibility_label.clone())
            .accessibility_description_optional(accessibility_description.clone())
            .accessibility_focusable_optional(accessibility_focusable.then_some(true))
            .child_shared(child);
    });
}
