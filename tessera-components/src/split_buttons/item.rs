use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, Dp, FrameNanosControl, LayoutResult,
    MeasurementError, Modifier, PxPosition, RenderSlot, SemanticsModifierNode, State,
    accesskit::{HasPopup, Role},
    current_frame_nanos,
    layout::{MeasureScope, layout},
    modifier::ModifierCapabilityExt as _,
    provide_context, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment},
    animation::Animatable,
    icon::{IconRotation, IconSize},
    modifier::{InteractionState, ModifierExt as _},
    row::row,
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceStyle, surface},
    theme::{MaterialTheme, TextStyle},
};

use super::{SplitButtonDefaults, SplitButtonItemArgs, SplitButtonSize};

#[derive(Clone, Copy, PartialEq)]
struct MotionTarget {
    fraction: f32,
    radius: f32,
    elevation: f32,
    rotation: f32,
}

struct SplitMotion {
    fraction: Animatable,
    radius: Animatable,
    elevation: Animatable,
    rotation: Animatable,
    target: MotionTarget,
    frame: u64,
}

impl SplitMotion {
    fn new(target: MotionTarget, frame: u64) -> Self {
        Self {
            fraction: Animatable::spring(target.fraction, 40.0, 1.0),
            radius: Animatable::spring(target.radius, 40.0, 1.0),
            elevation: Animatable::spring(target.elevation, 40.0, 1.0),
            rotation: Animatable::spring(target.rotation, 40.0, 1.0),
            target,
            frame,
        }
    }

    fn advance(&mut self, frame: u64) {
        if frame > self.frame {
            let dt = (frame - self.frame) as f32 / 1_000_000_000.0;
            self.fraction.tick(dt);
            self.radius.tick(dt);
            self.elevation.tick(dt);
            self.rotation.tick(dt);
            self.frame = frame;
        }
    }

    fn retarget(&mut self, target: MotionTarget, frame: u64, fixed_shape: bool) {
        self.advance(frame);
        if fixed_shape {
            self.fraction = Animatable::spring(target.fraction, 40.0, 1.0);
            self.radius = Animatable::spring(target.radius, 40.0, 1.0);
        } else {
            self.fraction.set_target(target.fraction);
            self.radius.set_target(target.radius);
        }
        self.elevation.set_target(target.elevation);
        self.rotation.set_target(target.rotation);
        self.target = target;
    }

    fn is_idle(&self) -> bool {
        self.fraction.is_idle()
            && self.radius.is_idle()
            && self.elevation.is_idle()
            && self.rotation.is_idle()
    }

    fn shape(&self, leading: bool) -> Shape {
        let inner =
            RoundedCorner::relative(self.fraction.value(), Dp(self.radius.value() as f64), 2.0);
        let outer = RoundedCorner::Capsule;
        let (left, right) = if leading {
            (outer, inner)
        } else {
            (inner, outer)
        };
        Shape::RoundedRectangle {
            top_left: left,
            top_right: right,
            bottom_right: right,
            bottom_left: left,
        }
    }
}

struct DisclosureSemantics(Option<bool>);

impl SemanticsModifierNode for DisclosureSemantics {
    fn apply(
        &self,
        node: &mut AccessibilityNode,
        _handler: &mut Option<AccessibilityActionHandler>,
    ) {
        node.expanded = self.0;
        node.has_popup = self.0.map(|_| HasPopup::Menu);
    }
}

#[derive(Clone, Copy, PartialEq)]
struct OpticalOffset(Dp);

impl tessera_ui::LayoutModifierNode for OpticalOffset {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let child = input.children()[0];
        let constraint = *input.parent_constraint().as_ref();
        let size = child.measure(input, &constraint)?.size();
        let mut result = LayoutResult::new(size);
        result.place_child(child, PxPosition::from_f32_xy(self.0.to_pixels_f32(), 0.0));
        Ok(result)
    }
}

