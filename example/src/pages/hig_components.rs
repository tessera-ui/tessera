use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    experiment::hig::{
        button::button,
        checkbox::{CheckboxState, checkbox},
        disclosure_button::disclosure_button,
        glass::{GlassAppearance, GlassMaterial, glass},
        group_box::group_box,
        help_button::help_button,
        level_indicator::capacity_indicator,
        progress::progress_bar,
        radio_button::{radio_button, radio_group},
        segmented_control::{Segment, segmented_control},
        separator::separator,
        slider::slider,
        stepper::stepper,
        switch::switch,
        text::text,
        text_field::text_field,
        theme::{Appearance, HigTheme, TextStyle, hig_theme},
    },
    flow_row::flow_row,
    image::{ImageData, image},
    modifier::ModifierExt as _,
    row::row,
    spacer::spacer,
};
use tessera_foundation::{
    modifier::{ModifierExt as _, SemanticsArgs},
    pointer_gestures::{ScrollGestureConfig, detect_scroll_gestures},
};
use tessera_shard::{RouterController, shard, use_router};
use tessera_ui::{
    AxisConstraint, CallbackWith, Color, Dp, KeyboardInput, KeyboardInputModifierNode,
    LayoutModifier, LayoutModifierChild, LayoutModifierInput, LayoutResult, MeasurementError,
    Model, ModelInit, Modifier, PxPosition, RenderSlot, State,
    accesskit::Role,
    focus::FocusRevealRequest,
    key,
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
    winit::{
        event::ElementState,
        keyboard::{Key, NamedKey},
    },
};

use crate::res::I18N;

#[cfg(test)]
#[path = "hig_components_tests.rs"]
mod tests;

const COMPONENTS: &[(&str, &str)] = &[
    ("ui-buttons", "ui-hig-buttons-description"),
    ("ui-hig-help-buttons", "ui-hig-help-description"),
    ("ui-hig-disclosure-buttons", "ui-hig-disclosure-description"),
    ("ui-checkbox", "ui-hig-checkbox-description"),
    ("ui-radio-button", "ui-hig-radio-description"),
    ("ui-switch", "ui-hig-switch-description"),
    ("ui-hig-segmented-controls", "ui-hig-segments-description"),
    ("ui-sliders", "ui-hig-slider-description"),
    ("ui-hig-steppers", "ui-hig-stepper-description"),
    ("ui-progress-indicators", "ui-hig-progress-description"),
    ("ui-hig-capacity-indicators", "ui-hig-capacity-description"),
    ("ui-hig-separators", "ui-hig-separator-description"),
    ("ui-hig-group-boxes", "ui-hig-group-description"),
    ("ui-hig-glass", "ui-hig-glass-description"),
    ("ui-hig-text", "ui-hig-text-description"),
    ("ui-hig-text-fields", "ui-hig-text-field-description"),
];

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

pub(crate) fn current_article(router: &RouterController) -> Option<usize> {
    (router.last()? as &dyn std::any::Any)
        .downcast_ref::<HigArticlePageDestination>()
        .map(|article| article.index)
}

#[tessera]
fn hig_catalog(scroll: State<ScrollState>, width: State<Dp>, on_open: CallbackWith<usize>) {
    hig_theme().child(move || {
        viewport().scroll(scroll).child(move || {
            column()
                .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
                    move |coordinates: tessera_ui::LayoutCoordinates| {
                        let next = Dp::from(coordinates.local_size.width);
                        if width.get() != next {
                            width.set(next);
                        }
                    },
                ))
                .children(move || {
                    header(
                        "ui-hig-components".into(),
                        "ui-hig-introduction".into(),
                        width.get(),
                    );
                    let columns = if width.get().0 <= 600.0 {
                        1
                    } else if width.get().0 < 1295.0 {
                        2
                    } else {
                        3
                    };
                    for (category, start, end) in [
                        ("ui-hig-actions", 0, 3),
                        ("ui-hig-selection", 3, 7),
                        ("ui-hig-values", 7, 11),
                        ("ui-hig-content", 11, COMPONENTS.len()),
                    ] {
                        gap(Dp(72.0));
                        boxed()
                            .modifier(Modifier::new().padding_symmetric(Dp(33.0), Dp::ZERO))
                            .children(move || {
                                heading(
                                    I18N.text(category),
                                    Dp(if width.get().0 <= 600.0 { 28.0 } else { 32.0 }),
                                );
                            });
                        gap(Dp(24.0));
                        flow_row()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(8.0)))
                            .item_spacing(Dp(8.0))
                            .line_spacing(Dp(8.0))
                            .children(move || {
                                let card_width =
                                    Dp(((width.get().0 - 16.0 - (columns - 1) as f64 * 8.0)
                                        / columns as f64)
                                        .max(0.0));
                                for index in start..end {
                                    component_card(index, card_width, on_open);
                                }
                            });
                    }
                    gap(Dp(64.0));
                });
        });
    });
}

