//! Tab selection, focus, and Material geometry.
//!
//! ## Usage
//!
//! Verify tab strips used to switch related views.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    Color, Dp, FocusDirection, FocusManager, HeadlessConfig, HeadlessRenderer, Modifier, State,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, TreeId},
    remember, tessera, use_context,
};

use crate::{
    button::button,
    column::column,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    text::text,
    theme::{MaterialTheme, material_theme},
};

use super::{TabActivationMode, TabsCompositionContext, TabsController, TabsVariant, tab, tabs};

#[derive(Clone, Copy, PartialEq)]
struct Config {
    variant: TabsVariant,
    mode: TabActivationMode,
    middle_enabled: bool,
    icons: bool,
    count: usize,
    scrollable: bool,
    controlled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            variant: TabsVariant::Primary,
            mode: TabActivationMode::Manual,
            middle_enabled: true,
            icons: false,
            count: 3,
            scrollable: false,
            controlled: false,
        }
    }
}

thread_local! {
    static SELECTED: Cell<Option<State<usize>>> = const { Cell::new(None) };
    static CONFIG: Cell<Option<State<Config>>> = const { Cell::new(None) };
    static CALLS: Cell<Option<State<usize>>> = const { Cell::new(None) };
    static CONTROLLER: Cell<Option<State<TabsController>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let selected = remember(|| 0usize);
    let config = remember(Config::default);
    let calls = remember(|| 0usize);
    SELECTED.set(Some(selected));
    CONFIG.set(Some(config));
    CALLS.set(Some(calls));
    material_theme()
        .theme(|| {
            let mut theme = MaterialTheme::default();
            theme.color_scheme.primary = Color::RED;
            theme.color_scheme.on_surface = Color::BLUE;
            theme.color_scheme.outline_variant = Color::GREEN;
            theme.color_scheme.surface = Color::WHITE;
            theme
        })
        .child(move || {
            column().children(move || {
                button()
                    .accessibility_label("Before tabs")
                    .on_click(|| {})
                    .child(|| {
                        text().content("Before tabs");
                    });
                tabs()
                    .modifier(Modifier::new().width(Dp(301.0)).semantics(SemanticsArgs {
                        label: Some("Transport views".to_owned()),
                        ..Default::default()
                    }))
                    .selected(selected)
                    .variant(config.get().variant)
                    .activation_mode(config.get().mode)
                    .scrollable(config.get().scrollable)
                    .content(move || {
                        CONTROLLER.set(Some(
                            use_context::<TabsCompositionContext>()
                                .unwrap()
                                .get()
                                .controller,
                        ));
                        for (index, label) in ["Flights", "Hotels", "Trains", "Buses", "Ferries"]
                            .into_iter()
                            .take(config.get().count)
                            .enumerate()
                        {
                            let mut item = tab()
                                .label(label)
                                .selected_optional(if config.get().controlled {
                                    Some(selected.get() == index)
                                } else {
                                    None
                                })
                                .enabled(index != 1 || config.get().middle_enabled)
                                .on_click(move || calls.with_mut(|calls| *calls += 1));
                            if config.get().icons {
                                item = item.icon(filled::ADD_SVG);
                            }
                        }
                    });
                button()
                    .accessibility_label("After tabs")
                    .on_click(|| {})
                    .child(|| {
                        text().content("After tabs");
                    });
            });
        });
}

fn action(renderer: &mut HeadlessRenderer, id: NodeId, action: Action) {
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: id,
        data: None,
    }));
    renderer.render_frames(12);
}

fn focused(renderer: &HeadlessRenderer) -> Option<String> {
    let tree = renderer.accessibility_snapshot().expect("tree");
    tree.nodes
        .iter()
        .find(|(id, _)| *id == tree.focus)
        .and_then(|(_, node)| node.label().map(str::to_owned))
}

fn move_focus(renderer: &mut HeadlessRenderer, direction: FocusDirection, label: &str) {
    FocusManager::current().request_move_focus(direction);
    renderer.render_frames(12);
    assert_eq!(focused(renderer).as_deref(), Some(label));
}

fn save(renderer: &mut HeadlessRenderer, name: &str, samples: u32) {
    if let Some(directory) = std::env::var_os("TESSERA_TABS_QA_DIR") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("QA directory");
        let size = renderer.physical_size();
        let pixels = renderer.read_pixels().expect("pixels");
        tessera_ui::headless::write_png_file(
            directory.join(format!("{name}-msaa{samples}.png")),
            size.width,
            size.height,
            &pixels,
        )
        .expect("QA image");
    }
}