#[tessera]
pub(super) fn split_button_item(
    args: Option<SplitButtonItemArgs>,
    size: Option<SplitButtonSize>,
    leading: Option<bool>,
    interaction_state: Option<State<InteractionState>>,
    shape: Option<Shape>,
    expanded: Option<bool>,
    content: Option<RenderSlot>,
) {
    let args = args.expect("split button item arguments");
    let size = size.unwrap_or_default();
    let leading = leading.unwrap_or(false);
    let enabled = args.enabled;
    let content = content.unwrap_or_else(RenderSlot::empty);
    let interaction = if let Some(state) = interaction_state {
        state
    } else {
        remember(InteractionState::new)
    };
    if !enabled && interaction.with(|state| *state != InteractionState::new()) {
        interaction.set(InteractionState::new());
    }
    let pressed = enabled && interaction.with(InteractionState::is_pressed);
    let hovered = enabled && interaction.with(InteractionState::is_hovered);
    let focused = enabled && interaction.with(InteractionState::is_focused);
    let checked = !leading && expanded.unwrap_or(false);
    let (fraction, radius) = if shape.is_some() {
        (0.0, 0.0)
    } else if pressed {
        (
            0.0,
            SplitButtonDefaults::active_inner_corner_size(size).0 as f32,
        )
    } else if checked {
        (0.5, 0.0)
    } else if hovered || focused {
        (
            0.0,
            SplitButtonDefaults::active_inner_corner_size(size).0 as f32,
        )
    } else {
        (0.0, SplitButtonDefaults::inner_corner_size(size).0 as f32)
    };
    let elevation = if !enabled {
        0.0
    } else if hovered && !pressed && args.elevation.is_some() {
        3.0
    } else {
        args.elevation.unwrap_or(Dp(0.0)).0 as f32
    };
    let target = MotionTarget {
        fraction,
        radius,
        elevation,
        rotation: if expanded == Some(true) { 180.0 } else { 0.0 },
    };
    let frame = current_frame_nanos();
    let motion = remember(|| SplitMotion::new(target, frame));
    if motion.with(|motion| motion.target) != target {
        motion.with_mut(|motion| motion.retarget(target, frame, shape.is_some()));
    }
    if !motion.with(SplitMotion::is_idle) {
        receive_frame_nanos(move |frame| {
            motion.with_mut(|motion| motion.advance(frame));
            if motion.with(SplitMotion::is_idle) {
                FrameNanosControl::Stop
            } else {
                FrameNanosControl::Continue
            }
        });
    }
    let content_color = args.colors.content_color(enabled);
    let overlay = content_color.with_alpha(1.0);
    let container = args.colors.container_color(enabled);
    let border = args.colors.border_color(enabled);
    let container = if checked {
        container.blend_over(overlay, 0.1)
    } else {
        container
    };
    let border = if checked {
        border.blend_over(overlay, 0.1)
    } else {
        border
    };
    let content_color = if checked {
        content_color.blend_over(overlay, 0.1)
    } else {
        content_color
    };
    let style = if args.border_width.0 > 0.0 {
        SurfaceStyle::FilledOutlined {
            fill_color: container,
            border_color: border,
            border_width: args.border_width,
        }
    } else {
        SurfaceStyle::Filled { color: container }
    };
    let optical = if leading || shape.is_some() {
        Dp(0.0)
    } else {
        Dp(SplitButtonDefaults::trailing_optical_offset(size).0
            * (1.0 - 2.0 * motion.with(|motion| motion.fraction.value()) as f64))
    };
    let padding = args.content_padding;
    let rotation = motion.with(|motion| motion.rotation.value());
    let text_style = size.text_style(&use_context::<MaterialTheme>().unwrap().get());
    let icon_size = if leading {
        SplitButtonDefaults::leading_icon_size(size)
    } else {
        SplitButtonDefaults::trailing_icon_size(size)
    };
    surface()
        .modifier(
            args.modifier
                .size_in(
                    Some(args.min_width),
                    None,
                    Some(args.container_height),
                    None,
                )
                .push_semantics(DisclosureSemantics(expanded)),
        )
        .style(style)
        .shape(shape.unwrap_or_else(|| motion.with(|motion| motion.shape(leading))))
        .content_alignment(Alignment::Center)
        .content_color(content_color)
        .enabled(enabled)
        .ripple_color(content_color)
        .interaction_state(interaction)
        .tonal_elevation(args.tonal_elevation)
        .elevation(Dp(
            motion.with(|motion| motion.elevation.value()).max(0.0) as f64
        ))
        .on_click_optional(args.on_click)
        .accessibility_role(Role::Button)
        .accessibility_focusable(enabled && args.on_click.is_some())
        .accessibility_label_optional(args.accessibility_label)
        .accessibility_description_optional(args.accessibility_description)
        .child(move || {
            provide_context(
                || text_style,
                move || {
                    let inherited = use_context::<TextStyle>().unwrap();
                    if inherited.get() != text_style {
                        inherited.set(text_style);
                    }
                    layout().policy(OpticalOffset(optical)).child(move || {
                        row()
                            .modifier(Modifier::new().padding(padding))
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .children(move || {
                                provide_context(
                                    || IconSize(icon_size),
                                    move || {
                                        let inherited = use_context::<IconSize>().unwrap();
                                        if inherited.get().0 != icon_size {
                                            inherited.set(IconSize(icon_size));
                                        }
                                        provide_context(
                                            || IconRotation(rotation),
                                            move || {
                                                let inherited =
                                                    use_context::<IconRotation>().unwrap();
                                                if inherited.get().0 != rotation {
                                                    inherited.set(IconRotation(rotation));
                                                }
                                                content.render();
                                            },
                                        );
                                    },
                                );
                            });
                    });
                },
            );
        });
}

