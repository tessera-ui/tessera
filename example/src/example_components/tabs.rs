use tessera_ui::{DimensionValue, Dp, shard, tessera, use_context};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    material_icons,
    scrollable::{ScrollableArgsBuilder, scrollable},
    surface::{SurfaceArgsBuilder, surface},
    tabs::{TabsArgsBuilder, TabsVariant, tabs},
    text::{TextArgsBuilder, text},
    theme::MaterialTheme,
};

#[tessera]
#[shard]
pub fn tabs_showcase() {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .build()
            .unwrap(),
        move || {
            scrollable(
                ScrollableArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .build()
                    .unwrap(),
                move || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .padding(Dp(25.0))
                            .width(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
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
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .build()
            .unwrap(),
        |scope| {
            scope.child(|| {
                text(
                    TextArgsBuilder::default()
                        .text("Tabs Showcase")
                        .size(Dp(20.0))
                        .build()
                        .unwrap(),
                )
            });

            scope.child(move || {
                tabs(
                    TabsArgsBuilder::default()
                        .width(DimensionValue::FILLED)
                        .variant(TabsVariant::Primary)
                        .build()
                        .unwrap(),
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
                    TextArgsBuilder::default()
                        .text("Secondary Tabs Showcase")
                        .size(Dp(16.0))
                        .build()
                        .unwrap(),
                )
            });

            scope.child(move || {
                let scheme = use_context::<MaterialTheme>().get().color_scheme;
                tabs(
                    TabsArgsBuilder::default()
                        .width(DimensionValue::FILLED)
                        .variant(TabsVariant::Secondary)
                        .active_content_color(scheme.on_surface)
                        .build()
                        .unwrap(),
                    |scope| {
                        scope.child_label("Flights", || text("Fly in the air..."));
                        scope.child_label("Hotel", || text("Sleep well..."));
                        scope.child_label("Cars", || text("Beep beep..."));
                    },
                );
            });

            scope.child(|| {
                text(
                    TextArgsBuilder::default()
                        .text("Scrollable Tabs Showcase")
                        .size(Dp(16.0))
                        .build()
                        .unwrap(),
                )
            });

            scope.child(move || {
                tabs(
                    TabsArgsBuilder::default()
                        .width(DimensionValue::FILLED)
                        .scrollable(true)
                        .variant(TabsVariant::Primary)
                        .build()
                        .unwrap(),
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
