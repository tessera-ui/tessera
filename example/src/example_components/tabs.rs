use tessera_ui::{Dp, Modifier, shard, tessera, use_context};
use tessera_ui_basic_components::{
    column::{ColumnArgs, column},
    material_icons,
    modifier::ModifierExt as _,
    scrollable::{ScrollableArgs, scrollable},
    surface::{SurfaceArgs, surface},
    tabs::{TabsArgs, TabsVariant, tabs},
    text::{TextArgs, text},
    theme::MaterialTheme,
};

#[tessera]
#[shard]
pub fn tabs_showcase() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            scrollable(
                ScrollableArgs::default().modifier(Modifier::new().fill_max_size()),
                move || {
                    surface(
                        SurfaceArgs::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0))),
                        move || {
                            test_content();
                        },
                    );
                },
            )
        },
    );
}

#[tessera]
fn test_content() {
    column(
        ColumnArgs::default().modifier(Modifier::new().fill_max_width()),
        |scope| {
            scope.child(|| text(TextArgs::default().text("Tabs Showcase").size(Dp(20.0))));

            scope.child(move || {
                tabs(
                    TabsArgs::default()
                        .modifier(Modifier::new().fill_max_width())
                        .variant(TabsVariant::Primary),
                    |scope| {
                        scope.child_label_with_icon(
                            "Flights",
                            material_icons::filled::flight_icon(),
                            || text("Fly in the air..."),
                        );
                        scope.child_label_with_icon(
                            "Hotel",
                            material_icons::filled::hotel_icon(),
                            || text("Sleep well..."),
                        );
                        scope.child_label_with_icon(
                            "Cars",
                            material_icons::filled::directions_car_icon(),
                            || text("Beep beep..."),
                        );
                    },
                );
            });

            scope.child(|| {
                text(
                    TextArgs::default()
                        .text("Secondary Tabs Showcase")
                        .size(Dp(16.0)),
                )
            });

            scope.child(move || {
                let scheme = use_context::<MaterialTheme>().get().color_scheme;
                tabs(
                    TabsArgs::default()
                        .modifier(Modifier::new().fill_max_width())
                        .variant(TabsVariant::Secondary)
                        .active_content_color(scheme.on_surface),
                    |scope| {
                        scope.child_label("Flights", || text("Fly in the air..."));
                        scope.child_label("Hotel", || text("Sleep well..."));
                        scope.child_label("Cars", || text("Beep beep..."));
                    },
                );
            });

            scope.child(|| {
                text(
                    TextArgs::default()
                        .text("Scrollable Tabs Showcase")
                        .size(Dp(16.0)),
                )
            });

            scope.child(move || {
                tabs(
                    TabsArgs::default()
                        .modifier(Modifier::new().fill_max_width())
                        .scrollable(true)
                        .variant(TabsVariant::Primary),
                    |scope| {
                        for label in [
                            "Home",
                            "Explore",
                            "Trips",
                            "Tickets",
                            "Favorites",
                            "Messages",
                            "Profile",
                            "Settings",
                            "Help",
                            "About",
                        ] {
                            let title = label.to_string();
                            let content = format!("Content for {label}");
                            scope.child_label(title, move || text(content));
                        }
                    },
                );
            });
        },
    )
}
