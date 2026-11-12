//! Extended floating action buttons with animated labels.
//!
//! ## Usage
//!
//! Emphasize a primary action with an icon and a descriptive label.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    AxisConstraint, Callback, Color, ComputedData, Constraint, Dp, LayoutResult, MeasurementError,
    Modifier, Px, PxPosition, launch, layout::MeasureScope, remember, tessera, time::Instant,
    use_context,
};

use crate::{
    animation::{Animatable, FrameDelta, Motion, animate_to},
    floating_action_button::{
        FloatingActionButtonElevation, FloatingActionButtonSize, floating_action_button,
    },
    painter::Painter,
    shape_def::Shape,
    text::text,
    theme::{MaterialTheme, TextStyle},
};

/// Expressive size of an extended floating action button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExtendedFloatingActionButtonSize {
    /// A 56dp container with a 24dp icon and title-medium label.
    #[default]
    Small,
    /// An 80dp container with a 28dp icon and title-large label.
    Medium,
    /// A 96dp container with a 36dp icon and headline-small label.
    Large,
}

impl ExtendedFloatingActionButtonSize {
    /// Returns the container height and collapsed width.
    pub const fn height(self) -> Dp {
        match self {
            Self::Small => Dp(56.0),
            Self::Medium => Dp(80.0),
            Self::Large => Dp(96.0),
        }
    }

    /// Returns the icon size.
    pub const fn icon_size(self) -> Dp {
        match self {
            Self::Small => Dp(24.0),
            Self::Medium => Dp(28.0),
            Self::Large => Dp(36.0),
        }
    }

    fn padding(self) -> Dp {
        match self {
            Self::Small => Dp(16.0),
            Self::Medium => Dp(26.0),
            Self::Large => Dp(28.0),
        }
    }

    fn gap(self) -> Dp {
        match self {
            Self::Small => Dp(8.0),
            Self::Medium => Dp(12.0),
            Self::Large => Dp(16.0),
        }
    }

    fn base_size(self) -> FloatingActionButtonSize {
        match self {
            Self::Small => FloatingActionButtonSize::Standard,
            Self::Medium => FloatingActionButtonSize::Medium,
            Self::Large => FloatingActionButtonSize::Large,
        }
    }

    fn text_style(self, theme: &MaterialTheme) -> TextStyle {
        match self {
            Self::Small => theme.typography.title_medium,
            Self::Medium => theme.typography.title_large,
            Self::Large => theme.typography.headline_small,
        }
    }
}

struct ExpansionMotion {
    width: Animatable,
    opacity: Animatable,
    expanded: bool,
    delta: FrameDelta,
}

impl ExpansionMotion {
    fn new(expanded: bool) -> Self {
        let value = if expanded { 1.0 } else { 0.0 };
        Self {
            width: Animatable::spring(value, 800.0_f32.sqrt(), 0.6),
            opacity: Animatable::spring(value, 3800.0_f32.sqrt(), 1.0),
            expanded,
            delta: FrameDelta::default(),
        }
    }

    fn is_idle(&self) -> bool {
        self.width.is_idle() && self.opacity.is_idle()
    }
}

impl Motion for ExpansionMotion {
    type Target = bool;

    fn is_settled_at(&self, expanded: &bool) -> bool {
        self.expanded == *expanded && self.is_idle()
    }

    fn retarget(&mut self, expanded: bool, now: Instant) {
        self.advance(now);
        self.expanded = expanded;
        let target = if expanded { 1.0 } else { 0.0 };
        self.width.set_target(target);
        self.opacity.set_target(target);
    }

    fn advance(&mut self, now: Instant) -> bool {
        let dt = self.delta.step(now);
        self.width.tick(dt);
        self.opacity.tick(dt);
        !self.is_idle()
    }
}

#[derive(Clone, PartialEq)]
struct ExtendedFabContent {
    label: String,
    icon: Option<Painter>,
    size: ExtendedFloatingActionButtonSize,
    style: TextStyle,
}

#[derive(Clone, Copy, PartialEq)]
struct ExtendedFabLayout {
    size: ExtendedFloatingActionButtonSize,
    has_icon: bool,
    progress: f32,
}

