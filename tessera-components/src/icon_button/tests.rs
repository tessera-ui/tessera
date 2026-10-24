use std::cell::{Cell, RefCell};

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, Px, PxPosition, PxSize,
    accesskit::{Action, Node, NodeId, Rect, TreeUpdate},
    modifier::FocusModifierExt as _,
    provide_context,
};

use crate::{
    column::column, painter::try_painter, row::row, spacer::spacer, theme::material_theme,
};

use super::*;

const SIZES: [IconButtonSize; 5] = [
    IconButtonSize::ExtraSmall,
    IconButtonSize::Small,
    IconButtonSize::Medium,
    IconButtonSize::Large,
    IconButtonSize::ExtraLarge,
];
const WIDTHS: [IconButtonWidth; 3] = [
    IconButtonWidth::Narrow,
    IconButtonWidth::Uniform,
    IconButtonWidth::Wide,
];

#[derive(Clone, Copy)]
struct Settings {
    focus_reference: bool,
    presentation_ripple: Option<RipplePresentation>,
    presentation_state: Option<InteractionVisualState>,
    enabled: bool,
    variant: IconButtonVariant,
    selected: Option<bool>,
    shape: IconButtonShape,
    fixed_shape: Option<Shape>,
    color_style: IconButtonColorStyle,
    color: Option<Color>,
    content: Option<Color>,
    inherited: Color,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            focus_reference: false,
            presentation_state: None,
            presentation_ripple: None,
            enabled: true,
            variant: IconButtonVariant::Filled,
            selected: None,
            shape: IconButtonShape::Round,
            fixed_shape: None,
            color_style: IconButtonColorStyle::Vibrant,
            color: None,
            content: None,
            inherited: Color::RED,
        }
    }
}

thread_local! {
    static FOCUS_REFERENCE: Cell<Option<tessera_ui::PxRect>> = const { Cell::new(None) };
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
    static CLICKS: Cell<Option<State<usize>>> = const { Cell::new(None) };
    static INTERACTIONS: RefCell<[Option<State<InteractionState>>; 15]> = const { RefCell::new([None; 15]) };
}

