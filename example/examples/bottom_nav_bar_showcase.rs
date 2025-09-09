use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui::{
    Color, DimensionValue, Renderer,
    renderer::TesseraConfig,
    router::{Router, router_root},
    shard, tessera,
};
use tessera_ui_basic_components::{
    alignment::Alignment,
    bottom_nav_bar::{BottomNavBarState, bottom_nav_bar},
    boxed::{BoxedArgsBuilder, boxed},
    column::{ColumnArgsBuilder, column},
    surface::{SurfaceArgs, surface},
    text::text,
};

// 1. Define the screens as actual shards

#[tessera]
fn white_surface_wrapper<F>(content: F)
where
    F: FnOnce(),
{
    surface(
        SurfaceArgs {
            style: Color::WHITE.into(),
            width: Some(DimensionValue::FILLED),
            height: Some(DimensionValue::FILLED),
            ..Default::default()
        },
        None,
        content,
    );
}

#[tessera]
#[shard]
fn home_screen() {
    white_surface_wrapper(|| {
        boxed(
            BoxedArgsBuilder::default()
                .alignment(Alignment::Center)
                .build()
                .unwrap(),
            |scope| scope.child(|| text("Welcome to the Home Screen!")),
        )
    })
}

#[tessera]
#[shard]
fn favorites_screen() {
    white_surface_wrapper(|| {
        boxed(
            BoxedArgsBuilder::default()
                .alignment(Alignment::Center)
                .build()
                .unwrap(),
            |scope| scope.child(|| text("Here are your Favorites.")),
        )
    })
}

#[tessera]
#[shard]
fn profile_screen() {
    white_surface_wrapper(|| {
        boxed(
            BoxedArgsBuilder::default()
                .alignment(Alignment::Center)
                .build()
                .unwrap(),
            |scope| scope.child(|| text("This is your Profile.")),
        )
    })
}

// 2. Set up and run the application in main
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    println!("Starting BottomNavBar Showcase Example");
    println!("Click the navigation items to switch screens.");

    let config = TesseraConfig {
        window_title: "Tessera BottomNavBar Showcase".to_string(),
        ..Default::default()
    };

    // The state for the bottom_nav_bar is managed here, at the root.
    let nav_bar_state = Arc::new(RwLock::new(BottomNavBarState::new(0)));

    Renderer::run_with_config(
        {
            let nav_bar_state = nav_bar_state.clone();
            move || {
                let nav_bar_state = nav_bar_state.clone();
                // The root component is a column that holds the router viewport and the nav bar.
                column(
                    ColumnArgsBuilder::default().build().unwrap(),
                    move |scope| {
                        // The router_root component acts as the viewport for the currently active shard.
                        scope.child_weighted(
                            move || {
                                router_root(HomeScreenDestination {});
                            },
                            1.0,
                        );

                        // The bottom_nav_bar is outside the router and always visible.
                        scope.child(move || {
                            bottom_nav_bar(nav_bar_state, |nav_scope| {
                                nav_scope.child(
                                    || text("Home"),
                                    move || {
                                        Router::with_mut(|router| {
                                            router.reset_with(HomeScreenDestination {});
                                        });
                                    },
                                );
                                nav_scope.child(
                                    || text("Favorites"),
                                    move || {
                                        Router::with_mut(|router| {
                                            router.reset_with(FavoritesScreenDestination {});
                                        });
                                    },
                                );
                                nav_scope.child(
                                    || text("Profile"),
                                    move || {
                                        Router::with_mut(|router| {
                                            router.reset_with(ProfileScreenDestination {});
                                        });
                                    },
                                );
                            });
                        });
                    },
                );
            }
        },
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
        },
        config,
    )?;

    Ok(())
}
