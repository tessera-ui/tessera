use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    app_bar::top_app_bar,
    boxed::boxed,
    button::button,
    column::column,
    icon::icon,
    icon_button::icon_button,
    material_icons::{filled, outlined},
    modifier::ModifierExt as _,
    navigation_drawer::navigation_drawer_item,
    row::row,
    scaffold::scaffold,
    side_sheet::{SideSheetController, modal_side_sheet_provider},
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_i18n::{I18n, langid};
use tessera_shard::{RouterController, RouterDestination, shard_home};
use tessera_ui::{
    Asset, Color, Dp, FrameNanosControl, Modifier, RenderSlotWith, State, WindowInsets,
    current_frame_nanos, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    pages::{
        custom_shader::CustomShaderPageDestination,
        glass_components::GlassComponentsPageDestination, home::HomePageDestination,
        material_components::MaterialComponentsPageDestination,
    },
    res::I18N,
};

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct AppLiveSessions {
    pub drawer: crate::pages::LiveDrawerSession,
    pub rail: crate::pages::LiveRailSession,
    pub side_sheet: crate::pages::LiveSideSheetSession,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct AppNavigationHandles {
    pub router: State<RouterController>,
    pub material: State<Option<usize>>,
    pub directory: State<SideSheetController>,
    pub sessions: Option<AppLiveSessions>,
}

#[tessera]
pub(crate) fn app(
    navigation: Option<AppNavigationHandles>,
    on_navigation_ready: Option<tessera_ui::CallbackWith<AppNavigationHandles>>,
) {
    material_theme().child(move || {
        let handles = if let Some(navigation) = navigation {
            navigation
        } else {
            AppNavigationHandles {
                directory: remember(|| SideSheetController::new(true)),
                router: remember(|| RouterController::with_root(HomePageDestination {})),
                material: remember(|| None::<usize>),
                sessions: None,
            }
        };
        let side_sheet_controller = handles.directory;
        let nav_controller = handles.router;
        let selected = handles.material;
        let content = RenderSlotWith::new(move |sessions: AppLiveSessions| {
            let background = use_context::<MaterialTheme>()
                .unwrap()
                .get()
                .color_scheme
                .surface;
            if let Some(on_ready) = on_navigation_ready {
                on_ready.call(AppNavigationHandles {
                    sessions: Some(sessions),
                    ..handles
                });
            }
            scaffold()
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .background(background)
                        .window_insets_padding(WindowInsets::SafeDrawing),
                )
                .top_bar(move || {
                    let _app_bar = top_app_bar()
                        .title(
                            I18N.text(
                                if nav_controller
                                    .with(|c| c.current_is::<MaterialComponentsPageDestination>())
                                {
                                    "Material Design 3"
                                } else {
                                    "ui-examples"
                                },
                            ),
                        )
                        .navigation_icon(move || {
                            navigation_button(
                                selected,
                                side_sheet_controller,
                                nav_controller
                                    .with(|c| c.current_is::<MaterialComponentsPageDestination>()),
                            );
                        });
                    if let Some(locale) = I18n::try_current() {
                        _app_bar.action(move || {
                            button()
                                .text()
                                .accessibility_label(
                                    I18N.format(
                                        "ui-switch-language",
                                        [(
                                            "language",
                                            I18N.text(if locale.locale() == langid!("zh-CN") {
                                                "ui-english"
                                            } else {
                                                "ui-language-chinese"
                                            })
                                            .into(),
                                        )],
                                    ),
                                )
                                .on_click(move || {
                                    let chinese = locale.locale() == langid!("zh-CN");
                                    locale.set_locale(if chinese {
                                        langid!("en")
                                    } else {
                                        langid!("zh-CN")
                                    });
                                })
                                .child(move || {
                                    text(if locale.locale() == langid!("zh-CN") {
                                        "English"
                                    } else {
                                        "简体中文"
                                    });
                                });
                        });
                    }
                    #[cfg(not(any(
                        target_family = "wasm",
                        target_os = "android",
                        target_os = "ios"
                    )))]
                    {
                        let _app_bar =
                            _app_bar.title_area_modifier(Modifier::new().window_drag_region());
                        #[cfg(not(target_os = "macos"))]
                        _app_bar
                            .window_control_minimize()
                            .window_control_toggle_maximize()
                            .window_control_close();
                    }
                })
                .content(move || {
                    surface()
                        .modifier(Modifier::new().fill_max_size())
                        .child(move || {
                            modal_side_sheet_provider()
                                .controller(side_sheet_controller)
                                .main_content(move || {
                                    shard_home().controller(nav_controller);
                                })
                                .side_sheet_content(move || {
                                    column()
                                        .modifier(Modifier::new().fill_max_width())
                                        .children(move || {
                                            drawer_header();
                                            nav_item(
                                                "ui-home",
                                                HomePageDestination {},
                                                nav_controller,
                                                side_sheet_controller,
                                            )
                                            .icon_asset(outlined::HOME_SVG)
                                            .selected_icon_asset(filled::HOME_SVG);
                                            nav_item(
                                                "ui-material-components",
                                                MaterialComponentsPageDestination {
                                                    selected: Some(selected),
                                                    sessions: Some(sessions),
                                                },
                                                nav_controller,
                                                side_sheet_controller,
                                            )
                                            .icon_asset(outlined::WIDGETS_SVG)
                                            .selected_icon_asset(filled::WIDGETS_SVG);
                                            nav_item(
                                                "ui-glass-components",
                                                GlassComponentsPageDestination {},
                                                nav_controller,
                                                side_sheet_controller,
                                            )
                                            .icon_asset(outlined::WATER_DROP_SVG)
                                            .selected_icon_asset(filled::WATER_DROP_SVG);
                                            nav_item(
                                                "ui-custom-shader",
                                                CustomShaderPageDestination {},
                                                nav_controller,
                                                side_sheet_controller,
                                            )
                                            .icon_asset(outlined::PALETTE_SVG)
                                            .selected_icon_asset(filled::PALETTE_SVG);
                                        });
                                });
                        });
                });
        });
        crate::pages::live_drawer_host(
            side_sheet_controller,
            nav_controller,
            selected,
            RenderSlotWith::new(move |drawer| {
                crate::pages::live_side_sheet_host(
                    side_sheet_controller,
                    nav_controller,
                    selected,
                    RenderSlotWith::new(move |side_sheet| {
                        crate::pages::live_rail_host(
                            side_sheet_controller,
                            nav_controller,
                            selected,
                            RenderSlotWith::new(move |rail| {
                                content.render(AppLiveSessions {
                                    drawer,
                                    rail,
                                    side_sheet,
                                });
                            }),
                        );
                    }),
                );
            }),
        );
    });
}