#[tessera]
fn fixture() {
    let settings = remember(Settings::default);
    let clicks = remember(|| 0usize);
    let painter = remember(|| {
        try_painter(br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M0 0H24V24H0Z"/></svg>"#.as_slice()).unwrap()
    });
    SETTINGS.set(Some(settings));
    CLICKS.set(Some(clicks));
    material_theme().child(move || {
        provide_context(
            || ContentColor {
                current: settings.get().inherited,
            },
            move || {
                let inherited = use_context::<ContentColor>().unwrap();
                if inherited.get().current != settings.get().inherited {
                    inherited.set(ContentColor {
                        current: settings.get().inherited,
                    });
                }
                boxed()
                    .modifier(
                        Modifier::new()
                            .fill_max_size()
                            .background(Color::WHITE)
                            .padding_all(Dp(16.0)),
                    )
                    .children(move || {
                        column().children(move || {
                            for (size_index, size) in SIZES.into_iter().enumerate() {
                                row().children(move || {
                                    for (width_index, width) in WIDTHS.into_iter().enumerate() {
                                        let index = size_index * 3 + width_index;
                                        let state = remember(InteractionState::new);
                                        INTERACTIONS.with(|states| {
                                            states.borrow_mut()[index] = Some(state)
                                        });
                                        let value = settings.get();
                                        icon_button()
                                            .icon(painter.get())
                                            .size(size)
                                            .width(width)
                                            .variant(value.variant)
                                            .selected_optional(value.selected)
                                            .shape_style(value.shape)
                                            .shape_optional(value.fixed_shape)
                                            .enabled(value.enabled)
                                            .color_style(value.color_style)
                                            .color_optional(value.color)
                                            .content_color_optional(value.content)
                                            .interaction_state(state)
                                            .presentation_state_optional(if index == 7 {
                                                value.presentation_state
                                            } else {
                                                None
                                            })
                                            .presentation_ripple_optional(if index == 7 {
                                                value.presentation_ripple
                                            } else {
                                                None
                                            })
                                            .accessibility_label(format!("Icon action {index}"))
                                            .on_click(move || {
                                                clicks.set(clicks.get() + 1);
                                                settings.with_mut(|settings| {
                                                    if let Some(selected) = &mut settings.selected {
                                                        *selected = !*selected;
                                                    }
                                                });
                                            });
                                        spacer().modifier(Modifier::new().width(Dp(24.0)));
                                    }
                                });
                                spacer().modifier(Modifier::new().height(Dp(8.0)));
                            }
                            if settings.with(|settings| settings.focus_reference) {
                                let reference_modifier = Modifier::new()
                                    .size(Dp(66.0), Dp(66.0))
                                    .on_globally_positioned(|bounds| {
                                        FOCUS_REFERENCE.set(Some(bounds))
                                    });
                                surface()
                                    .shape(Shape::CAPSULE)
                                    .style(SurfaceStyle::Filled {
                                        color: Color::WHITE,
                                    })
                                    .show_state_layer(false)
                                    .presentation_state_optional(
                                        settings.with(|settings| settings.presentation_state),
                                    )
                                    .presentation_ripple_optional(
                                        settings.with(|settings| settings.presentation_ripple),
                                    )
                                    .modifier(reference_modifier);
                            }
                        });
                    });
            },
        );
    });
}

fn node(tree: &TreeUpdate, index: usize) -> &(NodeId, Node) {
    tree.nodes
        .iter()
        .find(|(_, node)| node.label() == Some(format!("Icon action {index}").as_str()))
        .unwrap()
}

fn bounds(tree: &TreeUpdate, index: usize) -> Rect {
    node(tree, index).1.bounds().unwrap()
}

fn center(rect: Rect) -> PxPosition {
    PxPosition::from_f32_xy(
        ((rect.x0 + rect.x1) / 2.0) as f32,
        ((rect.y0 + rect.y1) / 2.0) as f32,
    )
}

fn click(renderer: &mut HeadlessRenderer, point: PxPosition) {
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.step_frame();
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(8);
}

fn check_geometry(samples: u32, scale: f64) {
    let width = (600.0 * scale) as u32;
    let height = (600.0 * scale) as u32;
    let mut renderer = HeadlessRenderer::new(
        Box::new(|| {
            fixture();
        }),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width,
            height,
            scale_factor: scale,
            sample_count: samples,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    let tree = renderer.accessibility_snapshot().unwrap();
    let pixels = renderer.read_pixels().unwrap();
    let pixel = |x: usize, y: usize| {
        &pixels[(y * width as usize + x) * 4..(y * width as usize + x + 1) * 4]
    };
    let expected_widths = [
        [28usize, 32, 40],
        [32, 40, 52],
        [48, 56, 72],
        [64, 96, 128],
        [104, 136, 184],
    ];
    for (row, height) in [32usize, 40, 56, 96, 136].into_iter().enumerate() {
        for column in 0..3 {
            let index = row * 3 + column;
            let rect = bounds(&tree, index);
            let visual_width = (expected_widths[row][column] as f64 * scale).round() as usize;
            let visual_height = (height as f64 * scale).round() as usize;
            assert_eq!(
                rect.width(),
                visual_width.max((48.0 * scale) as usize) as f64
            );
            assert_eq!(
                rect.height(),
                visual_height.max((48.0 * scale) as usize) as f64
            );
            let x = ((rect.x0 + rect.x1) / 2.0) as usize;
            let y = ((rect.y0 + rect.y1) / 2.0) as usize;
            let background = pixel((580.0 * scale) as usize, y);
            let horizontal: Vec<_> = (rect.x0 as usize..rect.x1 as usize)
                .filter(|&x| pixel(x, y) != background)
                .collect();
            let vertical: Vec<_> = (rect.y0 as usize..rect.y1 as usize)
                .filter(|&y| pixel(x, y) != pixel((580.0 * scale) as usize, y))
                .collect();
            assert_eq!(
                horizontal.last().unwrap() - horizontal.first().unwrap() + 1,
                visual_width,
                "visual width {index}"
            );
            assert_eq!(
                vertical.last().unwrap() - vertical.first().unwrap() + 1,
                visual_height,
                "visual height {index}"
            );
            let foreground = pixel(x, y);
            let icon_width = (rect.x0 as usize..rect.x1 as usize)
                .filter(|&x| pixel(x, y) == foreground)
                .count();
            let icon_height = (rect.y0 as usize..rect.y1 as usize)
                .filter(|&y| pixel(x, y) == foreground)
                .count();
            let expected_icon = ([20.0_f64, 24.0, 24.0, 32.0, 40.0][row] * scale).round() as usize;
            assert_eq!(icon_width, expected_icon, "icon width {index}");
            assert_eq!(icon_height, expected_icon, "icon height {index}");
        }
    }
    let settings = SETTINGS.get().unwrap();
    let scheme = MaterialTheme::default().color_scheme;
    let point = center(bounds(&tree, 7));
    let assert_icon_color = |renderer: &HeadlessRenderer, color: Color| {
        let pixels = renderer.read_pixels().unwrap();
        let offset = (point.y as usize * width as usize + point.x as usize) * 4;
        for (channel, (actual, mut expected)) in pixels[offset..offset + 4]
            .iter()
            .zip(color.to_array())
            .enumerate()
        {
            if channel < 3 {
                expected = if expected <= 0.0031308 {
                    12.92 * expected
                } else {
                    1.055 * expected.powf(1.0 / 2.4) - 0.055
                };
            }
            assert!(
                actual.abs_diff((expected * 255.0).round() as u8) <= 2,
                "icon color: {:?}, expected {color:?}",
                &pixels[offset..offset + 4]
            );
        }
    };
    for (variant, selected, content) in [
        (IconButtonVariant::Filled, None, scheme.on_primary),
        (
            IconButtonVariant::Filled,
            Some(false),
            scheme.on_surface_variant,
        ),
        (IconButtonVariant::Filled, Some(true), scheme.on_primary),
        (
            IconButtonVariant::FilledTonal,
            Some(false),
            scheme.on_secondary_container,
        ),
        (
            IconButtonVariant::FilledTonal,
            Some(true),
            scheme.on_secondary,
        ),
        (
            IconButtonVariant::Outlined,
            Some(false),
            scheme.on_surface_variant,
        ),
        (
            IconButtonVariant::Outlined,
            Some(true),
            scheme.inverse_on_surface,
        ),
        (
            IconButtonVariant::Standard,
            Some(false),
            scheme.on_surface_variant,
        ),
        (IconButtonVariant::Standard, Some(true), scheme.primary),
    ] {
        settings.with_mut(|settings| {
            settings.variant = variant;
            settings.selected = selected;
        });
        renderer.render_frames(3);
        assert_icon_color(&renderer, content);
    }
    settings.with_mut(|settings| {
        settings.selected = None;
        settings.color_style = IconButtonColorStyle::Inherited;
    });
    renderer.render_frames(3);
    assert_icon_color(&renderer, Color::RED);
    settings.with_mut(|settings| settings.inherited = Color::BLUE);
    renderer.render_frames(3);
    assert_icon_color(&renderer, Color::BLUE);
    settings.with_mut(|settings| {
        settings.variant = IconButtonVariant::Filled;
        settings.color = Some(scheme.secondary_container);
    });
    renderer.render_frames(3);
    assert_icon_color(&renderer, scheme.on_secondary_container);
    settings.with_mut(|settings| settings.content = Some(Color::GREEN));
    renderer.render_frames(3);
    assert_icon_color(&renderer, Color::GREEN);
    settings.set(Settings::default());
    renderer.render_frames(90);
    let rect = bounds(&tree, 0);
    click(
        &mut renderer,
        PxPosition::from_f32_xy((rect.x0 + 1.0) as f32, center(rect).y),
    );
    assert_eq!(
        CLICKS.get().unwrap().get(),
        1,
        "touch padding is interactive"
    );
    settings.with_mut(|settings| {
        settings.enabled = false;
        settings.selected = Some(true);
    });
    renderer.render_frames(60);
    let disabled = renderer.accessibility_snapshot().unwrap();
    for index in 0..15 {
        assert_eq!(bounds(&disabled, index), bounds(&tree, index));
        let node = &node(&disabled, index).1;
        assert!(node.is_disabled());
        assert_eq!(node.toggled(), Some(Toggled::True));
        assert!(!node.supports_action(Action::Click));
    }
    click(&mut renderer, center(rect));
    assert_eq!(CLICKS.get().unwrap().get(), 1);
    settings.with_mut(|settings| {
        settings.enabled = true;
        settings.selected = Some(false);
        settings.shape = IconButtonShape::Square;
    });
    renderer.render_frames(90);
    let before = renderer.accessibility_snapshot().unwrap();
    let identity = node(&before, 7).0;
    let point = center(bounds(&before, 7));
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(30);
    assert!(
        INTERACTIONS
            .with(|states| states.borrow()[7].unwrap())
            .with(InteractionState::is_pressed)
    );
    assert_eq!(CLICKS.get().unwrap().get(), 1);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_move(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy((590.0 * scale) as f32, (590.0 * scale) as f32),
    );
    renderer.render_frames(90);
    let after = renderer.accessibility_snapshot().unwrap();
    assert_eq!(node(&after, 7).0, identity);
    assert_eq!(node(&after, 7).1.role(), Role::Button);
    assert_eq!(node(&after, 7).1.toggled(), Some(Toggled::True));
    assert_eq!(CLICKS.get().unwrap().get(), 2);
    settings.with_mut(|settings| {
        settings.selected = None;
        settings.variant = IconButtonVariant::Outlined;
    });
    renderer.render_frames(90);
    let ordinary = renderer.accessibility_snapshot().unwrap();
    assert_eq!(node(&ordinary, 7).0, identity);
    assert_eq!(node(&ordinary, 7).1.toggled(), None);
    renderer.resize(1, height);
    renderer.render_frames(3);
    renderer.resize(width, height);
    renderer.render_frames(8);
    let restored = renderer.accessibility_snapshot().unwrap();
    for index in 0..15 {
        assert_eq!(bounds(&restored, index), bounds(&tree, index));
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn expressive_geometry_and_toggle_input_msaa1() {
    check_geometry(1, 1.0);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn expressive_geometry_and_toggle_input_msaa4() {
    check_geometry(4, 1.0);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn expressive_geometry_and_toggle_input_at_fractional_density() {
    check_geometry(4, 1.5);
}

#[test]
fn controlled_selection_resolves_each_variant_and_disabled_outline() {
    let scheme = MaterialTheme::default().color_scheme;
    for (variant, off_container, off_content, on_container, on_content) in [
        (
            IconButtonVariant::Standard,
            Color::TRANSPARENT,
            scheme.on_surface_variant,
            Color::TRANSPARENT,
            scheme.primary,
        ),
        (
            IconButtonVariant::Filled,
            scheme.surface_container,
            scheme.on_surface_variant,
            scheme.primary,
            scheme.on_primary,
        ),
        (
            IconButtonVariant::FilledTonal,
            scheme.secondary_container,
            scheme.on_secondary_container,
            scheme.secondary,
            scheme.on_secondary,
        ),
        (
            IconButtonVariant::Outlined,
            Color::TRANSPARENT,
            scheme.on_surface_variant,
            scheme.inverse_surface,
            scheme.inverse_on_surface,
        ),
    ] {
        let off = resolve_colors(&scheme, variant, Some(false), true, None);
        let on = resolve_colors(&scheme, variant, Some(true), true, None);
        assert_eq!((off.container, off.content), (off_container, off_content));
        assert_eq!((on.container, on.content), (on_container, on_content));
        assert_eq!(
            off.border,
            (variant == IconButtonVariant::Outlined).then_some(scheme.outline_variant)
        );
        assert_eq!(on.border, None);
        for selected in [None, Some(false), Some(true)] {
            let disabled = resolve_colors(&scheme, variant, selected, false, None);
            assert_eq!(disabled.content, scheme.on_surface.with_alpha(0.38));
            assert_eq!(
                disabled.border,
                (variant == IconButtonVariant::Outlined && selected != Some(true))
                    .then_some(scheme.outline_variant.with_alpha(0.38))
            );
        }
    }
    for variant in [IconButtonVariant::Standard, IconButtonVariant::Outlined] {
        let inherited = resolve_colors(&scheme, variant, Some(false), true, Some(Color::RED));
        assert_eq!(inherited.content, Color::RED);
        assert_eq!(
            inherited.border,
            (variant == IconButtonVariant::Outlined).then_some(Color::RED)
        );
        assert_eq!(
            resolve_colors(&scheme, variant, None, false, Some(Color::RED)).content,
            Color::RED.with_alpha(0.38)
        );
    }
}

#[test]
fn shape_motion_preserves_velocity_and_settles_without_bounce() {
    for size in SIZES {
        for shape in [IconButtonShape::Round, IconButtonShape::Square] {
            let start = IconButtonCorner::new(size, shape, false, false);
            let pressed = IconButtonCorner::new(size, shape, false, true);
            let mut motion = IconButtonMotion::new(start, 0);
            motion.retarget(pressed, 0);
            motion.advance(33_333_334);
            let (value, velocity) = (motion.radius.value(), motion.radius.velocity());
            motion.retarget(start, 33_333_334);
            assert_eq!(
                (motion.radius.value(), motion.radius.velocity()),
                (value, velocity)
            );
            for frame in 3..150 {
                motion.advance(frame * 16_666_667);
            }
            assert!(motion.is_idle());
            let checked = IconButtonCorner::new(size, shape, true, false);
            motion.retarget(checked, 2_500_000_000);
            for frame in 151..300 {
                motion.advance(frame * 16_666_667);
                assert!((0.0..=0.5).contains(&motion.fraction.value()));
                assert!((0.0..=size.square_radius().0 as f32).contains(&motion.radius.value()));
            }
            assert!(motion.is_idle());
            let Shape::RoundedRectangle { top_left, .. } = motion.shape() else {
                unreachable!()
            };
            let short_edge = size.width(IconButtonWidth::Narrow).0.min(size.height().0) as f32;
            let expected = if shape == IconButtonShape::Square {
                short_edge / 2.0
            } else {
                size.square_radius().0 as f32
            };
            assert_eq!(
                top_left
                    .resolve(PxSize::new(Px(short_edge as i32), Px(200)))
                    .0,
                expected
            );
        }
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn controlled_presentation_matches_native_states_without_taking_focus() {
    crate::test_utils::run_isolated_test(
        "icon_button::tests::controlled_presentation_matches_native_states_without_taking_focus",
        || {
            use tessera_ui::accesskit::{Action, ActionRequest, TreeId};

            let mut renderer = HeadlessRenderer::new(
                Box::new(fixture),
                vec![Box::new(crate::TesseraComponents)],
                HeadlessConfig {
                    width: 600,
                    height: 600,
                    frame_time: std::time::Duration::from_millis(100),
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(20);
            let settings = SETTINGS.get().unwrap();
            settings.with_mut(|settings| settings.focus_reference = true);
            renderer.render_frames(20);
            let tree = renderer.accessibility_snapshot().unwrap();
            let id = tree
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some("Icon action 7"))
                .unwrap()
                .0;
            let rect = bounds(&tree, 7);
            let point = PxPosition::from_f32_xy(
                ((rect.x0 + rect.x1) / 2.0) as f32,
                ((rect.y0 + rect.y1) / 2.0) as f32,
            );
            let outside = PxPosition::from_f32_xy((600 - 2) as f32, (600 - 2) as f32);
            let crop = |pixels: &[u8]| {
                let mut result = Vec::new();
                for y in rect.y0 as usize..rect.y1 as usize {
                    let start = (y * 600 + rect.x0 as usize) * 4;
                    result.extend_from_slice(&pixels[start..start + rect.width() as usize * 4]);
                }
                result
            };
            let focus_crop = |pixels: &[u8]| {
                let mut result = Vec::new();
                for y in rect.y0 as usize - 6..rect.y1 as usize + 6 {
                    let start = (y * 600 + rect.x0 as usize - 6) * 4;
                    result.extend_from_slice(
                        &pixels[start..start + (rect.width() as usize + 12) * 4],
                    );
                }
                result
            };
            let mut focus_preview = None;
            let rest_pixels = renderer.read_pixels().unwrap();
            let rest = crop(&rest_pixels);
            let mut previews = Vec::new();
            for presentation in [
                InteractionVisualState::Hover,
                InteractionVisualState::Focus,
                InteractionVisualState::Pressed,
            ] {
                settings.with_mut(|settings| settings.presentation_state = Some(presentation));
                renderer.render_frames(20);
                let preview = crop(&renderer.read_pixels().unwrap());
                assert_ne!(
                    preview, rest,
                    "presentation changes native appearance: {presentation:?}"
                );
                let live = INTERACTIONS
                    .with(|states| states.borrow()[7].unwrap())
                    .get();
                assert_eq!(live, InteractionState::new());
                assert_ne!(renderer.accessibility_snapshot().unwrap().focus, id);
                assert_eq!(CLICKS.get().unwrap().get(), 0);
                if presentation == InteractionVisualState::Focus {
                    let pixels = renderer.read_pixels().unwrap();
                    focus_preview = Some(focus_crop(&pixels));
                    let mut changed = Vec::new();
                    for y in rect.y0 as usize - 6..rect.y1 as usize + 6 {
                        for x in rect.x0 as usize - 6..rect.x1 as usize + 6 {
                            let offset = (y * 600 + x) * 4;
                            if pixels[offset..offset + 4] != rest_pixels[offset..offset + 4] {
                                changed.push((x, y));
                            }
                        }
                    }
                    let ink = (
                        changed.iter().map(|p| p.0).min().unwrap(),
                        changed.iter().map(|p| p.1).min().unwrap(),
                        changed.iter().map(|p| p.0).max().unwrap() + 1,
                        changed.iter().map(|p| p.1).max().unwrap() + 1,
                    );
                    assert_eq!(
                        ink,
                        (
                            rect.x0 as usize - 5,
                            rect.y0 as usize - 5,
                            rect.x1 as usize + 5,
                            rect.y1 as usize + 5
                        ),
                        "native focus outline extends five pixels beyond the container"
                    );
                }
                previews.push(preview);
                settings.with_mut(|settings| settings.presentation_state = None);
                renderer.render_frames(20);
                assert_eq!(
                    crop(&renderer.read_pixels().unwrap()),
                    rest,
                    "clearing retained presentation restores native rest"
                );
            }
            settings.with_mut(|settings| {
                settings.presentation_state = Some(InteractionVisualState::Pressed);
                settings.presentation_ripple = Some(RipplePresentation {
                    origin: [0.7, 0.9],
                    radius: 0.4,
                    opacity: 0.2,
                });
            });
            renderer.render_frames(20);
            let sampled = crop(&renderer.read_pixels().unwrap());
            let pixels = renderer.read_pixels().unwrap();
            let reference = FOCUS_REFERENCE.get().unwrap();
            let x = (reference.x.0 as f32 + reference.width.0 as f32 * 0.7) as usize;
            let y = (reference.y.0 as f32 + reference.height.0 as f32 * 0.9) as usize;
            assert!(
                pixels[(y * 600 + x) * 4..(y * 600 + x) * 4 + 3]
                    .iter()
                    .any(|value| *value < 240),
                "static ripple draws on a noninteractive surface with its state layer disabled"
            );

            assert_ne!(
                sampled, previews[2],
                "explicit native ripple sample changes pressed pixels"
            );
            renderer.render_frames(20);
            assert_eq!(
                crop(&renderer.read_pixels().unwrap()),
                sampled,
                "presentation ripple remains static across frame timestamps"
            );
            assert_eq!(renderer.diagnostics().paint_nodes_recorded, 0);
            settings.with_mut(|settings| settings.presentation_ripple = None);
            renderer.render_frames(20);
            assert_eq!(crop(&renderer.read_pixels().unwrap()), previews[2]);
            settings.with_mut(|settings| {
                settings.presentation_ripple = Some(RipplePresentation {
                    origin: [f32::NAN, f32::INFINITY],
                    radius: -1.0,
                    opacity: f32::NAN,
                })
            });
            renderer.render_frames(20);
            assert_eq!(
                crop(&renderer.read_pixels().unwrap()),
                previews[2],
                "invalid samples safely disable the ripple"
            );
            settings.with_mut(|settings| {
                settings.presentation_ripple = None;
                settings.presentation_state = None;
            });
            renderer.render_frames(20);
            renderer.pointer_move(MOUSE_POINTER_ID, point);
            renderer.render_frames(20);
            assert_eq!(
                crop(&renderer.read_pixels().unwrap()),
                previews[0],
                "preview hover matches actual pointer hover"
            );
            settings.with_mut(|settings| {
                settings.presentation_state = Some(InteractionVisualState::Pressed)
            });
            renderer.render_frames(20);
            assert_eq!(
                crop(&renderer.read_pixels().unwrap()),
                previews[0],
                "live hover overrides pressed presentation"
            );
            renderer.pointer_move(MOUSE_POINTER_ID, outside);
            settings.with_mut(|settings| settings.presentation_state = None);
            renderer.render_frames(20);
            assert!(renderer.dispatch_accessibility_action(ActionRequest {
                action: Action::Focus,
                target_tree: TreeId::ROOT,
                target_node: id,
                data: None,
            }));
            renderer.render_frames(20);
            assert_eq!(renderer.accessibility_snapshot().unwrap().focus, id);
            assert_eq!(
                focus_crop(&renderer.read_pixels().unwrap()),
                focus_preview.as_ref().unwrap().as_slice(),
                "preview focus matches actual native focus"
            );
            settings.with_mut(|settings| {
                settings.presentation_state = Some(InteractionVisualState::Hover)
            });
            renderer.render_frames(20);
            assert_eq!(
                focus_crop(&renderer.read_pixels().unwrap()),
                focus_preview.as_ref().unwrap().as_slice(),
                "live focus overrides hover presentation"
            );
            settings.with_mut(|settings| settings.presentation_state = None);
            renderer.pointer_move(MOUSE_POINTER_ID, point);
            renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
            renderer.render_frames(20);
            assert_eq!(
                renderer.accessibility_snapshot().unwrap().focus,
                id,
                "pointer modality preserves actual focus ownership"
            );
            let pointer_pixels = renderer.read_pixels().unwrap();
            for y in rect.y0 as usize - 5..rect.y1 as usize + 5 {
                for x in rect.x0 as usize - 5..rect.x1 as usize + 5 {
                    if x >= rect.x0 as usize
                        && x < rect.x1 as usize
                        && y >= rect.y0 as usize
                        && y < rect.y1 as usize
                    {
                        continue;
                    }
                    let offset = (y * 600 + x) * 4;
                    assert_eq!(
                        &pointer_pixels[offset..offset + 4],
                        &rest_pixels[offset..offset + 4],
                        "actual pointer press hides only the outside focus indicator"
                    );
                }
            }
            renderer.pointer_release(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
            renderer.pointer_move(MOUSE_POINTER_ID, outside);
            renderer.render_frames(20);
            assert_eq!(renderer.accessibility_snapshot().unwrap().focus, id);
            assert!(renderer.dispatch_accessibility_action(ActionRequest {
                action: Action::Focus,
                target_tree: TreeId::ROOT,
                target_node: id,
                data: None,
            }));
            renderer.render_frames(20);
            assert_eq!(
                focus_crop(&renderer.read_pixels().unwrap()),
                focus_preview.as_ref().unwrap().as_slice(),
                "explicit accessibility focus restores the ring on the same focused handle"
            );
            renderer.pointer_move(MOUSE_POINTER_ID, point);
            renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
            renderer.render_frames(20);
            assert!(
                INTERACTIONS
                    .with(|states| states.borrow()[7].unwrap())
                    .with(InteractionState::is_pressed)
            );
            tessera_ui::FocusManager::current().clear_focus(true);
            renderer.render_frames(20);
            let held = INTERACTIONS
                .with(|states| states.borrow()[7].unwrap())
                .get();
            assert!(
                held.is_pressed(),
                "clearing keyboard focus preserves the actual held pointer press"
            );
            assert!(!held.is_focused());
            let mask = |pixels: &[u8]| {
                pixels
                    .chunks_exact(4)
                    .map(|pixel| pixel[..3] != [255, 255, 255])
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                mask(&crop(&renderer.read_pixels().unwrap())),
                mask(&previews[2]),
                "pressed presentation uses the actual native pressed silhouette"
            );
            renderer.pointer_release(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
            renderer.pointer_move(MOUSE_POINTER_ID, outside);
            tessera_ui::FocusManager::current().clear_focus(true);
            renderer.render_frames(20);
            assert_eq!(CLICKS.get().unwrap().get(), 0);
            settings.with_mut(|settings| {
                settings.enabled = false;
                settings.presentation_state = Some(InteractionVisualState::Pressed);
            });
            renderer.render_frames(20);
            let disabled = renderer.read_pixels().unwrap();
            settings.with_mut(|settings| settings.presentation_state = None);
            renderer.render_frames(20);
            assert_eq!(
                renderer.read_pixels().unwrap(),
                disabled,
                "disabled styling overrides presentation"
            );
        },
    );
}
