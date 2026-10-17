//! Extended floating action buttons with animated labels.
//!
//! ## Usage
//!
//! Emphasize a primary action with an icon and a descriptive label.

use tessera_ui::{
    AxisConstraint, Callback, Color, ComputedData, Constraint, Dp, FrameNanosControl, LayoutResult,
    MeasurementError, Modifier, Px, PxPosition, current_frame_nanos,
    layout::{MeasureScope, layout},
    receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    animation::Animatable,
    floating_action_button::{
        FloatingActionButtonElevation, FloatingActionButtonSize, floating_action_button,
    },
    modifier::{ModifierExt, SemanticsArgs},
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
    frame_nanos: u64,
}

impl ExpansionMotion {
    fn new(expanded: bool, frame_nanos: u64) -> Self {
        let value = if expanded { 1.0 } else { 0.0 };
        Self {
            width: Animatable::spring(value, 800.0_f32.sqrt(), 0.6),
            opacity: Animatable::spring(value, 3800.0_f32.sqrt(), 1.0),
            frame_nanos,
        }
    }

    fn advance(&mut self, frame_nanos: u64) {
        if frame_nanos > self.frame_nanos {
            let dt = (frame_nanos - self.frame_nanos) as f32 / 1_000_000_000.0;
            self.width.tick(dt);
            self.opacity.tick(dt);
            self.frame_nanos = frame_nanos;
        }
    }

    fn set_expanded(&mut self, expanded: bool, frame_nanos: u64) {
        self.advance(frame_nanos);
        let target = if expanded { 1.0 } else { 0.0 };
        self.width.set_target(target);
        self.opacity.set_target(target);
    }

    fn is_idle(&self) -> bool {
        self.width.is_idle() && self.opacity.is_idle()
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

impl tessera_ui::LayoutModifierNode for ExtendedFabLayout {
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
///     extended_floating_action_button::{
///         ExtendedFloatingActionButtonSize, extended_floating_action_button,
///     },
///     material_icons::filled,
/// };
/// use tessera_ui::{Dp, remember, tessera};
///
/// assert_eq!(ExtendedFloatingActionButtonSize::Medium.height(), Dp(80.0));
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
    let frame = current_frame_nanos();
    let motion = remember(|| ExpansionMotion::new(expanded, frame));
    let was_expanded = remember(|| expanded);
    if was_expanded.get() != expanded {
        motion.with_mut(|motion| motion.set_expanded(expanded, frame));
        was_expanded.set(expanded);
    }
    if !motion.with(ExpansionMotion::is_idle) {
        receive_frame_nanos(move |frame| {
            motion.with_mut(|motion| motion.advance(frame));
            if motion.with(ExpansionMotion::is_idle) {
                FrameNanosControl::Stop
            } else {
                FrameNanosControl::Continue
            }
        });
    }
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
            layout()
                .modifier(
                    Modifier::new()
                        .clip_to_bounds()
                        .clear_and_set_semantics(SemanticsArgs {
                            hidden: true,
                            test_tag: cfg!(test).then(|| "extended-fab-content".into()),
                            ..Default::default()
                        }),
                )
                .policy(ExtendedFabLayout {
                    size: snapshot.size,
                    has_icon: snapshot.icon.is_some(),
                    progress,
                })
                .child(move || {
                    let snapshot = content.get();
                    if let Some(painter) = snapshot.icon {
                        crate::icon::icon()
                            .painter(painter)
                            .size(snapshot.size.icon_size());
                    }
                    text()
                        .content(snapshot.label)
                        .style(snapshot.style)
                        .modifier(
                            Modifier::new().alpha(
                                motion.with(|motion| motion.opacity.value().clamp(0.0, 1.0)),
                            ),
                        );
                });
        });
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use tessera_ui::{
        CursorEventContent, GestureState, PointerChange, PressKeyEventType, State,
        testing::{
            layout_test, pump_layout_test_session, pump_layout_test_session_with_input,
            start_layout_test_session,
        },
    };

    use crate::{
        boxed::boxed, column::column, material_icons::filled, spacer::spacer, theme::material_theme,
    };

    use super::*;

    #[derive(Clone)]
    struct FixtureConfig {
        label: String,
        size: ExtendedFloatingActionButtonSize,
    }

