//! HIG component directory and articles that compare each control in light and dark windows.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    experiment::hig::{
        presentation::presentation_host,
        text::text,
        theme::{Appearance, HigTheme, TextStyle, hig_theme},
    },
    flow_row::flow_row,
    modifier::ModifierExt as _,
    row::row,
    shape_def::Shape,
    spacer::spacer,
};
use tessera_foundation::{
    modifier::{ClickableArgs, ModifierExt as _, SemanticsArgs},
    pointer_gestures::{ScrollGestureConfig, detect_scroll_gestures},
};
use tessera_shard::{RouterController, shard, use_router};
use tessera_ui::{
    AxisConstraint, Callback, CallbackWith, Color, Dp, KeyboardInput, KeyboardInputModifierNode,
    LayoutCoordinates, LayoutModifier, LayoutModifierChild, LayoutModifierInput, LayoutResult,
    MeasurementError, Model, ModelInit, Modifier, PxPosition, RenderSlot, State,
    accesskit::Role,
    focus::{FocusProperties, FocusRevealRequest},
    key,
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
    winit::{
        event::ElementState,
        keyboard::{Key, NamedKey},
    },
};

use crate::res::I18N;

use self::examples::{
    CATEGORIES, COMPONENTS, ExampleState, ExampleValues, cell, label, label_colors,
};

mod examples;

#[cfg(test)]
#[path = "hig_components_tests.rs"]
mod tests;

/// Horizontal page margin.
const MARGIN: f64 = 32.0;
/// Space between directory cards and between the light and dark windows.
const GAP: f64 = 24.0;
const PREVIEW_HEIGHT: f64 = 180.0;
const ROW_LABEL_WIDTH: f64 = 88.0;
const MIN_CELL_WIDTH: f64 = 132.0;
const WINDOW_PADDING: f64 = 16.0;

struct DirectoryModel {
    scroll: State<ScrollState>,
    width: State<Dp>,
}

impl ModelInit for DirectoryModel {
    fn init() -> Self {
        Self {
            scroll: State::new(ScrollState::default()),
            width: State::new(Dp(900.0)),
        }
    }
}

#[shard]
pub fn hig_components_page(model: Model<DirectoryModel>) {
    let router = use_router();
    let (scroll, width) = model.with(|model| (model.scroll, model.width));
    hig_catalog(
        scroll,
        width,
        CallbackWith::new(move |index| {
            router.with_mut(|router| router.push(HigArticlePageDestination { index }));
        }),
    );
}

#[shard]
pub fn hig_article_page(index: usize) {
    let router = use_router();
    key(index, move || {
        hig_article(
            index,
            CallbackWith::new(move |index| {
                router.with_mut(|router| router.replace(HigArticlePageDestination { index }));
            }),
        );
    });
}

/// The localized title of the article at `index`.
pub(crate) fn article_title(index: usize) -> String {
    label(COMPONENTS[index].title)
}

pub(crate) fn current_article(router: &RouterController) -> Option<usize> {
    (router.last()? as &dyn std::any::Any)
        .downcast_ref::<HigArticlePageDestination>()
        .map(|article| article.index)
}