impl tessera_ui::LayoutPolicy for ExtendedFabLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let children = input.children();
        let height = input
            .parent_constraint()
            .height()
            .clamp(self.size.height().to_px());
        let padding = self.size.padding().to_px();
        let icon_size = self.size.icon_size().to_px();
        let gap = if self.has_icon {
            self.size.gap().to_px()
        } else {
            Px::ZERO
        };
        let icon_width = if self.has_icon { icon_size } else { Px::ZERO };
        let label_constraint = Constraint::new(
            AxisConstraint::new(
                Px::ZERO,
                input
                    .parent_constraint()
                    .width()
                    .resolve_max()
                    .map(|max| (max - padding * 2 - icon_width - gap).max(Px::ZERO)),
            ),
            AxisConstraint::new(Px::ZERO, Some(height)),
        );
        let label = children[usize::from(self.has_icon)];
        let label_size = label.measure(input, &label_constraint)?;
        let expanded_width =
            (padding * 2 + icon_width + gap + label_size.width).max(self.size.height().to_px());
        let collapsed_width = self.size.height().to_px();
        let width = input.parent_constraint().width().clamp(Px::new(
            (collapsed_width.to_f32()
                + (expanded_width - collapsed_width).to_f32() * self.progress.max(0.0))
            .round() as i32,
        ));
        let mut result = LayoutResult::new(ComputedData { width, height });
        if self.has_icon {
            let child = children[0];
            child.measure(
                input,
                &Constraint::new(
                    AxisConstraint::exact(icon_size),
                    AxisConstraint::exact(icon_size),
                ),
            )?;
            let collapsed_start = (collapsed_width - icon_size).to_f32() / 2.0;
            let icon_x = Px::new(
                (collapsed_start + (padding.to_f32() - collapsed_start) * self.progress).round()
                    as i32,
            );
            result.place_child(child, PxPosition::new(icon_x, (height - icon_size) / 2));
            result.place_child(
                label,
                PxPosition::new(icon_x + icon_size + gap, (height - label_size.height) / 2),
            );
        } else {
            result.place_child(
                label,
                PxPosition::new(
                    (width - label_size.width) / 2,
                    (height - label_size.height) / 2,
                ),
            );
        }
        Ok(result)
    }
}

