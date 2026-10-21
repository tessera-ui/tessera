use std::cell::Cell;

use tessera_ui::{
    FocusManager, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, State,
    accesskit::{Rect, Role},
    remember,
};

use crate::{icon_button::IconButtonColorStyle, painter::try_painter, theme::material_theme};

use super::*;

#[derive(Clone)]
struct Settings {
    variant: TopAppBarVariant,
    alignment: AppBarTitleAlignment,
    subtitle: Option<String>,
    navigation: bool,
    actions: usize,
    title: String,
    changed_colors: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            variant: TopAppBarVariant::Small,
            alignment: AppBarTitleAlignment::Start,
            subtitle: None,
            navigation: true,
            actions: 2,
            title: "Inbox".to_owned(),
            changed_colors: false,
        }
    }
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
    static CLICKS: Cell<Option<State<usize>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let settings = remember(Settings::default);
    let clicks = remember(|| 0usize);
    let painter = remember(|| {
        try_painter(br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M0 0H24V24H0Z" fill="#000"/></svg>"##.as_slice()).expect("icon")
    });
    SETTINGS.set(Some(settings));
    CLICKS.set(Some(clicks));
    material_theme().child(move || {
        surface()
            .style(Color::WHITE.into())
            .modifier(Modifier::new().fill_max_size())
            .child(move || {
                let values = settings.get();
                let mut bar = top_app_bar()
                    .variant(values.variant)
                    .title_alignment(values.alignment)
                    .title(values.title)
                    .subtitle_optional(values.subtitle)
                    .container_color(Color::WHITE)
                    .content_color(Color::BLACK)
                    .navigation_icon_color(if values.changed_colors {
                        Color::RED
                    } else {
                        Color::BLUE
                    })
                    .action_icon_color(if values.changed_colors {
                        Color::BLUE
                    } else {
                        Color::GREEN
                    });
                if values.navigation {
                    bar = bar.navigation_icon(move || {
                        icon_button()
                            .standard()
                            .color_style(IconButtonColorStyle::Inherited)
                            .icon(painter.get())
                            .accessibility_label("Back")
                            .on_click(move || clicks.set(clicks.get() + 1));
                    });
                }
                for index in 0..values.actions {
                    bar = bar.action(move || {
                        icon_button()
                            .standard()
                            .color_style(IconButtonColorStyle::Inherited)
                            .icon(painter.get())
                            .accessibility_label(format!("Action {index}"))
                            .on_click(move || clicks.set(clicks.get() + 1));
                    });
                }
            });
    });
}

fn bounds(renderer: &HeadlessRenderer, label: &str, role: Role) -> Rect {
    let nodes = renderer.accessibility_snapshot().expect("tree").nodes;
    nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label) && node.role() == role)
        .unwrap_or_else(|| {
            panic!(
                "missing {label}; nodes: {:?}",
                nodes
                    .iter()
                    .map(|(_, node)| (node.role(), node.label(), node.bounds()))
                    .collect::<Vec<_>>()
            )
        })
        .1
        .bounds()
        .expect("bounds")
}

fn toolbar(renderer: &HeadlessRenderer) -> Rect {
    renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::Toolbar)
        .expect("toolbar")
        .1
        .bounds()
        .expect("bounds")
}

fn near(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 1.0, "{actual} != {expected}");
}

fn icon_color(renderer: &HeadlessRenderer, label: &str, expected: [u8; 3]) {
    let bounds = bounds(renderer, label, Role::Button);
    let x = ((bounds.x0 + bounds.x1) / 2.0) as usize;
    let y = ((bounds.y0 + bounds.y1) / 2.0) as usize;
    let offset = (y * renderer.physical_size().width as usize + x) * 4;
    let pixels = renderer.read_pixels().expect("pixels");
    assert_eq!(&pixels[offset..offset + 3], &expected);
}

fn click(renderer: &mut HeadlessRenderer, label: &str) {
    let bounds = bounds(renderer, label, Role::Button);
    let position = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.step_frame();
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(4);
}