#[tessera]
fn component_card(index: usize, width: Dp, on_open: CallbackWith<usize>) {
    let (title, description) = COMPONENTS[index];
    group_box()
        .modifier(Modifier::new().width(width).semantics(SemanticsArgs {
            test_tag: Some(format!("hig-component-card-{index}")),
            ..Default::default()
        }))
        .child(move || {
            column()
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    preview(index, false);
                    column()
                        .modifier(Modifier::new().padding_all(Dp(24.0)))
                        .children(move || {
                            heading(I18N.text(title), Dp(24.0));
                            gap(Dp(8.0));
                            text(I18N.text(description))
                                .size(Dp(16.0))
                                .line_height(Dp(24.0));
                            gap(Dp(16.0));
                            button(I18N.text("ui-hig-open-example"))
                                .accessibility_label(I18N.format(
                                    "ui-open-value-specifications",
                                    [("v0", I18N.text(title).into())],
                                ))
                                .on_click(move || on_open.call(index));
                        });
                });
        });
}

#[tessera]
fn hig_article(index: usize, on_open: CallbackWith<usize>) {
    let (title, description) = COMPONENTS[index];
    let dark = remember(|| false);
    let tab = remember(|| 0);
    let width = remember(|| Dp(900.0));
    hig_theme()
        .theme(HigTheme {
            appearance: if dark.get() {
                Appearance::Dark
            } else {
                Appearance::Light
            },
        })
        .child(move || {
            viewport().child(move || {
                column()
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .on_layout_coordinates(
                                move |coordinates: tessera_ui::LayoutCoordinates| {
                                    let next = Dp::from(coordinates.local_size.width);
                                    if width.get() != next {
                                        width.set(next);
                                    }
                                },
                            )
                            .semantics(SemanticsArgs {
                                test_tag: Some(format!("hig-article-{index}")),
                                merge_descendants: true,
                                ..Default::default()
                            }),
                    )
                    .children(move || {
                        header(title.into(), description.into(), width.get());
                        gap(Dp(24.0));
                        flow_row()
                            .modifier(Modifier::new().padding_all(Dp(24.0)))
                            .item_spacing(Dp(24.0))
                            .line_spacing(Dp(16.0))
                            .children(move || {
                                segmented_control(vec![
                                    Segment::new(I18N.text("ui-hig-overview")),
                                    Segment::new(I18N.text("ui-hig-states")),
                                ])
                                .selected(tab.get())
                                .on_select(move |next| tab.set(next));
                                checkbox(if dark.get() {
                                    CheckboxState::On
                                } else {
                                    CheckboxState::Off
                                })
                                .label(I18N.text("ui-dark-theme"))
                                .on_value_change(move |next| dark.set(next == CheckboxState::On));
                            });
                        gap(Dp(32.0));
                        preview(index, false);
                        if tab.get() == 1 && matches!(index, 0..=8 | 10 | 15) {
                            gap(Dp(24.0));
                            heading(I18N.text("ui-hig-disabled"), Dp(24.0));
                            preview(index, true);
                        }
                        gap(Dp(32.0));
                        separator();
                        gap(Dp(24.0));
                        row().children(move || {
                            button(I18N.text("ui-hig-previous"))
                                .enabled(index > 0)
                                .on_click(move || on_open.call(index - 1));
                            spacer().modifier(Modifier::new().width(Dp(16.0)));
                            button(I18N.text("ui-hig-next"))
                                .enabled(index + 1 < COMPONENTS.len())
                                .on_click(move || on_open.call(index + 1));
                        });
                    });
            });
        });
}

