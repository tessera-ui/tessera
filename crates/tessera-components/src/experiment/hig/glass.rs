//! Liquid glass material that refracts, blurs, and tones its backdrop.
//!
//! ## Usage
//!
//! Use as a translucent background for floating controls, panels, and toolbars.
use tessera_foundation::modifier::{ClickableArgs, ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Callback, ComputedData, Constraint, Dp, FocusRequester, LayoutResult, MeasurementError,
    Modifier, Px, PxPosition, RenderSlot, accesskit::Role, layout::MeasureScope, remember, tessera,
};

use crate::{
    padding_utils::remove_padding_from_constraint,
    pipelines::liquid_glass::{
        LiquidGlassCommand, LiquidGlassElement, LiquidGlassMaterial, ToneMatrix,
    },
};

/// Selects how the glass filters its backdrop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GlassMaterial {
    /// Frosted glass for controls and surfaces that need stronger separation.
    #[default]
    Regular,
    /// More transparent glass that retains more of the backdrop's color and contrast.
    Clear,
}

/// Selects the glass material's light or dark treatment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GlassAppearance {
    /// A brighter glass surface for use with dark content.
    #[default]
    Light,
    /// A darker glass surface for use with light content.
    Dark,
}

impl GlassMaterial {
    pub(crate) fn optics(self, appearance: GlassAppearance) -> LiquidGlassMaterial {
        const HIGHLIGHT_ANGLE: f32 = std::f32::consts::FRAC_PI_2;
        match self {
            Self::Regular => {
                let dark = appearance == GlassAppearance::Dark;
                LiquidGlassMaterial {
                    margin: 16.0,
                    field_extent: 8.890_358,
                    inner_refraction: [-20.0, 10.0],
                    outer_refraction: [16.0, 16.0],
                    refraction_distance: [-1.0, 0.0],
                    refraction_opacity: 0.6,
                    blur_radius: 5.0,
                    blur_opacity: [0.0, 0.0, 0.5, 1.0],
                    blur_distance: [-20.0, -1.0, 0.0, 0.0],
                    blur_fill_radius: 8.0,
                    blur_fill: if dark {
                        [0.9, 0.0, 0.5]
                    } else {
                        [0.0, 0.9, 0.5]
                    },
                    face: if dark {
                        ToneMatrix {
                            white: 1.125,
                            black: 0.125,
                            saturation: 1.3,
                            fill: [0.0; 4],
                        }
                    } else {
                        ToneMatrix {
                            white: 1.03,
                            black: 0.4,
                            saturation: 1.2,
                            fill: [1.0, 1.0, 1.0, 0.2],
                        }
                    },
                    face_max_luma: if dark { 0.6 } else { 0.94 },
                    face_opacity: 1.0,
                    shadow: ToneMatrix {
                        white: 1.0,
                        black: 0.0,
                        saturation: 1.0,
                        fill: [0.0, 0.0, 0.0, 0.3],
                    },
                    shadow_opacity: 0.04,
                    shadow_radius: 4.0,
                    shadow_offset: 8.0,
                    ring: [8.0, 4.0, 5.0, 0.06, 1.0],
                    highlight: [
                        HIGHLIGHT_ANGLE,
                        1.0,
                        if dark { 1.308_997 } else { 1.850_049 },
                        0.4,
                        -1.0,
                        -0.25,
                    ],
                    clamp: if dark { 1.308_311 } else { 1.069_61 },
                    vibrant: [
                        [1.1202, -0.1894, -0.019, 0.0, 0.1471],
                        [-0.0563, 0.9871, -0.0191, 0.0, 0.1471],
                        [-0.0563, -0.1893, 1.1574, 0.0, 0.1471],
                        [0.0, 0.0, 0.0, 1.0, 0.0],
                    ],
                }
            }
            Self::Clear => LiquidGlassMaterial {
                margin: 1.0,
                field_extent: 2.0,
                inner_refraction: [-26.0, 14.4],
                outer_refraction: [8.0, 5.0],
                refraction_distance: [-1.0, 0.0],
                refraction_opacity: 0.0,
                blur_radius: 0.75,
                blur_opacity: [1.0, 0.5, 0.5, 1.0],
                blur_distance: [-20.0, -1.0, 0.0, 0.0],
                blur_fill_radius: 0.0,
                blur_fill: [0.0; 3],
                face: ToneMatrix {
                    white: 1.08,
                    black: 0.125,
                    saturation: 1.06,
                    fill: [1.0, 1.0, 1.0, 0.0],
                },
                face_max_luma: 1.0,
                face_opacity: 1.0,
                shadow: ToneMatrix {
                    white: 1.0,
                    black: 0.0,
                    saturation: 1.2,
                    fill: [0.0, 0.0, 0.0, 0.1],
                },
                shadow_opacity: 0.0,
                shadow_radius: 0.0,
                shadow_offset: 8.0,
                ring: [0.0; 5],
                highlight: [HIGHLIGHT_ANGLE, 1.0, 1.308_997, 0.4, -1.0, -0.25],
                clamp: 1.191_748,
                vibrant: [
                    [1.3192, -0.0478, -0.0047, 0.0, 0.2],
                    [-0.0142, 1.2856, -0.0048, 0.0, 0.2],
                    [-0.0142, -0.0476, 1.3285, 0.0, 0.2],
                    [0.0, 0.0, 0.0, 1.0, 0.0],
                ],
            },
        }
    }
}

