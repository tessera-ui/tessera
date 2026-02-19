use std::{thread, time::Duration};

use tessera_components::{
    column::{ColumnArgs, column},
    lazy_list::{LazyColumnArgs, lazy_column},
    modifier::ModifierExt as _,
    pull_refresh::{PullRefreshArgs, pull_refresh},
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
    theme::MaterialTheme,
};
use tessera_ui::{Dp, Modifier, remember, shard, tessera, use_context};

#[tessera]
#[shard]
pub fn pull_refresh_showcase() {
    let refreshing = remember(|| false);
    let refresh_count = remember(|| 0u32);
    let items = remember(|| (1..=20).collect::<Vec<u32>>());
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let container_color = scheme.surface_container_low;

    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            column(
                ColumnArgs::default()
                    .modifier(Modifier::new().fill_max_size().padding_all(Dp(16.0))),
                |scope| {
                    scope.child(|| {
                        text(&TextArgs::default().text("Pull-to-refresh").size(Dp(20.0)));
                    });

                    scope.child(|| {
                        spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0))));
                    });

                    scope.child(move || {
                        let status = if refreshing.get() {
                            "Refreshing"
                        } else {
                            "Idle"
                        };
                        let label =
                            format!("Status: {status} · Refreshes: {}", refresh_count.get());
                        text(&TextArgs::default().text(label).size(Dp(14.0)));
                    });

                    scope.child(|| {
                        spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0))));
                    });

                    scope.child(move || {
                        let args = PullRefreshArgs::new(move || {
                            if refreshing.get() {
                                return;
                            }
                            refreshing.set(true);
                            let next_count = refresh_count.get() + 1;
                            thread::spawn(move || {
                                thread::sleep(Duration::from_millis(800));
                                items.with_mut(|list| {
                                    if next_count % 2 == 0 {
                                        list.rotate_left(1);
                                    } else {
                                        list.reverse();
                                    }
                                });
                                refresh_count.set(next_count);
                                refreshing.set(false);
                            });
                        })
                        .refreshing(refreshing.get())
                        .modifier(Modifier::new().fill_max_width().height(Dp(320.0)))
                        .child(move || {
                            lazy_column(
                                &LazyColumnArgs {
                                    modifier: Modifier::new()
                                        .fill_max_size()
                                        .background(container_color),
                                    content_padding: Dp(12.0),
                                    ..Default::default()
                                }
                                .content(move |scope| {
                                    scope.item(|| {
                                        text(
                                            &TextArgs::default()
                                                .text("Pull down to trigger refresh.")
                                                .size(Dp(14.0)),
                                        );
                                    });
                                    scope.item(|| {
                                        spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0))));
                                    });

                                    let item_values = items.get();
                                    scope.items_from_iter_with_key(
                                        item_values,
                                        |_, value| *value,
                                        |_, value| {
                                            let label = format!("Feed item {}", value);
                                            text(&TextArgs::default().text(label));
                                        },
                                    );
                                }),
                            );
                        });
                        pull_refresh(&args);
                    });
                },
            );
        },
    ));
}
