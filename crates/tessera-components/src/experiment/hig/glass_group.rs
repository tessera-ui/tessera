//! Fused liquid glass behind a row of controls.
//!
//! ## Usage
//!
//! Group toolbar buttons or segmented actions whose glass shapes blend together when close.
use std::sync::Arc;

use parking_lot::Mutex;
use tessera_ui::{
    AxisConstraint, ComputedData, Constraint, Dp, LayoutResult, MeasurementError, Modifier, Px,
    PxPosition, RenderSlot,
    layout::{MeasureScope, layout},
    remember, tessera,
};

use crate::{
    experiment::hig::glass::{GlassAppearance, GlassMaterial},
    pipelines::liquid_glass::{LiquidGlassCommand, LiquidGlassElement},
};

/// # glass_group
///
/// Lays out its children in a row and renders one liquid glass shape behind each child, fusing
/// shapes that are closer than `spacing`.
///
/// ## Usage
///
/// Group toolbar buttons whose glass should merge as they approach one another.
///
/// ## Parameters
///
/// - `material` — Regular or Clear glass; defaults to [`GlassMaterial::Regular`].
/// - `appearance` — light or dark treatment; defaults to light.
/// - `spacing` — distance below which neighbouring shapes fuse; defaults to zero.
/// - `gap` — space between children; defaults to zero.
/// - `corner_radius` — element corner radius; defaults to a capsule.
/// - `modifier` — modifier chain applied to the group.
/// - `child` — the controls; each direct child gets its own glass shape.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{glass_group::glass_group, text::text};
/// use tessera_ui::Dp;
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// glass_group().spacing(Dp(16.0)).gap(Dp(8.0)).child(|| {
///     text("Back");
///     text("Forward");
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn glass_group(
    material: Option<GlassMaterial>,
    appearance: Option<GlassAppearance>,
    spacing: Option<Dp>,
    gap: Option<Dp>,
    corner_radius: Option<Dp>,
    modifier: Option<Modifier>,
    child: Option<RenderSlot>,
) {
    let cache = remember(|| Arc::new(Mutex::new(Vec::new()))).get();
    let policy = GlassGroupLayout {
        material: material.unwrap_or_default(),
        appearance: appearance.unwrap_or(GlassAppearance::Light),
        spacing: spacing.unwrap_or(Dp(0.0)),
        gap: gap.unwrap_or(Dp(0.0)),
        corner_radius,
        rects: cache,
    };
    layout(policy.clone())
        .modifier(modifier.unwrap_or_default())
        .draw(policy)
        .child(move || {
            if let Some(child) = child.as_ref() {
                child.render();
            }
        });
}

/// Child rectangles in pixels, written during measurement and read when drawing.
type ChildRects = Arc<Mutex<Vec<[f32; 4]>>>;

#[derive(Clone)]
struct GlassGroupLayout {
    material: GlassMaterial,
    appearance: GlassAppearance,
    spacing: Dp,
    gap: Dp,
    corner_radius: Option<Dp>,
    rects: ChildRects,
}

impl PartialEq for GlassGroupLayout {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.rects, &other.rects)
            && self.material == other.material
            && self.appearance == other.appearance
            && self.spacing == other.spacing
            && self.gap == other.gap
            && self.corner_radius == other.corner_radius
    }
}

impl tessera_ui::LayoutPolicy for GlassGroupLayout {
    fn measure_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.rects, &other.rects) && self.gap == other.gap
    }

    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *input.parent_constraint().as_ref();
        let child_constraint = Constraint::new(
            AxisConstraint::new(Px(0), parent.width.resolve_max()),
            AxisConstraint::new(Px(0), parent.height.resolve_max()),
        );
        let gap = self.gap.to_pixels_f32();
        let children = input.children();
        let mut sizes = Vec::with_capacity(children.len());
        for &child in &children {
            sizes.push(child.measure(input, &child_constraint)?.size());
        }
        let height = sizes.iter().map(|size| size.height).max().unwrap_or(Px(0));
        let mut result = LayoutResult::default();
        let mut rects = Vec::with_capacity(children.len());
        let mut x = 0.0;
        for (child, size) in children.into_iter().zip(sizes) {
            let y = (height - size.height).to_f32() * 0.5;
            result.place_child(child, PxPosition::from_f32_xy(x, y));
            rects.push([x, y, size.width.to_f32(), size.height.to_f32()]);
            x += size.width.to_f32() + gap;
        }
        let width = Px((x - gap).max(0.0).ceil() as i32);
        *self.rects.lock() = rects;
        Ok(result.with_size(ComputedData {
            width: parent.width.clamp(width),
            height: parent.height.clamp(height),
        }))
    }
}

impl tessera_ui::DrawModifierNode for GlassGroupLayout {
    fn draw(
        &self,
        ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        if let Some(material) = self.material.liquid(self.appearance) {
            let scale = Dp(1.0).to_pixels_f32();
            let elements = self
                .rects
                .lock()
                .iter()
                .map(|&[x, y, width, height]| {
                    let (width, height) = (width / scale, height / scale);
                    LiquidGlassElement {
                        x: x / scale,
                        y: y / scale,
                        width,
                        height,
                        radius: self
                            .corner_radius
                            .map_or(width.min(height) * 0.5, |radius| radius.0 as f32),
                    }
                })
                .collect();
            ctx.render_input
                .metadata_mut()
                .fragment_mut()
                .push_draw_command(LiquidGlassCommand {
                    material,
                    elements,
                    smoothness: self.spacing.0 as f32,
                    pixels_per_point: scale,
                    opacity: 1.0,
                });
        }
        content.draw(ctx.render_input);
    }
}