#[tessera]
fn drawer_header() {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let logo = if theme.color_scheme.is_dark {
        crate::res::LOGO_DARK_PNG
    } else {
        crate::res::LOGO_PNG
    };
    row()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(56.0))
                .padding_symmetric(Dp(16.0), Dp::ZERO),
        )
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            icon().painter(logo).size(Dp(28.0));
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text("Tessera")
                .style(theme.typography.title_large)
                .color(theme.color_scheme.on_surface);
        });
}

#[tessera]
fn nav_item<D: RouterDestination + Clone + PartialEq>(
    #[prop(into)] headline: String,
    destination: D,
    nav_controller: State<RouterController>,
    side_sheet_controller: State<SideSheetController>,
    icon_asset: Option<Asset>,
    selected_icon_asset: Option<Asset>,
) {
    let selected = nav_controller.with(RouterController::current_is::<D>);
    let item = navigation_drawer_item()
        .label(I18N.text(headline))
        .on_click(move || {
            nav_controller.with_mut(|c| c.replace(destination.clone()));
            side_sheet_controller.with_mut(|c| {
                if c.is_open() {
                    c.close();
                }
            });
        })
        .selected(selected);
    let asset = if selected {
        selected_icon_asset.or(icon_asset)
    } else {
        icon_asset
    };
    if let Some(asset) = asset {
        item.icon(move || {
            icon().painter(asset).size(Dp(24.0));
        });
    }
}

#[derive(Clone, Copy)]
struct NavigationFade {
    from: f32,
    value: f32,
    target: f32,
    started: u64,
}

#[tessera]
pub(crate) fn navigation_button(
    selected: State<Option<usize>>,
    drawer: State<SideSheetController>,
    material_page: bool,
) {
    let back = material_page && selected.get().is_some();
    let target = if back { 1.0 } else { 0.0 };
    let frame = current_frame_nanos();
    let fade = remember(|| NavigationFade {
        from: target,
        value: target,
        target,
        started: frame,
    });
    if fade.with(|fade| fade.target != target) {
        fade.with_mut(|fade| {
            fade.from = fade.value;
            fade.target = target;
            fade.started = frame;
        });
    }
    if fade.with(|fade| fade.value != fade.target) {
        receive_frame_nanos(move |frame| {
            fade.with_mut(|fade| {
                let t = (frame.saturating_sub(fade.started) as f32 / 200_000_000.0).clamp(0.0, 1.0);
                let eased = t * t * (3.0 - 2.0 * t);
                fade.value = fade.from + (fade.target - fade.from) * eased;
                if t == 1.0 {
                    fade.value = fade.target;
                }
            });
            if fade.with(|fade| fade.value == fade.target) {
                FrameNanosControl::Stop
            } else {
                FrameNanosControl::Continue
            }
        });
    }
    let color = use_context::<MaterialTheme>()
        .unwrap()
        .get()
        .color_scheme
        .on_surface;
    boxed()
        .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
        .alignment(Alignment::Center)
        .children(move || {
            icon_button()
                .standard()
                .icon(filled::MENU_OPEN_SVG)
                .content_color(Color::TRANSPARENT)
                .accessibility_label(I18N.text(if back {
                    "ui-back-to-components"
                } else {
                    "ui-open-navigation-drawer"
                }))
                .on_click(move || {
                    if material_page && selected.get().is_some() {
                        selected.set(None);
                    } else {
                        drawer.with_mut(|drawer| {
                            if drawer.is_open() {
                                drawer.close();
                            } else {
                                drawer.open();
                            }
                        });
                    }
                });
            icon()
                .painter(filled::MENU_OPEN_SVG)
                .tint(color.with_alpha(1.0 - fade.with(|fade| fade.value)));
            icon()
                .painter(filled::ARROW_BACK_SVG)
                .tint(color.with_alpha(fade.with(|fade| fade.value)));
        });
}

