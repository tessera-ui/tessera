use tessera_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    column::{ColumnArgs, column},
    icon::{IconArgs, icon},
    material_icons::filled,
    modifier::ModifierExt as _,
    row::{RowArgs, row},
    spacer::spacer,
    split_buttons::{
        SplitButtonDefaults, SplitButtonLayoutArgs, SplitButtonLeadingArgs, SplitButtonSize,
        SplitButtonTrailingArgs, SplitButtonVariant, split_button_layout, split_leading_button,
        split_trailing_button,
    },
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Dp, Modifier, remember, shard, tessera};

#[tessera]
#[shard]
pub fn split_buttons_showcase() {
    let counter = remember(|| 0u32);
    let small = SplitButtonSize::Small;
    let medium = SplitButtonSize::Medium;

    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            column(
                ColumnArgs::default()
                    .modifier(Modifier::new().fill_max_size().padding_all(Dp(16.0)))
                    .cross_axis_alignment(CrossAxisAlignment::Start),
                |scope| {
                    scope.child(|| {
                        text(
                            TextArgs::default()
                                .text("Split Buttons Showcase")
                                .size(Dp(20.0)),
                        );
                    });

                    scope.child(|| spacer(Modifier::new().height(Dp(16.0))));

                    scope.child(|| text(TextArgs::default().text("Filled").size(Dp(14.0))));

                    scope.child(move || {
                        let leading_counter = counter;
                        let trailing_counter = counter;
                        split_button_layout(
                            SplitButtonLayoutArgs::default(),
                            move || {
                                split_leading_button(
                                    SplitButtonLeadingArgs::default()
                                        .variant(SplitButtonVariant::Filled)
                                        .size(small)
                                        .on_click(move || {
                                            leading_counter.with_mut(|value| *value += 1);
                                        }),
                                    || text("Create"),
                                );
                            },
                            move || {
                                split_trailing_button(
                                    SplitButtonTrailingArgs::default()
                                        .variant(SplitButtonVariant::Filled)
                                        .size(small)
                                        .on_click(move || {
                                            trailing_counter.with_mut(|value| *value += 1);
                                        }),
                                    move || {
                                        icon(
                                            IconArgs::from(filled::chevron_right_icon()).size(
                                                SplitButtonDefaults::trailing_icon_size(small),
                                            ),
                                        );
                                    },
                                );
                            },
                        );
                    });

                    scope.child(|| spacer(Modifier::new().height(Dp(16.0))));

                    scope.child(|| text(TextArgs::default().text("Outlined").size(Dp(14.0))));

                    scope.child(move || {
                        split_button_layout(
                            SplitButtonLayoutArgs::default(),
                            move || {
                                split_leading_button(
                                    SplitButtonLeadingArgs::default()
                                        .variant(SplitButtonVariant::Outlined)
                                        .size(medium),
                                    || {
                                        row(
                                            RowArgs::default()
                                                .main_axis_alignment(MainAxisAlignment::Center)
                                                .cross_axis_alignment(CrossAxisAlignment::Center),
                                            |row_scope| {
                                                row_scope.child(|| {
                                                    icon(
                                                        IconArgs::from(filled::inbox_icon()).size(
                                                            SplitButtonDefaults::LEADING_ICON_SIZE,
                                                        ),
                                                    );
                                                });
                                                row_scope.child(|| {
                                                    spacer(Modifier::new().width(Dp(8.0)));
                                                });
                                                row_scope.child(|| text("Archive"));
                                            },
                                        );
                                    },
                                );
                            },
                            move || {
                                split_trailing_button(
                                    SplitButtonTrailingArgs::default()
                                        .variant(SplitButtonVariant::Outlined)
                                        .size(medium),
                                    move || {
                                        icon(
                                            IconArgs::from(filled::chevron_right_icon()).size(
                                                SplitButtonDefaults::trailing_icon_size(medium),
                                            ),
                                        );
                                    },
                                );
                            },
                        );
                    });
                },
            );
        },
    );
}