#[tessera]
fn header(title: String, description: String, width: Dp) {
    let compact = width.0 <= 600.0;
    group_box()
        .modifier(Modifier::new().fill_max_width().padding_all(Dp(8.0)))
        .child(move || {
            let title = title.clone();
            let description = description.clone();
            column()
                .modifier(Modifier::new().fill_max_width().padding_all(Dp(if compact {
                    27.0
                } else {
                    51.0
                })))
                .children(move || {
                    text(I18N.text(title.clone()))
                        .style(TextStyle {
                            font_size: Dp(if compact { 45.0 } else { 88.0 }),
                            line_height: Dp(if compact { 52.0 } else { 96.0 }),
                            font_weight: 600,
                            ..Default::default()
                        })
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            role: Some(Role::Heading),
                            heading_level: Some(1),
                            ..Default::default()
                        }));
                    gap(Dp(16.0));
                    text(I18N.text(description.clone()))
                        .size(Dp(if compact { 16.0 } else { 22.0 }))
                        .line_height(Dp(if compact { 24.0 } else { 30.0 }));
                });
        });
}

#[tessera]
fn heading(label: String, size: Dp) {
    text(label)
        .style(TextStyle {
            font_size: size,
            line_height: Dp(size.0 + 8.0),
            font_weight: 600,
            ..Default::default()
        })
        .modifier(Modifier::new().semantics(SemanticsArgs {
            role: Some(Role::Heading),
            heading_level: Some(2),
            ..Default::default()
        }));
}

#[tessera]
fn gap(height: Dp) {
    spacer().modifier(Modifier::new().height(height));
}