/// An sRGB-encoded color; [Color] holds linear components.
fn srgb(r: u8, g: u8, b: u8) -> Color {
    let channel = |value: u8| {
        let value = f32::from(value) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    Color::from_rgb(channel(r), channel(g), channel(b))
}

fn preview_fill() -> Color {
    srgb(246, 246, 246)
}

fn directory_columns(width: Dp) -> usize {
    if width.0 <= 600.0 {
        1
    } else if width.0 < 1295.0 {
        2
    } else {
        3
    }
}

fn track_width(width: State<Dp>) -> Modifier {
    Modifier::new()
        .fill_max_width()
        .on_layout_coordinates(move |coordinates: LayoutCoordinates| {
            let next = Dp::from(coordinates.local_size.width);
            if width.get() != next {
                width.set(next);
            }
        })
}

fn type_style(size: f64, line_height: f64, weight: u16) -> TextStyle {
    TextStyle {
        font_size: Dp(size),
        line_height: Dp(line_height),
        font_weight: weight,
        ..Default::default()
    }
}

#[tessera]
fn hig_catalog(scroll: State<ScrollState>, width: State<Dp>, on_open: CallbackWith<usize>) {
    hig_theme().child(move || {
        presentation_host().child(move || {
            viewport().scroll(scroll).child(move || {
                column().modifier(track_width(width)).children(move || {
                    header("ui-hig-components".into(), "ui-hig-introduction".into());
                    for (category, title) in CATEGORIES.into_iter().enumerate() {
                        gap(Dp(48.0));
                        padded(move || heading(title.into(), 22.0, 2));
                        gap(Dp(16.0));
                        directory_group(category, width.get(), on_open);
                    }
                    gap(Dp(64.0));
                });
            });
        });
    });
}

#[tessera]
fn directory_group(category: usize, width: Dp, on_open: CallbackWith<usize>) {
    let columns = directory_columns(width) as f64;
    let card_width = Dp(((width.0 - 2.0 * MARGIN - (columns - 1.0) * GAP) / columns).max(0.0));
    flow_row()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_symmetric(Dp(MARGIN), Dp::ZERO),
        )
        .item_spacing(Dp(GAP))
        .line_spacing(Dp(32.0))
        .children(move || {
            for (index, component) in COMPONENTS.iter().enumerate() {
                if component.category == category {
                    component_card(index, card_width, on_open);
                }
            }
        });
}

#[tessera]
fn component_card(index: usize, width: Dp, on_open: CallbackWith<usize>) {
    let component = COMPONENTS[index];
    let state = remember(ExampleValues::default);
    let colors = label_colors(Appearance::Light);
    column()
        .modifier(
            Modifier::new()
                .width(width)
                .semantics(SemanticsArgs {
                    test_tag: Some(format!("hig-component-card-{index}")),
                    ..Default::default()
                })
                .clickable(ClickableArgs {
                    on_click: Callback::new(move || on_open.call(index)),
                    role: Some(Role::Button),
                    label: Some(I18N.format(
                        "ui-open-value-specifications",
                        [("v0", I18N.text(component.title).into())],
                    )),
                    ..Default::default()
                }),
        )
        .children(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .size(width, Dp(PREVIEW_HEIGHT))
                        .clip_shape(Shape::rounded_rectangle(Dp(12.0)))
                        .background(preview_fill())
                        .clear_and_set_semantics(SemanticsArgs {
                            hidden: true,
                            ..Default::default()
                        }),
                )
                .children(move || {
                    cell(index, 0, 0, 0, state, width, Dp(PREVIEW_HEIGHT));
                    // The preview is a picture of the control; clicks open the article instead.
                    spacer().modifier(Modifier::new().fill_max_size().clickable(ClickableArgs {
                        on_click: Callback::new(move || on_open.call(index)),
                        keyboard_activation: false,
                        focus_properties: Some(FocusProperties {
                            can_focus: false,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }));
                });
            gap(Dp(12.0));
            text(I18N.text(component.title))
                .style(type_style(17.0, 22.0, 600))
                .color(colors[0]);
            gap(Dp(4.0));
            text(I18N.text(component.description))
                .style(type_style(13.0, 18.0, 400))
                .color(colors[1]);
        });
}

#[tessera]
fn hig_article(index: usize, on_open: CallbackWith<usize>) {
    let component = COMPONENTS[index];
    let state = remember(ExampleValues::default);
    let width = remember(|| Dp(900.0));
    hig_theme().child(move || {
        presentation_host().child(move || {
            viewport().child(move || {
                column().modifier(track_width(width)).children(move || {
                    header(component.title.into(), component.description.into());
                    for section in 0..component.sections.len() {
                        gap(Dp(40.0));
                        padded(move || heading(component.sections[section].title.into(), 22.0, 2));
                        gap(Dp(16.0));
                        windows(index, section, state, width.get());
                    }
                    gap(Dp(56.0));
                    padded(move || {
                        row()
                            .modifier(Modifier::new().fill_max_width())
                            .children(move || {
                                neighbour(index, false, on_open);
                                spacer().modifier(Modifier::new().width(Dp(GAP)));
                                neighbour(index, true, on_open);
                            });
                    });
                    gap(Dp(64.0));
                });
            });
        });
    });
}

