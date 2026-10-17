use std::cell::{Cell, RefCell};

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, State,
    accesskit::{Action, HasPopup, Node, NodeId, Rect, Role, TreeUpdate},
    remember,
};

use crate::{
    boxed::boxed,
    column::column,
    icon::icon,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    spacer::spacer,
    text::text,
    theme::material_theme,
};

use super::*;

const SIZES: [SplitButtonSize; 5] = [
    SplitButtonSize::ExtraSmall,
    SplitButtonSize::Small,
    SplitButtonSize::Medium,
    SplitButtonSize::Large,
    SplitButtonSize::ExtraLarge,
];

#[derive(Clone, Copy)]
struct Settings {
    enabled: bool,
    variant: SplitButtonVariant,
    width: Dp,
    middle_size: Option<SplitButtonSize>,
    expanded: Option<bool>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            variant: SplitButtonVariant::Filled,
            width: Dp(410.0),
            middle_size: None,
            expanded: None,
        }
    }
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
    static ACTIVATED: Cell<Option<State<Vec<usize>>>> = const { Cell::new(None) };
    static STYLES: RefCell<[Option<TextStyle>; 5]> = const { RefCell::new([None; 5]) };
}

#[tessera]
fn sized_icon(index: usize) {
    boxed()
        .modifier(Modifier::new().semantics(SemanticsArgs {
            role: Some(Role::Image),
            label: Some(format!("Split icon {index}")),
            ..Default::default()
        }))
        .children(|| {
            icon().vector(filled::ADD_SVG);
        });
}

#[tessera]
fn label(index: usize) {
    let style = use_context::<TextStyle>().unwrap().get();
    STYLES.with(|styles| styles.borrow_mut()[index] = Some(style));
    text().content("Send");
}

#[tessera]
fn fixture() {
    let settings = remember(Settings::default);
    let activated = remember(Vec::<usize>::new);
    SETTINGS.set(Some(settings));
    ACTIVATED.set(Some(activated));
    material_theme().child(move || {
        column().children(move || {
            for (index, preset) in SIZES.into_iter().enumerate() {
                let value = settings.get();
                let size = if index == 2 {
                    value.middle_size.unwrap_or(preset)
                } else {
                    preset
                };
                split_button_layout()
                    .modifier(Modifier::new().width(value.width))
                    .leading_button(move || {
                        let button = split_leading_button().enabled(settings.get().enabled);
                        let button = match settings.get().variant {
                            SplitButtonVariant::Filled => button.filled(),
                            SplitButtonVariant::Tonal => button.tonal(),
                            SplitButtonVariant::Elevated => button.elevated(),
                            SplitButtonVariant::Outlined => button.outlined(),
                        };
                        button
                            .size(size)
                            .accessibility_label(format!("Send {index}"))
                            .on_click(move || activated.with_mut(|items| items.push(index * 2)))
                            .content(move || {
                                sized_icon(index * 2);
                                spacer().modifier(
                                    Modifier::new().width(SplitButtonDefaults::icon_spacing(size)),
                                );
                                label(index);
                            });
                    })
                    .trailing_button(move || {
                        let button = split_trailing_button().enabled(settings.get().enabled);
                        let button = match settings.get().variant {
                            SplitButtonVariant::Filled => button.filled(),
                            SplitButtonVariant::Tonal => button.tonal(),
                            SplitButtonVariant::Elevated => button.elevated(),
                            SplitButtonVariant::Outlined => button.outlined(),
                        };
                        button
                            .size(size)
                            .expanded_optional(settings.get().expanded)
                            .accessibility_label(format!("More {index}"))
                            .on_click(move || {
                                activated.with_mut(|items| items.push(index * 2 + 1));
                                settings.with_mut(|settings| {
                                    if let Some(expanded) = &mut settings.expanded {
                                        *expanded = !*expanded;
                                    }
                                });
                            })
                            .content(move || {
                                sized_icon(index * 2 + 1);
                            });
                    });
                spacer().modifier(Modifier::new().height(Dp(8.0)));
            }
        });
    });
}

fn node<'a>(tree: &'a TreeUpdate, label: &str) -> &'a (NodeId, Node) {
    tree.nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .unwrap()
}

fn bounds(tree: &TreeUpdate, label: &str) -> Rect {
    node(tree, label).1.bounds().unwrap()
}

