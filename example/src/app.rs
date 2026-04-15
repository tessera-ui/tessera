use tessera_components::{
    app_bar::top_app_bar,
    column::column,
    list_item::list_item,
    modifier::ModifierExt as _,
    scaffold::scaffold,
    shape_def::{RoundedCorner, Shape},
    side_sheet::{SideSheetController, modal_side_sheet_provider},
    spacer::spacer,
    surface::surface,
    theme::material_theme,
};
use tessera_shard::{RouterController, RouterDestination, shard_home};
use tessera_ui::{Dp, Modifier, State, remember, tessera};

use crate::pages::{
    animation::AnimationPageDestination, basic_components::BasicComponentsPageDestination,
    custom_shader::CustomShaderPageDestination, glass_components::GlassComponentsPageDestination,
    home::HomePageDestination,
};

#[tessera]
pub fn app() {
    material_theme().child(|| {
        let side_sheet_controller = remember(|| SideSheetController::new(true));
        scaffold()
            .top_bar(move || {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    top_app_bar()
                        .title("Examples")
                        .title_area_modifier(Modifier::new().window_drag_region())
                        .navigation_icon(move || {
                            use tessera_components::{
                                icon_button::icon_button, res::material_icons,
                            };

                            icon_button()
                                .standard()
                                .icon(material_icons::filled::MENU_OPEN_SVG)
                                .on_click(move || {
                                    side_sheet_controller.with_mut(|c| {
                                        if c.is_open() {
                                            c.close();
                                        } else {
                                            c.open();
                                        }
                                    })
                                });
                        })
                        .window_control_minimize()
                        .window_control_toggle_maximize()
                        .window_control_close();
                }

                #[cfg(target_arch = "wasm32")]
                {
                    top_app_bar()
                        .navigation_icon(move || {
                            use tessera_components::{
                                icon_button::icon_button, res::material_icons,
                            };

                            icon_button()
                                .standard()
                                .icon(material_icons::filled::MENU_OPEN_SVG)
                                .on_click(move || {
                                    side_sheet_controller.with_mut(|c| {
                                        if c.is_open() {
                                            c.close();
                                        } else {
                                            c.open();
                                        }
                                    })
                                });
                        })
                        .title("Examples");
                }
            })
            .content(move || {
                let nav_controller =
                    remember(|| RouterController::with_root(HomePageDestination {}));
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
                                        nav_item(
                                            "Home",
                                            HomePageDestination {},
                                            nav_controller,
                                            side_sheet_controller,
                                        );

                                        spacer().modifier(Modifier::new().height(Dp(8.0)));

                                        nav_item(
                                            "Basic Components",
                                            BasicComponentsPageDestination {},
                                            nav_controller,
                                            side_sheet_controller,
                                        );

                                        spacer().modifier(Modifier::new().height(Dp(8.0)));

                                        nav_item(
                                            "Glass Components",
                                            GlassComponentsPageDestination {},
                                            nav_controller,
                                            side_sheet_controller,
                                        );

                                        spacer().modifier(Modifier::new().height(Dp(8.0)));

                                        nav_item(
                                            "Custom Shader",
                                            CustomShaderPageDestination {},
                                            nav_controller,
                                            side_sheet_controller,
                                        );

                                        spacer().modifier(Modifier::new().height(Dp(8.0)));

                                        nav_item(
                                            "Animation",
                                            AnimationPageDestination {},
                                            nav_controller,
                                            side_sheet_controller,
                                        );
                                    });
                            });
                    });
            });
    });
}

#[tessera]
fn nav_item<D: RouterDestination + Clone + PartialEq>(
    #[prop(into)] headline: String,
    destination: D,
    nav_controller: State<RouterController>,
    side_sheet_controller: State<SideSheetController>,
) {
    list_item()
        .modifier(Modifier::new().fill_max_width())
        .headline(headline)
        .on_click(move || {
            nav_controller.with_mut(|c| c.replace(destination.clone()));
            side_sheet_controller.with_mut(|c| {
                if c.is_open() {
                    c.close();
                }
            });
        })
        .selected(nav_controller.with(RouterController::current_is::<D>))
        .shape(Shape::RoundedRectangle {
            top_left: RoundedCorner::new(Dp(16.0)),
            top_right: RoundedCorner::Capsule,
            bottom_right: RoundedCorner::Capsule,
            bottom_left: RoundedCorner::new(Dp(16.0)),
        })
        .tonal_elevation(Dp(5.0));
}
