use tessera_ui::{Color, Dp, Modifier, State, remember, shard, tessera, use_context};
use tessera_ui_basic_components::{
    alignment::{Alignment, CrossAxisAlignment},
    column::{ColumnArgs, column},
    modifier::ModifierExt as _,
    pager::{
        PagerArgs, PagerController, PagerPageSize, horizontal_pager_with_controller, vertical_pager,
    },
    scrollable::{ScrollableArgs, scrollable},
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
    theme::MaterialTheme,
};

#[tessera]
#[shard]
pub fn pager_showcase() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            scrollable(
                ScrollableArgs::default().modifier(Modifier::new().fill_max_size()),
                move || {
                    surface(
                        SurfaceArgs::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0))),
                        move || {
                            pager_content();
                        },
                    );
                },
            )
        },
    );
}

#[tessera]
fn pager_content() {
    let horizontal_controller = remember(|| PagerController::new(0));
    let current_page = horizontal_controller.with(|c| c.current_page());
    column(
        ColumnArgs::default().modifier(Modifier::new().fill_max_width()),
        |scope| {
            scope.child(|| {
                text(TextArgs::default().text("Pager").size(Dp(24.0)));
            });
            scope.child(|| {
                text(
                    TextArgs::default()
                        .text("Snap-scrolling pages with spacing and padding.")
                        .color(
                            use_context::<MaterialTheme>()
                                .expect("MaterialTheme must be provided")
                                .get()
                                .color_scheme
                                .on_surface_variant,
                        ),
                );
            });
            scope.child(|| spacer(Modifier::new().height(Dp(16.0))));
            scope.child(move || {
                text(
                    TextArgs::default()
                        .text(format!(
                            "Horizontal pager (page {}/{})",
                            current_page + 1,
                            5
                        ))
                        .size(Dp(18.0)),
                );
            });
            scope.child(move || {
                horizontal_demo(horizontal_controller);
            });
            scope.child(|| spacer(Modifier::new().height(Dp(24.0))));
            scope.child(|| {
                text(TextArgs::default().text("Vertical pager").size(Dp(18.0)));
            });
            scope.child(vertical_demo);
        },
    );
}

#[tessera]
fn horizontal_demo(controller: State<PagerController>) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().fill_max_width().padding_all(Dp(12.0)))
            .style(scheme.surface_variant.into())
            .shape(Shape::rounded_rectangle(Dp(20.0))),
        move || {
            horizontal_pager_with_controller(
                PagerArgs::default()
                    .page_count(5)
                    .page_size(PagerPageSize::Fill)
                    .page_spacing(Dp(12.0))
                    .content_padding(Dp(16.0))
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(Modifier::new().fill_max_width().height(Dp(220.0))),
                controller,
                |page| {
                    pager_page("Page".to_string(), page);
                },
            );
        },
    );
}

#[tessera]
fn vertical_demo() {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().fill_max_width().padding_all(Dp(12.0)))
            .style(scheme.surface_variant.into())
            .shape(Shape::rounded_rectangle(Dp(20.0))),
        move || {
            vertical_pager(
                PagerArgs::default()
                    .page_count(4)
                    .page_size(PagerPageSize::Fixed(Dp(160.0)))
                    .page_spacing(Dp(12.0))
                    .content_padding(Dp(16.0))
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(Modifier::new().fill_max_width().height(Dp(320.0))),
                |page| {
                    pager_page("Step".to_string(), page);
                },
            );
        },
    );
}

#[tessera]
fn pager_page(label: String, page: usize) {
    let color = pager_color(page);
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().fill_max_size())
            .style(color.into())
            .shape(Shape::rounded_rectangle(Dp(18.0)))
            .content_alignment(Alignment::Center),
        move || {
            text(
                TextArgs::default()
                    .text(format!("{label} {}", page + 1))
                    .size(Dp(20.0))
                    .color(Color::WHITE),
            );
        },
    );
}

fn pager_color(index: usize) -> Color {
    let palette = [
        Color::new(0.15, 0.55, 0.85, 1.0),
        Color::new(0.1, 0.7, 0.55, 1.0),
        Color::new(0.95, 0.65, 0.15, 1.0),
        Color::new(0.9, 0.3, 0.45, 1.0),
        Color::new(0.45, 0.25, 0.8, 1.0),
    ];
    palette[index % palette.len()]
}