fn click(renderer: &mut HeadlessRenderer, rect: Rect) {
    let point = PxPosition::from_f32_xy(
        ((rect.x0 + rect.x1) / 2.0) as f32,
        ((rect.y0 + rect.y1) / 2.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.step_frame();
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(5);
}

fn check_sizes_and_shrink(samples: u32, scale: f64) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(|| {
            fixture();
        }),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: (750.0 * scale) as u32,
            height: (750.0 * scale) as u32,
            scale_factor: scale,
            sample_count: samples,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    let initial = renderer.accessibility_snapshot().unwrap();
    for (index, height) in [48.0, 48.0, 56.0, 96.0, 136.0].into_iter().enumerate() {
        let leading = bounds(&initial, &format!("Send {index}"));
        let trailing = bounds(&initial, &format!("More {index}"));
        assert_eq!(leading.height(), height * scale);
        assert_eq!(trailing.height(), height * scale);
        assert_eq!(trailing.x0 - leading.x1, 2.0 * scale);
        for (side, expected) in [
            (0, [20.0, 20.0, 24.0, 32.0, 40.0][index]),
            (1, [22.0, 22.0, 26.0, 38.0, 50.0][index]),
        ] {
            let icon = bounds(&initial, &format!("Split icon {}", index * 2 + side));
            assert_eq!(icon.width(), expected * scale);
            assert_eq!(icon.height(), expected * scale);
            if side == 1 {
                let offset = (icon.x0 + icon.x1 - trailing.x0 - trailing.x1) / 2.0;
                assert!((offset - [-1.0, -1.0, -2.0, -3.0, -6.0][index] * scale).abs() < 1.0e-5);
            }
        }
        assert_eq!(
            STYLES.with(|styles| styles.borrow()[index].unwrap().font_size),
            Dp([14.0, 14.0, 16.0, 24.0, 32.0][index])
        );
    }
    let settings = SETTINGS.get().unwrap();
    for width in [360.0, 200.0, 100.0, 72.0, 1.0] {
        settings.with_mut(|settings| settings.width = Dp(width));
        renderer.render_frames(4);
        let tree = renderer.accessibility_snapshot().unwrap();
        for (index, expected_width) in [48.0, 48.0, 56.0, 96.0, 136.0].into_iter().enumerate() {
            let leading = bounds(&tree, &format!("Send {index}"));
            let trailing = bounds(&tree, &format!("More {index}"));
            let max_width = (width * scale).trunc();
            assert_eq!(trailing.width(), (expected_width * scale).min(max_width));
            assert!(leading.x1 <= trailing.x0 && trailing.x1 <= max_width);
            assert_eq!(leading.height(), trailing.height());
            assert_eq!(leading.y0, trailing.y0);
        }
    }
    settings.with_mut(|settings| settings.width = Dp(200.0));
    renderer.render_frames(4);
    let narrow = renderer.accessibility_snapshot().unwrap();
    click(&mut renderer, bounds(&narrow, "Send 0"));
    click(&mut renderer, bounds(&narrow, "More 0"));
    assert_eq!(ACTIVATED.get().unwrap().get(), [0, 1]);
    for variant in [
        SplitButtonVariant::Filled,
        SplitButtonVariant::Tonal,
        SplitButtonVariant::Elevated,
        SplitButtonVariant::Outlined,
    ] {
        settings.with_mut(|settings| {
            settings.enabled = false;
            settings.variant = variant;
        });
        renderer.render_frames(8);
        let disabled = renderer.accessibility_snapshot().unwrap();
        for index in 0..5 {
            for name in [format!("Send {index}"), format!("More {index}")] {
                let node = &node(&disabled, &name).1;
                assert!(node.is_disabled());
                assert!(!node.supports_action(Action::Focus));
                assert!(!node.supports_action(Action::Click));
            }
        }
        click(&mut renderer, bounds(&disabled, "More 0"));
        assert_eq!(
            ACTIVATED.get().unwrap().get(),
            [0, 1],
            "visual presets must not re-enable input"
        );
    }
    settings.set(Settings {
        middle_size: Some(SplitButtonSize::ExtraLarge),
        ..Default::default()
    });
    renderer.render_frames(8);
    let changed = renderer.accessibility_snapshot().unwrap();
    assert_eq!(bounds(&changed, "Split icon 4").width(), 40.0 * scale);
    assert_eq!(bounds(&changed, "Split icon 5").width(), 50.0 * scale);
    assert_eq!(
        STYLES.with(|styles| styles.borrow()[2].unwrap().font_size),
        Dp(32.0)
    );
    settings.set(Settings {
        expanded: Some(false),
        ..Default::default()
    });
    renderer.render_frames(90);
    let before = renderer.accessibility_snapshot().unwrap();
    let identity = node(&before, "More 2").0;
    assert_eq!(node(&before, "More 2").1.is_expanded(), Some(false));
    assert_eq!(node(&before, "More 2").1.has_popup(), Some(HasPopup::Menu));
    click(&mut renderer, bounds(&before, "More 2"));
    renderer.render_frames(90);
    let expanded = renderer.accessibility_snapshot().unwrap();
    assert_eq!(node(&expanded, "More 2").0, identity);
    assert_eq!(node(&expanded, "More 2").1.is_expanded(), Some(true));
    let action = bounds(&expanded, "More 2");
    let icon = bounds(&expanded, "Split icon 5");
    assert_eq!((icon.x0 + icon.x1) / 2.0, (action.x0 + action.x1) / 2.0);
    settings.with_mut(|settings| settings.enabled = false);
    renderer.render_frames(90);
    let disabled = renderer.accessibility_snapshot().unwrap();
    assert_eq!(node(&disabled, "More 2").1.is_expanded(), Some(true));
    assert!(node(&disabled, "More 2").1.is_disabled());
    settings.with_mut(|settings| {
        settings.enabled = true;
        settings.expanded = None;
    });
    renderer.render_frames(90);
    let ordinary = renderer.accessibility_snapshot().unwrap();
    assert_eq!(node(&ordinary, "More 2").0, identity);
    assert_eq!(node(&ordinary, "More 2").1.is_expanded(), None);
    assert_eq!(node(&ordinary, "More 2").1.has_popup(), None);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn expressive_sizes_and_trailing_width_priority_msaa1() {
    check_sizes_and_shrink(1, 1.0);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn expressive_sizes_and_trailing_width_priority_msaa4() {
    check_sizes_and_shrink(4, 1.0);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn expressive_sizes_and_trailing_width_priority_fractional_density() {
    check_sizes_and_shrink(4, 1.5);
}
