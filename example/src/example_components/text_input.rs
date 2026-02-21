use tessera_components::{
    icon::{IconArgs, icon},
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column},
    material_icons::filled,
    modifier::ModifierExt as _,
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
    text_field::{TextFieldArgs, text_field},
    text_input::{TextInputArgs, TextInputController, text_input},
};
use tessera_ui::{Dp, Modifier, remember, retain, shard};
#[shard]
pub fn text_input_showcase() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        test_content,
    ));
}
fn test_content() {
    let filled_args = TextFieldArgs::filled()
        .modifier(Modifier::new().fill_max_width())
        .label("Filled label")
        .placeholder("Tessera");
    let filled_font_size = filled_args.font_size;
    let filled_line_height = filled_args.line_height;
    let filled_controller =
        remember(|| TextInputController::new(filled_font_size, filled_line_height));

    let outlined_args = TextFieldArgs::outlined()
        .modifier(Modifier::new().fill_max_width())
        .label("Outlined label")
        .placeholder("Outlined input");
    let outlined_font_size = outlined_args.font_size;
    let outlined_line_height = outlined_args.line_height;
    let outlined_controller =
        remember(|| TextInputController::new(outlined_font_size, outlined_line_height));

    let secure_args = TextFieldArgs::secure()
        .modifier(Modifier::new().fill_max_width())
        .placeholder("s3cret");
    let secure_font_size = secure_args.font_size;
    let secure_line_height = secure_args.line_height;
    let secure_controller =
        remember(|| TextInputController::new(secure_font_size, secure_line_height));

    let outlined_secure_args = TextFieldArgs::outlined_secure()
        .modifier(Modifier::new().fill_max_width())
        .placeholder("tessera");
    let outlined_secure_font_size = outlined_secure_args.font_size;
    let outlined_secure_line_height = outlined_secure_args.line_height;
    let outlined_secure_controller = remember(|| {
        TextInputController::new(outlined_secure_font_size, outlined_secure_line_height)
    });

    let leading_icon_args = IconArgs::from(filled::home_icon()).size(Dp(20.0));
    let trailing_icon_args = IconArgs::from(filled::info_icon()).size(Dp(20.0));
    let icon_prefix_args = TextFieldArgs::filled()
        .modifier(Modifier::new().fill_max_width())
        .placeholder("example.com")
        .leading_icon({
            let icon_args = leading_icon_args.clone();
            move || {
                icon(&icon_args.clone());
            }
        })
        .trailing_icon({
            let icon_args = trailing_icon_args.clone();
            move || {
                icon(&icon_args.clone());
            }
        })
        .prefix(|| {
            text(&TextArgs::default().text("https://"));
        })
        .suffix(|| {
            text(&TextArgs::default().text(".com"));
        });
    let icon_prefix_font_size = icon_prefix_args.font_size;
    let icon_prefix_line_height = icon_prefix_args.line_height;
    let icon_prefix_controller =
        remember(|| TextInputController::new(icon_prefix_font_size, icon_prefix_line_height));

    let editor_state = remember(|| {
        let mut controller = TextInputController::new(Dp(22.0), None);
        controller.set_text("Share notes, drafts, or feedback here.");
        controller
    });
    let list_controller = retain(LazyListController::new);

    lazy_column(
        &LazyColumnArgs::default()
            .modifier(Modifier::new().fill_max_width())
            .content_padding(Dp(25.0))
            .item_spacing(Dp(8.0))
            .controller(list_controller)
            .content(move |scope| {
                scope.item(|| {
                    text(
                        &TextArgs::default()
                            .text("Text Input Showcase")
                            .size(Dp(20.0)),
                    )
                });

                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0)))));

                scope.item(|| text(&TextArgs::default().text("Filled text field").size(Dp(14.0))));
                let filled_args = filled_args.clone();
                scope.item(move || {
                    let args = filled_args.clone().controller(filled_controller);
                    text_field(&args);
                });

                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(8.0)))));
                scope.item(|| {
                    text(
                        &TextArgs::default()
                            .text("Outlined text field")
                            .size(Dp(14.0)),
                    )
                });
                let outlined_args = outlined_args.clone();
                scope.item(move || {
                    let args = outlined_args.clone().controller(outlined_controller);
                    text_field(&args);
                });

                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(8.0)))));
                scope.item(|| {
                    text(
                        &TextArgs::default()
                            .text("Text field with icons and prefix/suffix")
                            .size(Dp(14.0)),
                    )
                });
                let icon_prefix_args = icon_prefix_args.clone();
                scope.item(move || {
                    let args = icon_prefix_args.clone().controller(icon_prefix_controller);
                    text_field(&args);
                });

                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(8.0)))));
                scope.item(|| text(&TextArgs::default().text("Secure text field").size(Dp(14.0))));
                let secure_args = secure_args.clone();
                scope.item(move || {
                    let args = secure_args.clone().controller(secure_controller);
                    text_field(&args);
                });

                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(8.0)))));
                scope.item(|| {
                    text(
                        &TextArgs::default()
                            .text("Outlined secure field")
                            .size(Dp(14.0)),
                    )
                });
                let outlined_secure_args = outlined_secure_args.clone();
                scope.item(move || {
                    let args = outlined_secure_args
                        .clone()
                        .controller(outlined_secure_controller);
                    text_field(&args);
                });

                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));
                scope.item(|| text(&TextArgs::default().text("Multiline editor").size(Dp(14.0))));
                scope.item(move || {
                    let args = TextInputArgs::default()
                        .modifier(Modifier::new().fill_max_width().height(Dp(200.0)))
                        .on_change(move |v| v)
                        .controller(editor_state);
                    text_input(&args);
                });
            }),
    )
}
