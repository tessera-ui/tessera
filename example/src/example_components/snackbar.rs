use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::{BoxedArgs, boxed},
    button::{ButtonArgs, button},
    column::{ColumnArgs, column},
    modifier::ModifierExt as _,
    snackbar::{
        SnackbarDuration, SnackbarHostArgs, SnackbarHostState, SnackbarRequest, snackbar_host,
    },
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Dp, Modifier, remember, shard};
#[shard]
pub fn snackbar_showcase() {
    let host_state = remember(SnackbarHostState::new);

    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            boxed(
                BoxedArgs::default().modifier(Modifier::new().fill_max_size()),
                move |scope| {
                    scope.child(move || {
                        column(
                            ColumnArgs::default()
                                .modifier(Modifier::new().fill_max_width().padding_all(Dp(16.0)))
                                .cross_axis_alignment(CrossAxisAlignment::Start),
                            |column_scope| {
                                column_scope.child(|| {
                                    text(
                                        &TextArgs::default()
                                            .text("Snackbar Showcase")
                                            .size(Dp(20.0)),
                                    );
                                });

                                column_scope.child(|| {
                                    spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0))))
                                });

                                column_scope.child(move || {
                                    button(&ButtonArgs::with_child(
                                        ButtonArgs::filled(move || {
                                            host_state.with_mut(|state| {
                                                state.show_snackbar(
                                                    SnackbarRequest::new("Saved")
                                                        .duration(SnackbarDuration::Short),
                                                );
                                            });
                                        })
                                        .modifier(Modifier::new().fill_max_width()),
                                        || {
                                            text(&TextArgs::from("Show short message"));
                                        },
                                    ));
                                });

                                column_scope.child(|| {
                                    spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0))))
                                });

                                column_scope.child(move || {
                                    let host_state = host_state;
                                    button(&ButtonArgs::with_child(
                                        ButtonArgs::filled(move || {
                                            host_state.with_mut(|state| {
                                                state.show_snackbar(
                                                    SnackbarRequest::new("Message archived")
                                                        .action_label("Undo")
                                                        .with_dismiss_action(true),
                                                );
                                            });
                                        })
                                        .modifier(Modifier::new().fill_max_width()),
                                        || {
                                            text(&TextArgs::from("Show action snackbar"));
                                        },
                                    ));
                                });

                                column_scope.child(|| {
                                    spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0))))
                                });

                                column_scope.child(move || {
                                    let host_state = host_state;
                                    button(&ButtonArgs::with_child(
                                        ButtonArgs::filled(move || {
                                            host_state.with_mut(|state| {
                                                state.show_snackbar(
                                                    SnackbarRequest::new("Sync paused")
                                                        .action_label("Resume")
                                                        .with_dismiss_action(true)
                                                        .duration(SnackbarDuration::Indefinite),
                                                );
                                            });
                                        })
                                        .modifier(Modifier::new().fill_max_width()),
                                        || {
                                            text(&TextArgs::from("Show indefinite snackbar"));
                                        },
                                    ));
                                });
                            },
                        );
                    });

                    scope.child_with_alignment(Alignment::BottomCenter, move || {
                        snackbar_host(&SnackbarHostArgs::new(host_state));
                    });
                },
            );
        },
    ));
}
