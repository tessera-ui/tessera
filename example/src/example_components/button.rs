use std::sync::{Arc, Mutex};

use closure::closure;
use tessera_ui::{DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    RippleState,
    alignment::CrossAxisAlignment,
    button::{ButtonArgs, ButtonArgsBuilder, button},
    column::{ColumnArgsBuilder, column},
    icon::{IconArgsBuilder, IconContent},
    icon_button::{IconButtonArgsBuilder, icon_button},
    image_vector::{ImageVectorData, ImageVectorSource, load_image_vector_from_source},
    material_color::global_material_scheme,
    row::{RowArgsBuilder, row},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    shape_def::Shape,
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

const ICON_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/emoji_u1f416.svg"
));

#[derive(Clone)]
struct ButtonShowcaseState {
    scrollable_state: ScrollableState,
    icon_button_ripple: RippleState,
    filled_ripple: RippleState,
    elevated_ripple: RippleState,
    tonal_ripple: RippleState,
    outlined_ripple: RippleState,
    text_ripple: RippleState,
    counter: Arc<Mutex<i32>>,
    icon_data: Arc<ImageVectorData>,
}

impl ButtonShowcaseState {
    fn new() -> Self {
        let icon_data = Arc::new(
            load_image_vector_from_source(&ImageVectorSource::Bytes(Arc::from(ICON_BYTES)))
                .expect("Failed to load icon SVG"),
        );

        Self {
            scrollable_state: Default::default(),
            icon_button_ripple: RippleState::new(),
            filled_ripple: RippleState::new(),
            elevated_ripple: RippleState::new(),
            tonal_ripple: RippleState::new(),
            outlined_ripple: RippleState::new(),
            text_ripple: RippleState::new(),
            counter: Default::default(),
            icon_data,
        }
    }
}

impl Default for ButtonShowcaseState {
    fn default() -> Self {
        Self::new()
    }
}

#[tessera]
#[shard]
pub fn button_showcase(#[state] state: ButtonShowcaseState) {
    let scrollable_state = state.scrollable_state.clone();
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .build()
            .unwrap(),
        None,
        move || {
            scrollable(
                ScrollableArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .build()
                    .unwrap(),
                scrollable_state,
                move || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .padding(Dp(25.0))
                            .width(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        None,
                        move || {
                            column(
                                ColumnArgsBuilder::default()
                                    .cross_axis_alignment(CrossAxisAlignment::Start)
                                    .build()
                                    .unwrap(),
                                move |scope| {
                                    scope.child(|| {
                                        text(
                                            TextArgsBuilder::default()
                                                .text("Button Showcase")
                                                .size(Dp(20.0))
                                                .build()
                                                .unwrap(),
                                        )
                                    });

                                    scope.child(|| {
                                        text(
                                            TextArgsBuilder::default()
                                                .text("Icon Button")
                                                .size(Dp(16.0))
                                                .build()
                                                .unwrap(),
                                        )
                                    });

                                    scope.child(closure!(clone state, || {
                                        let icon = IconArgsBuilder::default()
                                            .content(IconContent::from(
                                                state.icon_data.clone(),
                                            ))
                                            .size(Dp(24.0))
                                            .build()
                                            .unwrap();

                                        let button_args = IconButtonArgsBuilder::default()
                                            .button(
                                                ButtonArgsBuilder::default()
                                                    .shape(Shape::Ellipse)
                                                    .color(global_material_scheme().surface_variant)
                                                    .hover_color(Some(
                                                        global_material_scheme().surface,
                                                    ))
                                                    .on_click(Arc::new(closure!(
                                                        clone state.counter,
                                                        || {
                                                            let mut count =
                                                                counter.lock().unwrap();
                                                            *count += 1;
                                                            println!("Icon button clicked!");
                                                        }
                                                    )))
                                                    .padding(Dp(12.0))
                                                    .build()
                                                    .unwrap(),
                                            )
                                            .icon(icon)
                                            .build()
                                            .unwrap();

                                        icon_button(
                                            button_args,
                                            state.icon_button_ripple.clone(),
                                        );
                                    }));

                                    scope.child(|| {
                                        spacer(
                                            SpacerArgsBuilder::default()
                                                .height(Dp(20.0))
                                                .build()
                                                .unwrap(),
                                        );
                                    });

                                    scope.child(|| {
                                        text(
                                            TextArgsBuilder::default()
                                                .text("Button Styles")
                                                .size(Dp(20.0))
                                                .build()
                                                .unwrap(),
                                        )
                                    });

                                    scope.child(closure!(clone state, || {
                                        row(
                                            RowArgsBuilder::default().build().unwrap(),
                                            move |scope| {
                                                scope.child(closure!(clone state, || {
                                                    button(
                                                        ButtonArgs::filled(Arc::new(|| println!("Filled clicked"))),
                                                        state.filled_ripple.clone(),
                                                        || {
                                                            text(TextArgsBuilder::default().text("Filled").color(global_material_scheme().on_primary).build().unwrap());
                                                        },
                                                    );
                                                }));
                                                scope.child(|| spacer(SpacerArgsBuilder::default().width(Dp(8.0)).build().unwrap()));

                                                scope.child(closure!(clone state, || {
                                                    button(
                                                        ButtonArgs::elevated(Arc::new(|| println!("Elevated clicked"))),
                                                        state.elevated_ripple.clone(),
                                                        || {
                                                            text(TextArgsBuilder::default().text("Elevated").color(global_material_scheme().primary).build().unwrap());
                                                        },
                                                    );
                                                }));
                                                scope.child(|| spacer(SpacerArgsBuilder::default().width(Dp(8.0)).build().unwrap()));

                                                scope.child(closure!(clone state, || {
                                                    button(
                                                        ButtonArgs::tonal(Arc::new(|| println!("Tonal clicked"))),
                                                        state.tonal_ripple.clone(),
                                                        || {
                                                            text(TextArgsBuilder::default().text("Tonal").color(global_material_scheme().on_secondary_container).build().unwrap());
                                                        },
                                                    );
                                                }));
                                            },
                                        );
                                    }));

                                    scope.child(|| {
                                        spacer(
                                            SpacerArgsBuilder::default()
                                                .height(Dp(8.0))
                                                .build()
                                                .unwrap(),
                                        )
                                    });

                                    scope.child(closure!(clone state, || {
                                        row(
                                            RowArgsBuilder::default().build().unwrap(),
                                            move |scope| {
                                                scope.child(closure!(clone state, || {
                                                    button(
                                                        ButtonArgs::outlined(Arc::new(|| println!("Outlined clicked"))),
                                                        state.outlined_ripple.clone(),
                                                        || {
                                                            text(TextArgsBuilder::default().text("Outlined").color(global_material_scheme().primary).build().unwrap());
                                                        },
                                                    );
                                                }));
                                                scope.child(|| spacer(SpacerArgsBuilder::default().width(Dp(8.0)).build().unwrap()));

                                                scope.child(closure!(clone state, || {
                                                    button(
                                                        ButtonArgs::text(Arc::new(|| println!("Text clicked"))),
                                                        state.text_ripple.clone(),
                                                        || {
                                                            text(TextArgsBuilder::default().text("Text").color(global_material_scheme().primary).build().unwrap());
                                                        },
                                                    );
                                                }));
                                            },
                                        );
                                    }));
                                },
                            )
                        },
                    );
                },
            )
        },
    );
}