#[cfg(test)]
mod tests {
    use tessera_ui::{Px, PxSize};

    use super::*;

    #[test]
    fn active_inner_corner_tokens_keep_extra_small_distinct_from_small() {
        assert_eq!(
            SplitButtonDefaults::active_inner_corner_size(SplitButtonSize::ExtraSmall),
            Dp(8.0)
        );
        assert_eq!(
            SplitButtonDefaults::active_inner_corner_size(SplitButtonSize::Small),
            Dp(12.0)
        );
        assert_eq!(
            SplitButtonDefaults::inner_corner_size(SplitButtonSize::ExtraSmall),
            Dp(4.0)
        );
    }

    #[test]
    fn corner_rotation_and_elevation_motion_preserve_velocity_and_settle() {
        let initial = MotionTarget {
            fraction: 0.0,
            radius: 4.0,
            elevation: 1.0,
            rotation: 0.0,
        };
        let active = MotionTarget {
            radius: 12.0,
            elevation: 3.0,
            ..initial
        };
        let mut motion = SplitMotion::new(initial, 0);
        motion.retarget(active, 0, false);
        motion.advance(33_333_334);
        let radius = (motion.radius.value(), motion.radius.velocity());
        let elevation = (motion.elevation.value(), motion.elevation.velocity());
        motion.retarget(
            MotionTarget {
                fraction: 0.5,
                radius: 0.0,
                elevation: 1.0,
                rotation: 180.0,
            },
            33_333_334,
            false,
        );
        assert_eq!((motion.radius.value(), motion.radius.velocity()), radius);
        assert_eq!(
            (motion.elevation.value(), motion.elevation.velocity()),
            elevation
        );
        for frame in 3..150 {
            motion.advance(frame * 16_666_667);
            assert!((0.0..=0.5).contains(&motion.fraction.value()));
            assert!((0.0..=180.0).contains(&motion.rotation.value()));
        }
        assert!(motion.is_idle());
        let Shape::RoundedRectangle {
            top_left,
            top_right,
            ..
        } = motion.shape(false)
        else {
            unreachable!()
        };
        let size = PxSize::new(Px(96), Px(136));
        assert_eq!(top_left.resolve(size), (48.0, 2.0));
        assert_eq!(top_right.resolve(size), (48.0, 2.0));
        motion.retarget(
            MotionTarget {
                elevation: 3.0,
                ..initial
            },
            2_500_000_000,
            true,
        );
        assert!(motion.fraction.is_idle() && motion.radius.is_idle());
        assert!(!motion.elevation.is_idle() && !motion.rotation.is_idle());
        for frame in 151..300 {
            motion.advance(frame * 16_666_667);
        }
        assert!(motion.is_idle());
    }
}