#[cfg(test)]
#[tessera]
pub(crate) fn app_fixture(
    navigation: tessera_ui::testing::TestProbe<Option<AppNavigationHandles>>,
) {
    let handles = AppNavigationHandles {
        directory: remember(|| SideSheetController::new(true)),
        router: remember(|| RouterController::with_root(HomePageDestination {})),
        material: remember(|| None::<usize>),
        sessions: None,
    };
    app()
        .navigation(handles)
        .on_navigation_ready(move |handles| navigation.set(Some(handles)));
}

#[cfg(test)]
mod tests {
    use tessera_ui::{
        accesskit::{Action, ActionRequest, Role, TreeId},
        testing::{TestProbe, layout_test, pump_layout_test_session, start_layout_test_session},
    };

    use super::*;

    #[test]
    fn language_switch_preserves_route_and_edited_control_state_in_both_directions() {
        let navigation = TestProbe::default();
        let mut session = start_layout_test_session(
            layout_test(|| app_fixture(navigation.clone())).viewport_px(1200, 1600),
        );
        session.run(|| tessera_ui::EntryRegistry::new().register_package(crate::res::i18n()));
        pump_layout_test_session(&mut session, 0);
        let handles = navigation.get().unwrap();
        session.run(|| {
            handles.directory.with_mut(SideSheetController::close);
            handles
                .router
                .with_mut(|router| router.replace(GlassComponentsPageDestination {}));
        });
        pump_layout_test_session(&mut session, 1_000_000_000);
        pump_layout_test_session(&mut session, 2_000_000_000);
        let language = session.run(|| {
            I18N.format(
                "ui-switch-language",
                [("language", I18N.text("ui-language-chinese").into())],
            )
        });
        let accessibility = session.inspect_accessibility().unwrap();
        let language = accessibility
            .find(&language, Role::Button)
            .expect("language action")
            .0;
        let slider = accessibility
            .nodes()
            .iter()
            .find(|(_, node)| node.role() == Role::Slider)
            .expect("glass slider after navigation")
            .0;
        let action = |target_node, action| ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node,
            data: None,
        };
        assert!(session.dispatch_accessibility_action(action(slider, Action::Increment)));
        pump_layout_test_session(&mut session, 2_016_666_667);
        let accessibility = session.inspect_accessibility().unwrap();
        let value = accessibility
            .nodes()
            .iter()
            .find(|(id, _)| *id == slider)
            .unwrap()
            .1
            .numeric_value()
            .unwrap();
        assert!(value > 0.0);
        let labels = |snapshot: &tessera_ui::testing::AccessibilitySnapshot| {
            snapshot
                .nodes()
                .iter()
                .filter_map(|(_, node)| node.label().map(str::to_owned))
                .collect::<Vec<_>>()
        };
        let english = labels(&accessibility);
        for (index, timestamp) in [2_033_333_334, 2_050_000_001].into_iter().enumerate() {
            assert!(session.dispatch_accessibility_action(action(language, Action::Click)));
            pump_layout_test_session(&mut session, timestamp);
            assert!(session.run(|| {
                handles
                    .router
                    .with(RouterController::current_is::<GlassComponentsPageDestination>)
            }));
            let current = session.inspect_accessibility().unwrap();
            assert_eq!(
                current
                    .nodes()
                    .iter()
                    .find(|(id, _)| *id == slider)
                    .unwrap()
                    .1
                    .numeric_value(),
                Some(value)
            );
            let current = labels(&current);
            assert!(current.iter().all(|label| !label.contains("ui-")));
            if index == 0 {
                assert_ne!(current, english);
            } else {
                assert_eq!(current, english);
            }
        }
        for _ in 0..2 {
            assert!(session.dispatch_accessibility_action(action(language, Action::Click)));
        }
        pump_layout_test_session(&mut session, 2_066_666_668);
        assert_eq!(labels(&session.inspect_accessibility().unwrap()), english);
    }
}