fn red_pixels_on_row(pixels: &[u8], width: u32, y: u32, x0: u32, x1: u32) -> usize {
    (x0..x1)
        .filter(|x| {
            let offset = ((y * width + x) * 4) as usize;
            let red = i16::from(pixels[offset]);
            red > 180
                && red - i16::from(pixels[offset + 1]) > 60
                && red - i16::from(pixels[offset + 2]) > 60
        })
        .count()
}

fn check_indicator(renderer: &mut HeadlessRenderer, label: &str, height: u32, rounded_top: bool) {
    let bounds = control(renderer, label, Role::Tab)
        .1
        .bounds()
        .expect("indicator tab");
    let size = renderer.physical_size();
    let pixels = renderer.read_pixels().expect("indicator pixels");
    let x = ((bounds.x0 + bounds.x1) * 0.5) as u32;
    let bottom = bounds.y1 as u32;
    for y in bottom - height..bottom {
        assert_eq!(red_pixels_on_row(&pixels, size.width, y, x, x + 1), 1);
    }
    assert_eq!(
        red_pixels_on_row(&pixels, size.width, bottom - height - 1, x, x + 1),
        0
    );
    if rounded_top {
        let top = red_pixels_on_row(
            &pixels,
            size.width,
            bottom - height,
            bounds.x0 as u32,
            bounds.x1 as u32,
        );
        let base = red_pixels_on_row(
            &pixels,
            size.width,
            bottom - 1,
            bounds.x0 as u32,
            bounds.x1 as u32,
        );
        assert!(
            base > top,
            "the top corners round while the bottom edge stays square"
        );
    }
}

fn control(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
    let nodes = renderer.accessibility_snapshot().expect("tree").nodes;
    nodes
        .iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .cloned()
        .unwrap_or_else(|| {
            panic!(
                "missing {label} ({role:?}): {:?}",
                nodes
                    .iter()
                    .map(|(_, node)| (node.role(), node.label()))
                    .collect::<Vec<_>>()
            )
        })
}