#[tessera]
fn preview(index: usize, disabled: bool) {
    let checked = remember(|| CheckboxState::Off);
    let selected = remember(|| 0);
    let value = remember(|| if index == 8 { 5.0 } else { 0.5 });
    let enabled_setting = remember(|| false);
    let expanded = remember(|| false);
    let clicks = remember(|| 0);
    let theme = use_context::<HigTheme>().unwrap().get();
    let backdrop = remember(|| {
        let mut pixels = Vec::with_capacity(400 * 200 * 4);
        for y in 0..200 {
            for x in 0..400 {
                let checker = if (x / 16 + y / 16) % 2 == 0 { 28 } else { 0 };
                pixels.extend_from_slice(&[
                    70 + (x / 4) as u8 + checker,
                    100 + (y / 3) as u8,
                    180 - (x / 8) as u8,
                    255,
                ]);
            }
        }
        ImageData::from_rgba(pixels, 400, 200)
    });
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(200.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(I18N.text(if disabled {
                        "ui-hig-disabled-example"
                    } else {
                        "ui-hig-interactive-example"
                    })),
                    test_tag: Some(format!("hig-preview-{index}-{disabled}")),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            image()
                .painter(backdrop.get())
                .modifier(Modifier::new().fill_max_size());
            column()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || match index {
                    0 => {
                        button(I18N.text("ui-hig-action"))
                            .enabled(!disabled)
                            .default_action(true)
                            .on_click(move || clicks.set(clicks.get() + 1));
                        text(clicks.get().to_string());
                    }
                    1 => {
                        help_button()
                            .enabled(!disabled)
                            .accessibility_label(I18N.text("ui-hig-help-buttons"))
                            .on_click(move || expanded.set(!expanded.get()));
                        if expanded.get() {
                            text(I18N.text("ui-hig-help-content"));
                        }
                    }
                    2 => {
                        disclosure_button(expanded.get())
                            .enabled(!disabled)
                            .accessibility_label(I18N.text("ui-hig-details"))
                            .on_expanded_change(move |next| expanded.set(next));
                        if expanded.get() {
                            text(I18N.text("ui-hig-details"));
                        }
                    }
                    3 => {
                        checkbox(checked.get())
                            .enabled(!disabled)
                            .allows_mixed(true)
                            .label(I18N.text("ui-hig-option"))
                            .on_value_change(move |next| checked.set(next));
                    }
                    4 => {
                        radio_group()
                            .accessibility_label(I18N.text("ui-hig-choice"))
                            .child(move |group| {
                                column().children(move || {
                                    for index in 0..2 {
                                        radio_button(selected.get() == index)
                                            .group(group)
                                            .enabled(!disabled)
                                            .label(I18N.text(if index == 0 {
                                                "ui-hig-first"
                                            } else {
                                                "ui-hig-second"
                                            }))
                                            .on_select(move || selected.set(index));
                                    }
                                });
                            });
                    }
                    5 => {
                        switch(enabled_setting.get())
                            .enabled(!disabled)
                            .accessibility_label(I18N.text("ui-hig-option"))
                            .on_value_change(move |next| enabled_setting.set(next));
                    }
                    6 => {
                        segmented_control(vec![
                            Segment::new(I18N.text("ui-hig-first")),
                            Segment::new(I18N.text("ui-hig-second")),
                        ])
                        .enabled(!disabled)
                        .selected(selected.get())
                        .on_select(move |next| selected.set(next));
                    }
                    7 => {
                        slider(value.get())
                            .enabled(!disabled)
                            .modifier(Modifier::new().width(Dp(160.0)))
                            .accessibility_label(I18N.text("ui-hig-value"))
                            .on_value_change(move |next| value.set(next));
                    }
                    8 => {
                        stepper(value.get())
                            .range(0.0..=10.0)
                            .enabled(!disabled)
                            .accessibility_label(I18N.text("ui-hig-value"))
                            .on_value_change(move |next| value.set(next));
                        text(format!("{:.1}", value.get()));
                    }
                    9 => {
                        progress_bar(value.get())
                            .modifier(Modifier::new().width(Dp(160.0)))
                            .accessibility_label(I18N.text("ui-hig-value"));
                        slider(value.get())
                            .modifier(Modifier::new().width(Dp(160.0)))
                            .accessibility_label(I18N.text("ui-hig-progress-value"))
                            .on_value_change(move |next| value.set(next));
                    }
                    10 => {
                        capacity_indicator(value.get() * 5.0)
                            .enabled(!disabled)
                            .warning_value(3.0)
                            .critical_value(4.0)
                            .modifier(Modifier::new().width(Dp(160.0)))
                            .accessibility_label(I18N.text("ui-hig-value"));
                        slider(value.get())
                            .enabled(!disabled)
                            .modifier(Modifier::new().width(Dp(160.0)))
                            .accessibility_label(I18N.text("ui-hig-capacity-value"))
                            .on_value_change(move |next| value.set(next));
                    }
                    11 => {
                        text(I18N.text("ui-hig-first"));
                        separator().modifier(Modifier::new().width(Dp(160.0)));
                        text(I18N.text("ui-hig-second"));
                    }
                    12 => {
                        group_box()
                            .title(I18N.text("ui-hig-group-boxes"))
                            .child(|| {
                                text(I18N.text("ui-hig-details"));
                            });
                    }
                    13 => {
                        glass()
                            .material(GlassMaterial::Clear)
                            .appearance(if theme.appearance == Appearance::Dark {
                                GlassAppearance::Dark
                            } else {
                                GlassAppearance::Light
                            })
                            .padding(Dp(24.0))
                            .child(|| {
                                text(I18N.text("ui-hig-glass"));
                            });
                    }
                    14 => {
                        text(I18N.text("ui-hig-text")).size(Dp(24.0));
                        text(I18N.text("ui-hig-details"));
                    }
                    15 => {
                        text_field()
                            .enabled(!disabled)
                            .placeholder(I18N.text("ui-hig-search"))
                            .accessibility_label(I18N.text("ui-hig-search"));
                    }
                    _ => unreachable!(),
                });
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
                .background(if dark {
                    Color::from_rgb(0.025, 0.025, 0.025)
                } else {
                    Color::from_rgb(0.96, 0.96, 0.97)
                })
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