/// # extended_floating_action_button
///
/// Show a primary action with an animated label in one of three Expressive
/// sizes.
///
/// ## Usage
///
/// Keep primary actions recognizable and collapse their labels when space is
/// limited.
///
/// ## Parameters
///
/// - `label` — action name, retained as the accessible name while collapsed.
/// - `size` — small, medium, or large container and typography.
/// - `icon` — optional vector or raster icon, sized automatically.
/// - `expanded` — show the label beside the icon; label-only buttons stay
///   expanded.
/// - `modifier` — positioning and size constraints.
/// - `on_click`, `enabled` — activation callback and enabled state.
/// - `container_color`, `content_color` — optional theme color overrides.
/// - `shape`, `elevation` — optional shape and interaction elevation overrides.
/// - `accessibility_label` — optional spoken name replacing the visible label.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     extended_floating_action_button::extended_floating_action_button,
///     material_icons::filled,
/// };
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn compose_action() {
///     let expanded = remember(|| true);
///     assert!(expanded.get());
///     extended_floating_action_button("Compose")
///         .icon(filled::EDIT_SVG)
///         .expanded(expanded.get())
///         .on_click(move || expanded.set(!expanded.get()));
/// }
/// ```
#[tessera]
pub fn extended_floating_action_button(
    #[prop(into)] label: String,
    size: Option<ExtendedFloatingActionButtonSize>,
    #[prop(into)] icon: Option<Painter>,
    expanded: Option<bool>,
    modifier: Option<Modifier>,
    on_click: Option<Callback>,
    enabled: Option<bool>,
    container_color: Option<Color>,
    content_color: Option<Color>,
    shape: Option<Shape>,
    elevation: Option<FloatingActionButtonElevation>,
    #[prop(into)] accessibility_label: Option<String>,
) {
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let size = size.unwrap_or_default();
    let expanded = icon.is_none() || expanded.unwrap_or(true);
    let motion = remember(|| ExpansionMotion::new(expanded));
    launch(expanded, animate_to(motion, expanded));
    let current = ExtendedFabContent {
        label: label.clone(),
        icon,
        size,
        style: size.text_style(&theme),
    };
    let content = remember(|| current.clone());
    if content.with(|content| content != &current) {
        content.set(current);
    }
    floating_action_button()
        .size(size.base_size())
        .modifier(modifier.unwrap_or_default())
        .on_click_optional(on_click)
        .enabled(enabled.unwrap_or(true))
        .container_color_optional(container_color)
        .content_color_optional(content_color)
        .shape_optional(shape)
        .elevation_optional(elevation)
        .accessibility_label(accessibility_label.unwrap_or(label))
        .content(move || {
            let snapshot = content.get();
            let progress = motion.with(|motion| motion.width.value());
            tessera_ui::layout::layout(ExtendedFabLayout {
                size: snapshot.size,
                has_icon: snapshot.icon.is_some(),
                progress,
            })
            .modifier(
                Modifier::new()
                    .clip_to_bounds()
                    .clear_and_set_semantics(SemanticsArgs {
                        hidden: true,
                        test_tag: cfg!(test).then(|| "extended-fab-content".into()),
                        ..Default::default()
                    }),
            )
            .child(move || {
                let snapshot = content.get();
                if let Some(painter) = snapshot.icon {
                    crate::icon::icon()
                        .painter(painter)
                        .size(snapshot.size.icon_size());
                }
                text(snapshot.label).style(snapshot.style).modifier(
                    Modifier::new()
                        .alpha(motion.with(|motion| motion.opacity.value().clamp(0.0, 1.0))),
                );
            });
        });
}

#[cfg(test)]
mod tests {
    use tessera_ui::{State, testing::UiTest};

    use crate::{
        boxed::boxed, column::column, material_icons::filled, spacer::spacer, theme::material_theme,
    };

    use super::*;

    #[derive(Clone)]
    struct FixtureConfig {
        label: String,
        size: ExtendedFloatingActionButtonSize,
    }

    #[derive(Clone, Default)]
    struct FixtureHandles {
        expanded: Option<State<bool>>,
        config: Option<State<FixtureConfig>>,
    }

    fn tag(name: &str) -> Modifier {
        Modifier::new().semantics(SemanticsArgs {
            test_tag: Some(name.into()),
            ..Default::default()
        })
    }

    #[tessera]
    fn fixture(
        size: ExtendedFloatingActionButtonSize,
        expanded: bool,
        has_icon: bool,
        enabled: bool,
        probe: Option<tessera_ui::testing::TestProbe<FixtureHandles>>,
    ) {
        let expanded = remember(|| expanded);
        let config = remember(|| FixtureConfig {
            label: "Compose".into(),
            size,
        });
        if let Some(probe) = &probe {
            probe.with_mut(|handles| handles.expanded = Some(expanded));
        }
        if let Some(probe) = &probe {
            probe.with_mut(|handles| handles.config = Some(config));
        }
        material_theme().child(move || {
            column().children(move || {
                boxed().modifier(tag("fab")).children(move || {
                    let config = config.get();
                    extended_floating_action_button(config.label)
                        .size(config.size)
                        .icon_optional(if has_icon {
                            Some(filled::EDIT_SVG.into())
                        } else {
                            None
                        })
                        .expanded(expanded.get())
                        .enabled(enabled)
                        .on_click(move || expanded.set(!expanded.get()));
                });
            });
        });
    }

    #[tessera]
    fn geometry_fixture(size: ExtendedFloatingActionButtonSize) {
        column().children(move || {
            tessera_ui::layout::layout(ExtendedFabLayout {
                size,
                has_icon: true,
                progress: 1.0,
            })
            .modifier(tag("geometry"))
            .child(move || {
                spacer().modifier(tag("icon").size(size.icon_size(), size.icon_size()));
                spacer().modifier(tag("label").size(Dp(80.0), Dp(24.0)));
            });
        });
    }

    #[test]
    fn expressive_geometry_uses_current_padding_gaps_and_icon_sizes() {
        for (size, width, height, icon_x, label_x, icon_size) in [
            (ExtendedFloatingActionButtonSize::Small, 144, 56, 16, 48, 24),
            (
                ExtendedFloatingActionButtonSize::Medium,
                172,
                80,
                26,
                66,
                28,
            ),
            (ExtendedFloatingActionButtonSize::Large, 188, 96, 28, 80, 36),
        ] {
            let snapshot = UiTest::new()
                .content(move || geometry_fixture(size))
                .viewport(500, 400)
                .frame();
            snapshot.node("geometry").size(width, height);
            snapshot.node("icon").size(icon_size, icon_size);
            assert_eq!(snapshot.node("icon").bounds().x, Px::new(icon_x));
            assert_eq!(snapshot.node("label").bounds().x, Px::new(label_x));
        }
    }

    #[test]
    fn all_sizes_collapse_to_square_containers() {
        for (size, side) in [
            (ExtendedFloatingActionButtonSize::Small, 56),
            (ExtendedFloatingActionButtonSize::Medium, 80),
            (ExtendedFloatingActionButtonSize::Large, 96),
        ] {
            let snapshot = UiTest::new()
                .content(move || fixture(size, false, true, true))
                .viewport(500, 400)
                .frame();
            snapshot.node("fab").size(side, side);
        }
    }

    #[test]
    fn label_only_buttons_remain_expanded_without_an_icon() {
        let snapshot = UiTest::new()
            .content(|| fixture(ExtendedFloatingActionButtonSize::Small, false, false, true))
            .viewport(500, 400)
            .frame();
        let bounds = snapshot.node("fab").bounds();
        assert!(bounds.width > Px::new(56));
        assert_eq!(bounds.height, Px::new(56));
    }

    #[test]
    fn changing_size_and_label_updates_retained_content() {
        let probe = tessera_ui::testing::TestProbe::<FixtureHandles>::default();

        let mut session = UiTest::new()
            .content(|| {
                fixture(ExtendedFloatingActionButtonSize::Small, true, true, true)
                    .probe(probe.clone())
            })
            .viewport(500, 400);
        let before = session.frame().node("fab").bounds();
        probe
            .get()
            .config
            .expect("fixture state")
            .with_mut(|config| {
                config.size = ExtendedFloatingActionButtonSize::Medium;
                config.label = "Create a document".into();
            });
        let after = session.frames(1).node("fab").bounds();
        assert!(after.width > before.width);
        assert_eq!(after.height, Px::new(80));
    }

    #[test]
    fn click_collapse_animates_and_disabled_buttons_ignore_input() {
        let probe = tessera_ui::testing::TestProbe::<FixtureHandles>::default();

        for enabled in [true, false] {
            let mut session = UiTest::new()
                .content(|| {
                    fixture(ExtendedFloatingActionButtonSize::Small, true, true, enabled)
                        .probe(probe.clone())
                })
                .viewport(500, 400);
            let initial = session.frame().node("fab").bounds();
            session.click_at(PxPosition::from_f32_xy(
                initial.x.to_f32() + 28.0,
                initial.y.to_f32() + 28.0,
            ));
            session.frames(1);
            let intermediate = session
                .advance(std::time::Duration::from_nanos(33_333_333))
                .node("fab")
                .bounds();
            if enabled {
                assert!(!probe.get().expanded.expect("fixture state").get());
                assert!(intermediate.width < initial.width);
                assert!(intermediate.width > Px::new(56));
            } else {
                assert!(probe.get().expanded.expect("fixture state").get());
                assert_eq!(intermediate.width, initial.width);
            }
            let final_snapshot = session.frames(114);
            assert_eq!(
                final_snapshot.node("fab").bounds().width,
                if enabled { Px::new(56) } else { initial.width }
            );
        }
    }

    #[test]
    fn reversing_expansion_keeps_position_and_velocity_then_settles() {
        use crate::animation::{Motion as _, test_instant as at};

        let mut motion = ExpansionMotion::new(false);
        motion.retarget(true, at(0));
        motion.advance(at(33_333_333));
        let before = (motion.width.value(), motion.width.velocity());
        motion.retarget(false, at(33_333_333));
        assert_eq!((motion.width.value(), motion.width.velocity()), before);
        for frame in 3..120 {
            motion.advance(at(frame * 16_666_667));
        }
        assert!(motion.is_idle());
        assert_eq!(motion.width.value(), 0.0);
        assert_eq!(motion.opacity.value(), 0.0);
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    #[ignore = "Requires a native GPU adapter"]
    fn rendered_labels_restore_after_a_full_collapse_and_expansion() {
        let probe = tessera_ui::testing::TestProbe::<FixtureHandles>::default();

        use tessera_ui::testing::UiTest;

        let mut renderer = UiTest::new()
            .viewport(500, 400)
            .rendered(crate::TesseraComponents)
            .content(|| {
                fixture(ExtendedFloatingActionButtonSize::Small, true, true, true)
                    .probe(probe.clone())
            });
        renderer.frames(3);
        let expanded = renderer.read_pixels();
        probe.get().expanded.expect("fixture state").set(false);
        renderer.frames(120);
        let collapsed = renderer.read_pixels();
        assert_ne!(expanded, collapsed);
        probe.get().expanded.expect("fixture state").set(true);
        renderer.frames(120);
        let restored = renderer.read_pixels();
        crate::test_utils::assert_pixels_near(&expanded, &restored, 500);
    }
}