fn check_tabs(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 900,
            height: 600,
            scale_factor: 1.5,
            frame_time: if samples == 4 {
                Duration::from_millis(50)
            } else {
                Duration::from_nanos(16_666_667)
            },
            sample_count: samples,
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(12);
    assert_eq!(
        control(&renderer, "Flights", Role::Tab).1.is_selected(),
        Some(true)
    );
    assert_eq!(
        control(&renderer, "Hotels", Role::Tab).1.is_selected(),
        Some(false)
    );
    let selected = SELECTED.get().expect("selection");
    let config = CONFIG.get().expect("config");
    let calls = CALLS.get().expect("activation count");
    let group = control(&renderer, "Transport views", Role::TabList)
        .1
        .bounds()
        .expect("group bounds");
    assert_eq!(group.x1 - group.x0, 451.0);
    let mut edge = group.x0;
    for (label, width) in [("Flights", 151.0), ("Hotels", 150.0), ("Trains", 150.0)] {
        let (_, node) = control(&renderer, label, Role::Tab);
        let bounds = node.bounds().expect("tab bounds");
        assert_eq!(bounds.x0, edge);
        assert_eq!(bounds.x1 - bounds.x0, width);
        assert_eq!(bounds.y1 - bounds.y0, 72.0);
        assert!(node.supports_action(Action::Focus));
        assert!(node.supports_action(Action::Click));
        assert_eq!(node.toggled(), None);
        edge = bounds.x1;
    }
    assert_eq!(edge, group.x1);
    check_indicator(&mut renderer, "Flights", 4, true);
    save(&mut renderer, "primary", samples);
    let initial = renderer.read_pixels().expect("initial pixels");
    let before = control(&renderer, "Before tabs", Role::Button).0;
    action(&mut renderer, before, Action::Focus);
    move_focus(&mut renderer, FocusDirection::Next, "Flights");
    move_focus(&mut renderer, FocusDirection::Next, "After tabs");
    move_focus(&mut renderer, FocusDirection::Previous, "Flights");
    let hotels = control(&renderer, "Hotels", Role::Tab).0;
    action(&mut renderer, hotels, Action::Focus);
    assert_eq!(selected.get(), 0);
    assert_eq!(calls.get(), 0);
    assert_eq!(focused(&renderer).as_deref(), Some("Hotels"));
    assert_ne!(renderer.read_pixels().expect("focus pixels"), initial);
    move_focus(&mut renderer, FocusDirection::Next, "After tabs");
    move_focus(&mut renderer, FocusDirection::Previous, "Flights");
    action(&mut renderer, hotels, Action::Click);
    assert_eq!(selected.get(), 1);
    assert_eq!(calls.get(), 1);
    assert_eq!(control(&renderer, "Hotels", Role::Tab).0, hotels);
    assert_eq!(
        control(&renderer, "Hotels", Role::Tab).1.is_selected(),
        Some(true)
    );
    renderer.render_frames(90);
    check_indicator(&mut renderer, "Hotels", 4, true);
    config.with_mut(|config| {
        config.icons = true;
        config.variant = TabsVariant::Secondary;
    });
    renderer.render_frames(90);
    assert_eq!(control(&renderer, "Hotels", Role::Tab).0, hotels);
    let bounds = control(&renderer, "Hotels", Role::Tab)
        .1
        .bounds()
        .expect("icon bounds");
    assert_eq!(bounds.y1 - bounds.y0, 96.0);
    check_indicator(&mut renderer, "Hotels", 3, false);
    let pixels = renderer.read_pixels().expect("secondary label pixels");
    let size = renderer.physical_size();
    assert!(
        (bounds.y0 as u32..bounds.y1 as u32 - 3).any(|y| {
            (bounds.x0 as u32..bounds.x1 as u32).any(|x| {
                let offset = ((y * size.width + x) * 4) as usize;
                pixels[offset] < 20 && pixels[offset + 1] < 20 && pixels[offset + 2] > 240
            })
        }),
        "secondary active labels use on-surface rather than primary"
    );
    save(&mut renderer, "secondary-icons", samples);
    config.with_mut(|config| config.middle_enabled = false);
    renderer.render_frames(12);
    let node = control(&renderer, "Hotels", Role::Tab).1;
    assert!(node.is_disabled());
    assert!(!node.supports_action(Action::Focus));
    assert_eq!(calls.get(), 1);
    action(&mut renderer, before, Action::Focus);
    move_focus(&mut renderer, FocusDirection::Next, "Flights");
    move_focus(&mut renderer, FocusDirection::Next, "After tabs");
    config.with_mut(|config| {
        config.middle_enabled = true;
        config.count = 5;
        config.scrollable = true;
    });
    renderer.render_frames(90);
    let ferries = control(&renderer, "Ferries", Role::Tab).0;
    action(&mut renderer, ferries, Action::Focus);
    renderer.render_frames(90);
    assert_eq!(focused(&renderer).as_deref(), Some("Ferries"));
    let bounds = control(&renderer, "Ferries", Role::Tab)
        .1
        .bounds()
        .expect("revealed tab");
    save(&mut renderer, "scrollable-focus", samples);
    assert!(
        bounds.x0 >= 0.0 && bounds.x1 <= 451.0,
        "bounds={bounds:?}, focus={:?}, controller={:?}",
        focused(&renderer),
        CONTROLLER.get().unwrap().with(|controller| (
            controller.focused_requester,
            controller.tab_row_scroll_user_overridden,
            controller.tab_row_scroll_offset,
            controller
                .targets
                .iter()
                .map(|target| target.requester)
                .collect::<Vec<_>>()
        ))
    );
    assert_eq!(selected.get(), 1);
    save(&mut renderer, "scrollable-focus", samples);
    config.with_mut(|config| config.count = 3);
    renderer.render_frames(20);
    assert_eq!(focused(&renderer).as_deref(), Some("Hotels"));
    assert_eq!(calls.get(), 1);
    renderer.resize(180, 600);
    config.with_mut(|config| {
        config.scrollable = false;
        config.icons = false;
    });
    renderer.render_frames(90);
    let node = control(&renderer, "Hotels", Role::Tab).1;
    let bounds = node.bounds().expect("narrow target");
    assert!(bounds.x1 - bounds.x0 >= 72.0);
    assert!(bounds.x0 >= 0.0 && bounds.x1 <= 180.0);
    assert_eq!(selected.get(), 1);
    save(&mut renderer, "narrow", samples);
    config.with_mut(|config| config.controlled = true);
    renderer.render_frames(12);
    let flights = control(&renderer, "Flights", Role::Tab).0;
    action(&mut renderer, flights, Action::Click);
    assert_eq!(
        selected.get(),
        1,
        "controlled tabs leave selection to the caller"
    );
    assert_eq!(calls.get(), 2);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_tabs_semantics_focus_and_geometry_msaa() {
    crate::test_utils::run_isolated_test(
        "tabs::accessibility_tests::gpu_tabs_semantics_focus_and_geometry_msaa",
        || check_tabs(4),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_tabs_semantics_focus_and_geometry() {
    crate::test_utils::run_isolated_test(
        "tabs::accessibility_tests::gpu_tabs_semantics_focus_and_geometry",
        || check_tabs(1),
    );
}