fn check_bar(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(|| fixture()),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 540,
            height: 600,
            scale_factor: 1.5,
            sample_count: samples,
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(8);
    let settings = SETTINGS.get().expect("settings");
    let clicks = CLICKS.get().expect("clicks");
    let published = renderer.accessibility_snapshot().expect("tree");
    let toolbar_node = &published
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Toolbar)
        .expect("toolbar")
        .1;
    let order = toolbar_node
        .children()
        .iter()
        .map(|id| {
            published
                .nodes
                .iter()
                .find(|(node_id, _)| node_id == id)
                .expect("published child")
                .1
                .label()
                .expect("child label")
        })
        .collect::<Vec<_>>();
    assert_eq!(order, ["Back", "Inbox", "Action 0", "Action 1"]);
    near(toolbar(&renderer).y1, 96.0);
    near(bounds(&renderer, "Back", Role::Button).x0, 6.0);
    near(bounds(&renderer, "Back", Role::Button).x1, 78.0);
    near(bounds(&renderer, "Inbox", Role::Label).x0, 84.0);
    near(bounds(&renderer, "Action 1", Role::Button).x1, 534.0);
    icon_color(&renderer, "Back", [0, 0, 255]);
    icon_color(&renderer, "Action 0", [0, 255, 0]);

    settings.with_mut(|values| values.alignment = AppBarTitleAlignment::Center);
    renderer.render_frames(5);
    let title = bounds(&renderer, "Inbox", Role::Label);
    near((title.x0 + title.x1) / 2.0, 270.0);
    settings.with_mut(|values| values.actions = 4);
    renderer.render_frames(5);
    let title = bounds(&renderer, "Inbox", Role::Label);
    assert!(title.x0 >= bounds(&renderer, "Back", Role::Button).x1);
    assert!(title.x1 <= bounds(&renderer, "Action 0", Role::Button).x0);

    for (variant, title_height, subtitle_height, without, with) in [
        (TopAppBarVariant::Small, 42.0, 24.0, 96.0, 96.0),
        (TopAppBarVariant::MediumFlexible, 54.0, 30.0, 168.0, 204.0),
        (TopAppBarVariant::LargeFlexible, 66.0, 36.0, 180.0, 228.0),
    ] {
        settings.with_mut(|values| {
            values.variant = variant;
            values.subtitle = None;
            values.actions = 2;
        });
        renderer.render_frames(5);
        near(toolbar(&renderer).y1, without);
        let title = bounds(&renderer, "Inbox", Role::Label);
        near(title.y1 - title.y0, title_height);
        if variant != TopAppBarVariant::Small {
            near(title.y0, 96.0);
        }
        settings.with_mut(|values| values.subtitle = Some("3 unread".to_owned()));
        renderer.render_frames(5);
        near(toolbar(&renderer).y1, with);
        let title = bounds(&renderer, "Inbox", Role::Label);
        let subtitle = bounds(&renderer, "3 unread", Role::Label);
        near(subtitle.y1 - subtitle.y0, subtitle_height);
        near(subtitle.y0, title.y1);
        near(
            (title.x0 + title.x1) / 2.0,
            (subtitle.x0 + subtitle.x1) / 2.0,
        );
    }
    settings.with_mut(|values| values.changed_colors = true);
    renderer.render_frames(5);
    icon_color(&renderer, "Back", [255, 0, 0]);
    icon_color(&renderer, "Action 0", [0, 0, 255]);
    click(&mut renderer, "Back");
    click(&mut renderer, "Action 1");
    assert_eq!(clicks.get(), 2);

    settings.with_mut(|values| {
        values.navigation = false;
        values.alignment = AppBarTitleAlignment::Start;
        values.subtitle = None;
        values.variant = TopAppBarVariant::Small;
    });
    renderer.render_frames(5);
    near(bounds(&renderer, "Inbox", Role::Label).x0, 24.0);
    settings.with_mut(|values| {
        values.title = "A longer screen title that wraps on narrow windows".to_owned();
        values.variant = TopAppBarVariant::MediumFlexible;
        values.actions = 0;
    });
    renderer.resize(240, 600);
    renderer.render_frames(5);
    let title = bounds(
        &renderer,
        "A longer screen title that wraps on narrow windows",
        Role::Label,
    );
    assert!(title.x0 >= 24.0 && title.x1 <= 216.0);
    assert!(title.y1 - title.y0 > 54.0);
    near(toolbar(&renderer).y1 - title.y1, 18.0);

    settings.set(Settings::default());
    FocusManager::current().clear_focus(true);
    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(530.0, 590.0));
    renderer.resize(540, 600);
    renderer.render_frames(90);
    let pixels = renderer.read_pixels().expect("settled pixels");
    renderer.resize(1, 1);
    renderer.render_frames(3);
    renderer.resize(540, 600);
    renderer.render_frames(8);
    assert!(
        renderer.read_pixels().expect("restored pixels") == pixels,
        "resize restores the complete app bar"
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_app_bar_variants_geometry_colors_input_and_resize() {
    crate::test_utils::run_isolated_test(
        "app_bar::tests::gpu_app_bar_variants_geometry_colors_input_and_resize",
        || check_bar(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_app_bar_variants_geometry_colors_input_and_resize_msaa() {
    crate::test_utils::run_isolated_test(
        "app_bar::tests::gpu_app_bar_variants_geometry_colors_input_and_resize_msaa",
        || check_bar(4),
    );
}