/// The light and dark windows of one section, side by side when both fit.
#[tessera]
fn windows(index: usize, section: usize, state: ExampleState, width: Dp) {
    let spec = COMPONENTS[index].sections[section];
    let unlabelled = spec.rows[0].is_empty();
    let label_width = if unlabelled { 0.0 } else { ROW_LABEL_WIDTH };
    let content = (width.0 - 2.0 * MARGIN).max(0.0);
    let needed = label_width + spec.columns.len() as f64 * MIN_CELL_WIDTH + 2.0 * WINDOW_PADDING;
    let side_by_side = (content - GAP) / 2.0 >= needed;
    let window_width = Dp(if side_by_side {
        (content - GAP) / 2.0
    } else {
        content
    });
    padded(move || {
        if side_by_side {
            row().children(move || {
                window_pair(index, section, state, window_width, true);
            });
        } else {
            column().children(move || {
                window_pair(index, section, state, window_width, false);
            });
        }
    });
}

#[tessera]
fn window_pair(index: usize, section: usize, state: ExampleState, width: Dp, horizontal: bool) {
    window(index, section, Appearance::Light, state, width);
    gap_both(Dp(GAP), horizontal);
    window(index, section, Appearance::Dark, state, width);
}

#[tessera]
fn window(index: usize, section: usize, appearance: Appearance, state: ExampleState, width: Dp) {
    let title = if appearance == Appearance::Dark {
        "ui-hig-dark"
    } else {
        "ui-hig-light"
    };
    let spec = COMPONENTS[index].sections[section];
    let dark = appearance == Appearance::Dark;
    let shape = Shape::rounded_rectangle(Dp(10.0));
    let label_width = if spec.rows[0].is_empty() {
        0.0
    } else {
        ROW_LABEL_WIDTH
    };
    let cell_width =
        Dp(((width.0 - 2.0 * WINDOW_PADDING - label_width) / spec.columns.len() as f64).max(0.0));
    hig_theme().theme(HigTheme { appearance }).child(move || {
        let colors = label_colors(appearance);
        column()
            .modifier(
                Modifier::new()
                    .width(width)
                    .clip_shape(shape)
                    .background(if dark { srgb(30, 30, 30) } else { Color::WHITE })
                    .border_with_shape(
                        Dp(1.0),
                        if dark {
                            Color::WHITE.with_alpha(0.16)
                        } else {
                            Color::BLACK.with_alpha(0.12)
                        },
                        shape,
                    )
                    .semantics(SemanticsArgs {
                        role: Some(Role::Group),
                        label: Some(I18N.text(title)),
                        test_tag: Some(format!("hig-window-{section}-{}", dark as u8)),
                        ..Default::default()
                    }),
            )
            .children(move || {
                title_bar(I18N.text(title), colors[1]);
                column()
                    .modifier(Modifier::new().padding_all(Dp(WINDOW_PADDING)))
                    .children(move || {
                        if !spec.columns[0].is_empty() {
                            row().children(move || {
                                spacer().modifier(Modifier::new().width(Dp(label_width)));
                                for column in spec.columns {
                                    grid_label(label(column), cell_width, colors[1], true);
                                }
                            });
                            gap(Dp(4.0));
                        }
                        for row_index in 0..spec.rows.len() {
                            row()
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .children(move || {
                                    if label_width > 0.0 {
                                        grid_label(
                                            label(spec.rows[row_index]),
                                            Dp(label_width),
                                            colors[1],
                                            false,
                                        );
                                    }
                                    for column in 0..spec.columns.len() {
                                        cell(
                                            index,
                                            section,
                                            row_index,
                                            column,
                                            state,
                                            cell_width,
                                            Dp(spec.height),
                                        );
                                    }
                                });
                        }
                    });
            });
    });
}