    thread_local! {
        static EXPANDED: Cell<Option<State<bool>>> = const { Cell::new(None) };
        static CONFIG: Cell<Option<State<FixtureConfig>>> = const { Cell::new(None) };
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
    ) {
        let expanded = remember(|| expanded);
        let config = remember(|| FixtureConfig {
            label: "Compose".into(),
            size,
        });
        EXPANDED.set(Some(expanded));
        CONFIG.set(Some(config));
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
            layout()
                .modifier(tag("geometry"))
                .policy(ExtendedFabLayout {
                    size,
                    has_icon: true,
                    progress: 1.0,
                })
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
            let snapshot = layout_test(move || {
                geometry_fixture(size);
            })
            .viewport_px(500, 400)
            .run();
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
            let snapshot = layout_test(move || {
                fixture(size, false, true, true);
            })
            .viewport_px(500, 400)
            .run();
            snapshot.node("fab").size(side, side);
        }
    }

    #[test]
    fn label_only_buttons_remain_expanded_without_an_icon() {
        let snapshot = layout_test(|| {
            fixture(ExtendedFloatingActionButtonSize::Small, false, false, true);
        })
        .viewport_px(500, 400)
        .run();
        let bounds = snapshot.node("fab").bounds();
        assert!(bounds.width > Px::new(56));
        assert_eq!(bounds.height, Px::new(56));
    }

    #[test]
    fn changing_size_and_label_updates_retained_content() {
        let mut session = start_layout_test_session(
            layout_test(|| {
                fixture(ExtendedFloatingActionButtonSize::Small, true, true, true);
            })
            .viewport_px(500, 400),
        );
        let before = pump_layout_test_session(&mut session, 0)
            .node("fab")
            .bounds();
        CONFIG.get().expect("fixture state").with_mut(|config| {
            config.size = ExtendedFloatingActionButtonSize::Medium;
            config.label = "Create a document".into();
        });
        let after = pump_layout_test_session(&mut session, 16_666_667)
            .node("fab")
            .bounds();
        assert!(after.width > before.width);
        assert_eq!(after.height, Px::new(80));
    }

    #[test]
    fn click_collapse_animates_and_disabled_buttons_ignore_input() {
        for enabled in [true, false] {
            let mut session = start_layout_test_session(
                layout_test(move || {
                    fixture(ExtendedFloatingActionButtonSize::Small, true, true, enabled);
                })
                .viewport_px(500, 400),
            );
            let initial = pump_layout_test_session(&mut session, 0)
                .node("fab")
                .bounds();
            let cursor = Some(PxPosition::from_f32_xy(
                initial.x.to_f32() + 28.0,
                initial.y.to_f32() + 28.0,
            ));
            for (index, event) in [
                CursorEventContent::Pressed(PressKeyEventType::Left),
                CursorEventContent::Released(PressKeyEventType::Left),
            ]
            .into_iter()
            .enumerate()
            {
                pump_layout_test_session_with_input(
                    &mut session,
                    (index as u64 + 1) * 16_666_667,
                    vec![PointerChange::new(
                        tessera_ui::time::Instant::now(),
                        1,
                        event,
                        GestureState::TapCandidate,
                    )],
                    cursor,
                );
            }
            pump_layout_test_session(&mut session, 50_000_000);
            let intermediate = pump_layout_test_session(&mut session, 83_333_333)
                .node("fab")
                .bounds();
            if enabled {
                assert!(!EXPANDED.get().expect("fixture state").get());
                assert!(intermediate.width < initial.width);
                assert!(intermediate.width > Px::new(56));
            } else {
                assert!(EXPANDED.get().expect("fixture state").get());
                assert_eq!(intermediate.width, initial.width);
            }
            let mut final_snapshot = pump_layout_test_session(&mut session, 100_000_000);
            for frame in 7..120 {
                final_snapshot = pump_layout_test_session(&mut session, frame * 16_666_667);
            }
            assert_eq!(
                final_snapshot.node("fab").bounds().width,
                if enabled { Px::new(56) } else { initial.width }
            );
        }
    }

    #[test]
    fn reversing_expansion_keeps_position_and_velocity_then_settles() {
        let mut motion = ExpansionMotion::new(false, 0);
        motion.set_expanded(true, 0);
        motion.advance(33_333_333);
        let before = (motion.width.value(), motion.width.velocity());
        motion.set_expanded(false, 33_333_333);
        assert_eq!((motion.width.value(), motion.width.velocity()), before);
        for frame in 3..120 {
            motion.advance(frame * 16_666_667);
        }
        assert!(motion.is_idle());
        assert_eq!(motion.width.value(), 0.0);
        assert_eq!(motion.opacity.value(), 0.0);
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    #[ignore = "Requires a native GPU adapter"]
    fn rendered_labels_restore_after_a_full_collapse_and_expansion() {
        use tessera_ui::headless::{HeadlessConfig, HeadlessRenderer};

        let mut renderer = HeadlessRenderer::new(
            Box::new(|| {
                fixture(ExtendedFloatingActionButtonSize::Small, true, true, true);
            }),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 500,
                height: 400,
                ..Default::default()
            },
        )
        .expect("headless GPU renderer");
        renderer.render_frames(3);
        let expanded = renderer.read_pixels().expect("expanded pixels");
        EXPANDED.get().expect("fixture state").set(false);
        renderer.render_frames(120);
        let collapsed = renderer.read_pixels().expect("collapsed pixels");
        assert_ne!(expanded, collapsed);
        EXPANDED.get().expect("fixture state").set(true);
        renderer.render_frames(120);
        let restored = renderer.read_pixels().expect("restored pixels");
        let different_pixels = expanded
            .chunks_exact(4)
            .zip(restored.chunks_exact(4))
            .filter(|(before, after)| before != after)
            .count();
        assert_eq!(
            different_pixels, 0,
            "expanded pixels must return after the animation settles"
        );
    }
}