/// # glass
///
/// Renders a liquid glass surface that refracts, blurs, and tones its backdrop.
///
/// ## Usage
///
/// Place floating controls, panels, or toolbars over rich content.
///
/// ## Parameters
///
/// - `material` — backdrop treatment; defaults to [`GlassMaterial::Regular`].
/// - `appearance` — light or dark treatment; defaults to light.
/// - `corner_radius` — corner radius of the continuous-corner shape; defaults to a capsule.
/// - `modifier` — modifier chain applied to the glass.
/// - `padding` — inner padding around the child.
/// - `on_click` — optional click callback.
/// - `block_input` — stops pointer input from reaching content beneath the glass.
/// - `accessibility_role` — optional accessibility role.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
/// - `accessibility_focusable` — makes the glass focusable for assistive technologies.
/// - `child` — content drawn on the glass.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{glass::{GlassMaterial, glass}, text::text};
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// glass().material(GlassMaterial::Clear).child(|| {
///     text("Content on glass");
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn glass(
    material: Option<GlassMaterial>,
    appearance: Option<GlassAppearance>,
    corner_radius: Option<Dp>,
    modifier: Option<Modifier>,
    padding: Option<Dp>,
    on_click: Option<Callback>,
    block_input: Option<bool>,
    accessibility_role: Option<Role>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    accessibility_focusable: Option<bool>,
    child: Option<RenderSlot>,
) {
    let block_input = block_input.unwrap_or(false);
    let accessibility_focusable = accessibility_focusable.unwrap_or(false);
    let focus_requester = remember(FocusRequester::new).get();
    let role = accessibility_role.or_else(|| accessibility_focusable.then_some(Role::Button));
    let mut modifier = modifier.unwrap_or_default();
    if let Some(on_click) = on_click {
        modifier = modifier.clickable(ClickableArgs {
            on_click,
            block_input,
            role,
            label: accessibility_label,
            description: accessibility_description,
            focus_requester: Some(focus_requester),
            ..Default::default()
        });
    } else {
        if block_input {
            modifier = modifier.block_touch_propagation();
        }
        if role.is_some() || accessibility_label.is_some() || accessibility_description.is_some() {
            modifier = modifier.clear_and_set_semantics(SemanticsArgs {
                role,
                label: accessibility_label,
                description: accessibility_description,
                focusable: accessibility_focusable,
                ..Default::default()
            });
        }
    }
    let policy = GlassLayout {
        material: material.unwrap_or_default(),
        appearance: appearance.unwrap_or_default(),
        corner_radius,
        padding: padding.unwrap_or(Dp(0.0)),
    };
    tessera_ui::layout::layout(policy)
        .modifier(modifier)
        .draw(policy)
        .child(move || {
            if let Some(child) = child.as_ref() {
                child.render();
            }
        });
}

#[derive(Clone, Copy, PartialEq)]
struct GlassLayout {
    material: GlassMaterial,
    appearance: GlassAppearance,
    corner_radius: Option<Dp>,
    padding: Dp,
}

impl tessera_ui::LayoutPolicy for GlassLayout {
    fn measure_eq(&self, other: &Self) -> bool {
        self.padding == other.padding
    }

    fn placement_eq(&self, other: &Self) -> bool {
        self.measure_eq(other)
    }

    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let parent = *input.parent_constraint().as_ref();
        let inner = Constraint::new(
            remove_padding_from_constraint(parent.width, self.padding.into()),
            remove_padding_from_constraint(parent.height, self.padding.into()),
        );
        let content = match input.children().first() {
            Some(&child) => {
                let size = child.measure(input, &inner)?.size();
                let inset = self.padding.to_pixels_f32();
                result.place_child(child, PxPosition::from_f32_xy(inset, inset));
                size
            }
            None => ComputedData {
                width: Px(0),
                height: Px(0),
            },
        };
        let padding: Px = self.padding.into();
        Ok(result.with_size(ComputedData {
            width: parent.width.clamp(content.width + padding * 2),
            height: parent.height.clamp(content.height + padding * 2),
        }))
    }
}

impl tessera_ui::DrawModifierNode for GlassLayout {
    fn draw(
        &self,
        ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        let scale = Dp(1.0).to_pixels_f32();
        let Some(size) = ctx.render_input.metadata_mut().computed_data() else {
            content.draw(ctx.render_input);
            return;
        };
        let (width, height) = (size.width.to_f32() / scale, size.height.to_f32() / scale);
        ctx.render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(LiquidGlassCommand {
                material: self.material.optics(self.appearance),
                elements: vec![LiquidGlassElement {
                    x: 0.0,
                    y: 0.0,
                    width,
                    height,
                    radius: self
                        .corner_radius
                        .map_or(width.min(height) * 0.5, |radius| radius.0 as f32),
                }],
                smoothness: 0.0,
                pixels_per_point: scale,
                opacity: 1.0,
            });
        content.draw(ctx.render_input);
    }
}