#[tessera]
fn title_bar(title: String, color: Color) {
    boxed()
        .modifier(Modifier::new().fill_max_width().height(Dp(28.0)))
        .children(move || {
            row()
                .modifier(
                    Modifier::new()
                        .align(Alignment::CenterStart)
                        .padding_symmetric(Dp(12.0), Dp::ZERO),
                )
                .children(|| {
                    for (position, color) in [
                        srgb(0xFF, 0x5F, 0x57),
                        srgb(0xFE, 0xBC, 0x2E),
                        srgb(0x28, 0xC8, 0x40),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        if position > 0 {
                            spacer().modifier(Modifier::new().width(Dp(8.0)));
                        }
                        spacer().modifier(
                            Modifier::new()
                                .size(Dp(12.0), Dp(12.0))
                                .background_with_shape(color, Shape::Ellipse),
                        );
                    }
                });
            text(title.clone())
                .style(type_style(13.0, 16.0, 600))
                .color(color)
                .modifier(Modifier::new().align(Alignment::Center));
        });
}

#[tessera]
fn grid_label(content: String, width: Dp, color: Color, centered: bool) {
    boxed()
        .alignment(if centered {
            Alignment::Center
        } else {
            Alignment::CenterStart
        })
        .modifier(Modifier::new().width(width))
        .children(move || {
            text(content.clone())
                .style(type_style(11.0, 14.0, 500))
                .color(color);
        });
}

#[tessera]
fn neighbour(index: usize, next: bool, on_open: CallbackWith<usize>) {
    let target = if next {
        (index + 1 < COMPONENTS.len()).then_some(index + 1)
    } else {
        index.checked_sub(1)
    };
    let direction = if next {
        "ui-hig-next"
    } else {
        "ui-hig-previous"
    };
    let colors = label_colors(Appearance::Light);
    let mut modifier = Modifier::new().weight(1.0);
    if let Some(target) = target {
        modifier = modifier
            .clip_shape(Shape::rounded_rectangle(Dp(12.0)))
            .background(preview_fill())
            .clickable(ClickableArgs {
                on_click: Callback::new(move || on_open.call(target)),
                role: Some(Role::Button),
                label: Some(I18N.text(direction)),
                ..Default::default()
            });
    }
    column().modifier(modifier).children(move || {
        if let Some(target) = target {
            column()
                .modifier(Modifier::new().padding_all(Dp(16.0)))
                .children(move || {
                    text(I18N.text(direction))
                        .style(type_style(13.0, 18.0, 400))
                        .color(colors[1]);
                    gap(Dp(4.0));
                    text(I18N.text(COMPONENTS[target].title))
                        .style(type_style(17.0, 22.0, 600))
                        .color(colors[0]);
                });
        }
    });
}

#[tessera]
fn header(title: String, description: String) {
    let colors = label_colors(Appearance::Light);
    column()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_symmetric(Dp(MARGIN), Dp::ZERO),
        )
        .children(move || {
            gap(Dp(48.0));
            text(I18N.text(&title))
                .style(type_style(34.0, 41.0, 700))
                .color(colors[0])
                .modifier(Modifier::new().clear_and_set_semantics(SemanticsArgs {
                    role: Some(Role::Heading),
                    heading_level: Some(1),
                    ..Default::default()
                }));
            gap(Dp(8.0));
            text(I18N.text(&description))
                .style(type_style(17.0, 24.0, 400))
                .color(colors[1]);
        });
}

#[tessera]
fn heading(key: String, size: f64, level: u32) {
    text(I18N.text(&key))
        .style(type_style(size, size + 6.0, 700))
        .color(label_colors(Appearance::Light)[0])
        .modifier(Modifier::new().clear_and_set_semantics(SemanticsArgs {
            role: Some(Role::Heading),
            heading_level: Some(level),
            ..Default::default()
        }));
}

#[tessera]
fn padded(child: RenderSlot) {
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_symmetric(Dp(MARGIN), Dp::ZERO),
        )
        .children_shared(child);
}

#[tessera]
fn gap(height: Dp) {
    spacer().modifier(Modifier::new().height(height));
}

#[tessera]
fn gap_both(size: Dp, horizontal: bool) {
    spacer().modifier(if horizontal {
        Modifier::new().width(size)
    } else {
        Modifier::new().height(size)
    });
}

#[derive(Clone, Copy, Default, PartialEq)]
struct ScrollState {
    offset: f32,
    limit: f32,
    viewport: f32,
}

impl ScrollState {
    fn move_to(&mut self, offset: f32) -> f32 {
        let previous = self.offset;
        self.offset = offset.clamp(0.0, self.limit);
        self.offset - previous
    }
}

