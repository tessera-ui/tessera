use tessera_components::{
    app_bar::top_app_bar, column::column, list_item::list_item, modifier::ModifierExt as _,
    row::row, scaffold::scaffold, shape_def::Shape, surface::surface, theme::material_theme,
};
use tessera_ui::{
    Dp, Modifier, remember,
    router::{RouterController, RouterDestination, shard_home},
    tessera,
};

use crate::pages::home::HomeDestination;

#[tessera]
pub fn app() {
    material_theme().child(|| {
        scaffold()
            .top_bar(|| {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    top_app_bar()
                        .title("Examples")
                        .title_area_modifier(Modifier::new().window_drag_region())
                        .window_control_minimize()
                        .window_control_toggle_maximize()
                        .window_control_close();
                }

                #[cfg(target_arch = "wasm32")]
                {
                    top_app_bar().title("Examples");
                }
            })
            .content(|| {
                surface()
                    .modifier(Modifier::new().fill_max_size())
                    .child(|| {
                        row().children(|| {
                            let controller =
                                remember(|| RouterController::with_root(HomeDestination {}));

                            column()
                                .modifier(Modifier::new().width(Dp(300.0)))
                                .children(move || {
                                    list_item()
                                        .headline("Home")
                                        .on_click(move || {
                                            controller.with_mut(|c| c.replace(HomeDestination {}));
                                        })
                                        .selected(controller.with(|c| {
                                            c.last().is_some_and(|destination| {
                                                destination.shard_id()
                                                    == HomeDestination {}.shard_id()
                                            })
                                        }))
                                        .shape(Shape::rounded_rectangle(Dp(16.0)))
                                        .tonal_elevation(Dp(5.0));
                                });

                            shard_home().controller(controller);
                        });
                    });
            });
    });
}