#[derive(Clone, Copy, PartialEq)]
struct ScrollLayout(State<ScrollState>);

impl LayoutModifier for ScrollLayout {
    fn measure(
        &self,
        input: &LayoutModifierInput<'_>,
        child: &mut dyn LayoutModifierChild,
    ) -> Result<LayoutResult, MeasurementError> {
        let parent = input.layout_input.parent_constraint();
        let mut constraints = *parent.as_ref();
        constraints.height = AxisConstraint::NONE;
        let measured = child.measure(&constraints)?;
        let width = parent.width().clamp(measured.width);
        let height = parent.height().clamp(measured.height);
        let offset = input.placement(|| {
            let limit = (measured.height.0 - height.0).max(0) as f32;
            if self
                .0
                .with(|scroll| scroll.limit != limit || scroll.viewport != height.to_f32())
            {
                self.0.with_mut(|scroll| {
                    scroll.limit = limit;
                    scroll.viewport = height.to_f32();
                    scroll.move_to(scroll.offset);
                });
            }
            self.0.with(|scroll| scroll.offset)
        });
        child.place(PxPosition::from_f32_xy(0.0, -offset));
        Ok(LayoutResult::new(tessera_ui::ComputedData {
            width,
            height,
        }))
    }
}

#[derive(Clone, Copy, PartialEq)]
struct ScrollKeys(State<ScrollState>);

impl KeyboardInputModifierNode for ScrollKeys {
    fn on_keyboard_input(&self, input: KeyboardInput<'_>) {
        if input.key_modifiers.control_key()
            || input.key_modifiers.alt_key()
            || input.key_modifiers.super_key()
        {
            return;
        }
        input.keyboard_events.retain(|event| {
            let Key::Named(key) = event.logical_key else {
                return true;
            };
            if !matches!(
                key,
                NamedKey::PageDown | NamedKey::PageUp | NamedKey::Home | NamedKey::End
            ) {
                return true;
            }
            if event.state == ElementState::Pressed {
                self.0.with_mut(|scroll| {
                    let next = match key {
                        NamedKey::PageDown => scroll.offset + scroll.viewport * 0.9,
                        NamedKey::PageUp => scroll.offset - scroll.viewport * 0.9,
                        NamedKey::Home => 0.0,
                        NamedKey::End => scroll.limit,
                        _ => unreachable!(),
                    };
                    scroll.move_to(next);
                });
            }
            false
        });
    }
}

#[tessera]
fn viewport(scroll: Option<State<ScrollState>>, child: Option<RenderSlot>) {
    let child = child.unwrap_or_default();
    let scroll = if let Some(scroll) = scroll {
        scroll
    } else {
        remember(ScrollState::default)
    };
    let dark = use_context::<HigTheme>().unwrap().get().appearance == Appearance::Dark;
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_size()
                .background(if dark { srgb(30, 30, 30) } else { Color::WHITE })
                .clip_to_bounds()
                .focus_group(None)
                .focus_reveal_handler(move |request: FocusRevealRequest| {
                    let target = request.local_target_bounds();
                    let viewport = request.local_viewport_bounds();
                    scroll.with_mut(|scroll| {
                        let delta = if target.y < viewport.y {
                            target.y - viewport.y
                        } else if target.y + target.height > viewport.y + viewport.height {
                            target.y + target.height - viewport.y - viewport.height
                        } else {
                            0.0
                        };
                        scroll.move_to(scroll.offset + delta) != 0.0
                    })
                })
                .push_keyboard_input(ScrollKeys(scroll))
                .push_layout(ScrollLayout(scroll))
                .pointer_input(scroll, move |mut scope| async move {
                    detect_scroll_gestures(
                        &mut scope,
                        ScrollGestureConfig::default(),
                        move |pointer, _, event| {
                            event.normalize_to_pixels();
                            let scale = pointer.local_to_window.scale();
                            event.delta_y += scale
                                * scroll.with_mut(|scroll| {
                                    scroll.move_to(scroll.offset - event.delta_y / scale)
                                });
                        },
                    )
                    .await;
                }),
        )
        .children_shared(child);
}
