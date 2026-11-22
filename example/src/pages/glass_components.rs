use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    column::column,
    experiment::hig::{
        glass::{GlassAppearance, GlassMaterial, glass},
        glass_group::glass_group,
    },
    flow_row::flow_row,
    md3::{
        button::button,
        chip::chip,
        lazy_list::lazy_column,
        scrollable::{ScrollBarBehavior, ScrollableController, ScrollbarDefaults, scrollbar_v},
        slider::slider,
        surface::surface,
        switch::switch,
        text::text,
        text_field::text_field,
        text_input::TextInputController,
        theme::MaterialTheme,
    },
    modifier::ModifierExt as _,
    row::row,
    shape_def::Shape,
    spacer::spacer,
};
use tessera_foundation::modifier::{DragDelta, DraggableArgs, ModifierExt as _};
use tessera_shard::shard;
use tessera_ui::{
    Callback, CallbackWith, Color, Context, Dp, LayoutCoordinates, Modifier, RenderSlotWith, State,
    accesskit::Role, modifier::FocusModifierExt as _, remember, tessera, use_context,
};

use crate::res::I18N;

const STAGE_HEIGHT: Dp = Dp(152.0);
const PAGE_INSET: Dp = Dp(16.0);
const LENS_ORIGIN: [Dp; 2] = [Dp(32.0), Dp(32.0)];
const TILE_SIZE: Dp = Dp(64.0);
const TILE_SPACING: Dp = Dp(12.0);
const TILE_COLUMNS: usize = 8;
const TILE_ROWS: usize = 3;
const BACKDROP_PADDING: Dp = Dp(24.0);
const DEMO_WIDTH: Dp = Dp(TILE_SIZE.0 * TILE_COLUMNS as f64
    + TILE_SPACING.0 * (TILE_COLUMNS - 1) as f64
    + BACKDROP_PADDING.0 * 2.0);

const MATERIALS: [(&str, GlassMaterial); 2] = [
    ("ui-glass-regular", GlassMaterial::Regular),
    ("ui-glass-clear", GlassMaterial::Clear),
];
const APPEARANCES: [(&str, Option<GlassAppearance>); 3] = [
    ("ui-glass-theme", None),
    ("ui-glass-light", Some(GlassAppearance::Light)),
    ("ui-glass-dark", Some(GlassAppearance::Dark)),
];
const SHAPES: [&str; 2] = ["ui-glass-capsule", "ui-glass-rounded-rectangle"];
const ROLES: [(&str, Option<Role>); 4] = [
    ("ui-glass-auto", None),
    ("ui-glass-role-button", Some(Role::Button)),
    ("ui-glass-role-group", Some(Role::Group)),
    ("ui-glass-role-image", Some(Role::Image)),
];

const LAYOUT_NUMBERS: [Number; 4] = [
    Number::Width,
    Number::Height,
    Number::Opacity,
    Number::Padding,
];
const FUSION_NUMBERS: [Number; 2] = [Number::Spacing, Number::Gap];

#[derive(Clone, Copy, PartialEq)]
enum Number {
    Width,
    Height,
    Opacity,
    Corner,
    Padding,
    Spacing,
    Gap,
}

impl Number {
    const COUNT: usize = Self::Gap as usize + 1;

    fn spec(self) -> (&'static str, f32, f32, &'static str) {
        match self {
            Self::Width => ("ui-glass-width", 48.0, 224.0, " dp"),
            Self::Height => ("ui-glass-height", 36.0, 144.0, " dp"),
            Self::Opacity => ("ui-glass-opacity", 0.0, 1.0, ""),
            Self::Corner => ("ui-glass-corner-radius", 0.0, 72.0, " dp"),
            Self::Padding => ("ui-glass-padding", 0.0, 32.0, " dp"),
            Self::Spacing => ("ui-glass-spacing", 0.0, 48.0, " dp"),
            Self::Gap => ("ui-glass-gap", 0.0, 48.0, " dp"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct LensSettings {
    numbers: [f32; Number::COUNT],
    material: usize,
    appearance: usize,
    shape: usize,
    role: usize,
    click: bool,
    block_input: bool,
    focusable: bool,
    show_content: bool,
}

impl Default for LensSettings {
    fn default() -> Self {
        let mut numbers = [0.0; Number::COUNT];
        for (parameter, value) in [
            (Number::Width, 156.0),
            (Number::Height, 80.0),
            (Number::Opacity, 1.0),
            (Number::Corner, 24.0),
            (Number::Padding, 8.0),
            (Number::Spacing, 16.0),
            (Number::Gap, 8.0),
        ] {
            numbers[parameter as usize] = value;
        }
        Self {
            numbers,
            material: 0,
            appearance: 0,
            shape: 0,
            role: 0,
            click: false,
            block_input: false,
            focusable: false,
            show_content: true,
        }
    }
}

impl LensSettings {
    fn value(self, parameter: Number) -> f32 {
        self.numbers[parameter as usize]
    }

    fn dp(self, parameter: Number) -> Dp {
        Dp(f64::from(self.value(parameter)))
    }

    fn corner_radius(self) -> Option<Dp> {
        (self.shape == 1).then(|| self.dp(Number::Corner))
    }

    fn lens_size(self, stage_width: Dp) -> [Dp; 2] {
        [
            Dp(self.dp(Number::Width).0.min(stage_width.0)),
            self.dp(Number::Height),
        ]
    }
}

#[derive(Clone, Copy, PartialEq)]
struct LensEditors {
    content: State<TextInputController>,
    label: State<TextInputController>,
    description: State<TextInputController>,
}

fn lens_editor(initial: &str) -> TextInputController {
    let mut editor = TextInputController::new(Dp(16.0), None);
    editor.set_text(initial);
    editor
}

fn optional_text(editor: State<TextInputController>) -> Option<String> {
    let value = editor.with(TextInputController::text);
    (!value.is_empty()).then_some(value)
}

fn bounded_position(position: [Dp; 2], size: [Dp; 2], stage_width: Dp) -> [Dp; 2] {
    [
        Dp(position[0]
            .0
            .clamp(0.0, (stage_width.0 - size[0].0).max(0.0))),
        Dp(position[1]
            .0
            .clamp(0.0, (STAGE_HEIGHT.0 - size[1].0).max(0.0))),
    ]
}

#[shard]
pub fn glass_components_page() {
    let settings = remember(LensSettings::default);
    let scroll = remember(ScrollableController::new);
    let position = remember(|| LENS_ORIGIN);
    let clicks = remember(|| 0_u32);
    let editors = LensEditors {
        content: remember(|| lens_editor(&I18N.text("ui-glass-lens"))),
        label: remember(|| lens_editor(&I18N.text("ui-glass-lens"))),
        description: remember(|| lens_editor("")),
    };
    let theme = use_context::<MaterialTheme>().unwrap();

    column()
        .modifier(
            Modifier::new()
                .fill_max_size()
                .padding_symmetric(Dp::ZERO, PAGE_INSET),
        )
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            glass_preview(settings, position, clicks, editors);
            spacer().modifier(Modifier::new().height(Dp(12.0)));
            boxed()
                .alignment(Alignment::TopCenter)
                .modifier(Modifier::new().fill_max_width().weight(1.0))
                .children(move || {
                    lazy_column()
                        .modifier(
                            Modifier::new()
                                .padding_symmetric(PAGE_INSET, Dp::ZERO)
                                .size_in(None, Some(DEMO_WIDTH), None, None)
                                .fill_max_size(),
                        )
                        .viewport_controller(scroll)
                        .scrollbar_behavior(ScrollBarBehavior::Hidden)
                        .item_spacing(Dp(12.0))
                        .item(move || {
                            text(I18N.text("ui-glass-components"))
                                .style(theme.with(|t| t.typography.headline_large));
                        })
                        .item(move || {
                            text(I18N.text("ui-drag-the-glass-lens"))
                                .style(theme.with(|t| t.typography.body_large));
                        })
                        .item(move || {
                            flow_row()
                                .item_spacing(Dp(8.0))
                                .line_spacing(Dp(8.0))
                                .children(move || {
                                    button()
                                        .tonal()
                                        .on_click(move || position.set(LENS_ORIGIN))
                                        .child(|| {
                                            text(I18N.text("ui-reset-lens-position"));
                                        });
                                    button()
                                        .outlined()
                                        .on_click(move || {
                                            settings.set(LensSettings::default());
                                            position.set(LENS_ORIGIN);
                                            clicks.set(0);
                                            let name = I18N.text("ui-glass-lens");
                                            editors
                                                .content
                                                .with_mut(|editor| editor.set_text(&name));
                                            editors.label.with_mut(|editor| editor.set_text(&name));
                                            editors
                                                .description
                                                .with_mut(|editor| editor.set_text(""));
                                        })
                                        .child(|| {
                                            text(I18N.text("ui-glass-reset-settings"));
                                        });
                                });
                        })
                        .item(|| {
                            section_title("ui-glass-material-section");
                        })
                        .item(move || {
                            choices(
                                "ui-glass-material",
                                labels(&MATERIALS),
                                settings.with(|s| s.material),
                            )
                            .on_select(move |index| settings.with_mut(|s| s.material = index));
                        })
                        .item(move || {
                            choices(
                                "ui-glass-appearance",
                                labels(&APPEARANCES),
                                settings.with(|s| s.appearance),
                            )
                            .on_select(move |index| settings.with_mut(|s| s.appearance = index));
                        })
                        .item(|| {
                            section_title("ui-glass-shape-section");
                        })
                        .item(move || {
                            choices(
                                "ui-glass-shape",
                                SHAPES.to_vec(),
                                settings.with(|s| s.shape),
                            )
                            .on_select(move |index| settings.with_mut(|s| s.shape = index));
                        })
                        .item(move || {
                            number_control(settings, Number::Corner)
                                .enabled(settings.with(|s| s.shape == 1));
                        })
                        .items_with_key_shared(
                            LAYOUT_NUMBERS.len(),
                            |index| ("glass-number", LAYOUT_NUMBERS[index] as usize),
                            RenderSlotWith::new(move |index| {
                                number_control(settings, LAYOUT_NUMBERS[index]);
                            }),
                        )
                        .item(|| {
                            section_title("ui-glass-fusion-section");
                        })
                        .items_with_key_shared(
                            FUSION_NUMBERS.len(),
                            |index| ("glass-number", FUSION_NUMBERS[index] as usize),
                            RenderSlotWith::new(move |index| {
                                number_control(settings, FUSION_NUMBERS[index]);
                            }),
                        )
                        .item(|| {
                            section_title("ui-glass-interaction-section");
                        })
                        .item(move || {
                            toggle("ui-glass-click-action", settings.with(|s| s.click))
                                .on_toggle(move |checked| settings.with_mut(|s| s.click = checked));
                        })
                        .item(move || {
                            text(
                                I18N.format(
                                    "ui-glass-click-count",
                                    [("count", clicks.get().into())],
                                ),
                            );
                        })
                        .item(move || {
                            toggle("ui-glass-block-input", settings.with(|s| s.block_input))
                                .on_toggle(move |checked| {
                                    settings.with_mut(|s| s.block_input = checked)
                                });
                        })
                        .item(|| {
                            section_title("ui-glass-content-section");
                        })
                        .item(move || {
                            toggle("ui-glass-show-content", settings.with(|s| s.show_content))
                                .on_toggle(move |checked| {
                                    settings.with_mut(|s| s.show_content = checked)
                                });
                        })
                        .item(move || {
                            editor_field("ui-glass-content-text", editors.content)
                                .enabled(settings.with(|s| s.show_content));
                        })
                        .item(move || {
                            choices(
                                "ui-glass-accessibility-role",
                                labels(&ROLES),
                                settings.with(|s| s.role),
                            )
                            .on_select(move |index| settings.with_mut(|s| s.role = index));
                        })
                        .item(move || {
                            editor_field("ui-glass-accessibility-label", editors.label);
                        })
                        .item(move || {
                            editor_field("ui-glass-accessibility-description", editors.description);
                        })
                        .item(move || {
                            toggle("ui-glass-focusable", settings.with(|s| s.focusable)).on_toggle(
                                move |checked| settings.with_mut(|s| s.focusable = checked),
                            );
                        });
                    boxed()
                        .modifier(
                            Modifier::new()
                                .align(Alignment::TopEnd)
                                .width(ScrollbarDefaults::THICKNESS)
                                .fill_max_height(),
                        )
                        .children(move || {
                            scrollbar_v().controller(scroll);
                        });
                });
        });
}

#[tessera]
fn glass_preview(
    settings: State<LensSettings>,
    position: State<[Dp; 2]>,
    clicks: State<u32>,
    editors: LensEditors,
) {
    let theme = use_context::<MaterialTheme>().unwrap();
    let stage_width = remember(|| DEMO_WIDTH);
    let shape = theme.with(|t| t.shapes.extra_large);
    surface()
        .style(theme.with(|t| t.color_scheme.surface_container_low).into())
        .shape(shape)
        .modifier(
            Modifier::new()
                .padding_symmetric(PAGE_INSET, Dp::ZERO)
                .size_in(None, Some(DEMO_WIDTH), None, None)
                .fill_max_width()
                .height(STAGE_HEIGHT),
        )
        .child(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .clip_shape(shape)
                        .on_layout_coordinates(move |coordinates: LayoutCoordinates| {
                            let width = coordinates.local_size.width.to_dp();
                            if stage_width.get() != width {
                                stage_width.set(width);
                            }
                        }),
                )
                .children(move || {
                    glass_backdrop();
                    glass_pair(settings);
                    glass_lens(settings, position, stage_width, clicks, editors);
                });
        });
}

#[tessera]
fn glass_lens(
    settings: State<LensSettings>,
    position: State<[Dp; 2]>,
    stage_width: State<Dp>,
    clicks: State<u32>,
    editors: LensEditors,
) {
    let values = settings.get();
    let [width, height] = values.lens_size(stage_width.get());
    let [x, y] = bounded_position(position.get(), [width, height], stage_width.get());
    let appearance = APPEARANCES[values.appearance].1;
    let theme = use_context::<MaterialTheme>().unwrap();
    let dark = is_dark(appearance, theme);
    let lens = glass()
        .modifier(
            Modifier::new()
                .size(width, height)
                .offset(x, y)
                .alpha(values.value(Number::Opacity))
                .draggable(DraggableArgs {
                    on_drag_delta: CallbackWith::new(move |delta: DragDelta| {
                        let width = stage_width.get();
                        let size = settings.get().lens_size(width);
                        let [x, y] = bounded_position(position.get(), size, width);
                        position.set(bounded_position(
                            [Dp(x.0 + delta.x.to_dp().0), Dp(y.0 + delta.y.to_dp().0)],
                            size,
                            width,
                        ));
                    }),
                    ..Default::default()
                }),
        )
        .material(MATERIALS[values.material].1)
        .appearance(if dark {
            GlassAppearance::Dark
        } else {
            GlassAppearance::Light
        })
        .corner_radius_optional(values.corner_radius())
        .padding(values.dp(Number::Padding))
        .on_click_optional(
            values
                .click
                .then(|| Callback::new(move || clicks.set(clicks.get().saturating_add(1)))),
        )
        .block_input(values.block_input)
        .accessibility_role_optional(ROLES[values.role].1)
        .accessibility_label_optional(optional_text(editors.label))
        .accessibility_description_optional(optional_text(editors.description))
        .accessibility_focusable(values.focusable);
    if values.show_content {
        lens.child(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(Modifier::new().fill_max_size().clip_to_bounds())
                .children(move || {
                    text(editors.content.with(TextInputController::text)).color(if dark {
                        Color::WHITE
                    } else {
                        Color::BLACK
                    });
                });
        });
    }
}

fn is_dark(appearance: Option<GlassAppearance>, theme: Context<MaterialTheme>) -> bool {
    match appearance {
        Some(appearance) => appearance == GlassAppearance::Dark,
        None => theme.with(|theme| theme.color_scheme.is_dark),
    }
}

/// Two capsules in a fused group at the right edge of the stage.
#[tessera]
fn glass_pair(settings: State<LensSettings>) {
    let values = settings.get();
    let theme = use_context::<MaterialTheme>().unwrap();
    let appearance = if is_dark(APPEARANCES[values.appearance].1, theme) {
        GlassAppearance::Dark
    } else {
        GlassAppearance::Light
    };
    glass_group()
        .material(MATERIALS[values.material].1)
        .appearance(appearance)
        .spacing(values.dp(Number::Spacing))
        .gap(values.dp(Number::Gap))
        .modifier(
            Modifier::new()
                .align(Alignment::BottomEnd)
                .padding_all(Dp(16.0)),
        )
        .child(|| {
            spacer().modifier(Modifier::new().size(Dp(56.0), Dp(40.0)));
            spacer().modifier(Modifier::new().size(Dp(56.0), Dp(40.0)));
        });
}

fn labels<T>(choices: &[(&'static str, T)]) -> Vec<&'static str> {
    choices.iter().map(|(label, _)| *label).collect()
}

#[tessera]
fn section_title(#[prop(into)] label: String) {
    let theme = use_context::<MaterialTheme>().unwrap();
    text(I18N.text(&label)).style(theme.with(|t| t.typography.title_medium));
}

#[tessera]
fn choices(
    #[prop(into)] label: String,
    options: Vec<&'static str>,
    selected: usize,
    on_select: Option<CallbackWith<usize>>,
) {
    column().children(move || {
        text(I18N.text(&label));
        spacer().modifier(Modifier::new().height(Dp(4.0)));
        let options = options.clone();
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(4.0))
            .children(move || {
                for (index, label) in options.iter().enumerate() {
                    chip()
                        .filter(I18N.text(label))
                        .selected(index == selected)
                        .on_click(move || {
                            if let Some(on_select) = on_select {
                                on_select.call(index);
                            }
                        });
                }
            });
    });
}

#[tessera]
fn number_control(settings: State<LensSettings>, parameter: Number, enabled: Option<bool>) {
    let enabled = enabled.unwrap_or(true);
    let (label, min, max, unit) = parameter.spec();
    let value = settings.with(|s| s.value(parameter));
    let formatted = format!("{value:.2}{unit}");
    let label = I18N.text(label);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            row()
                .modifier(Modifier::new().fill_max_width())
                .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                .children({
                    let label = label.clone();
                    let formatted = formatted.clone();
                    move || {
                        text(label.clone());
                        text(formatted.clone());
                    }
                });
            slider((value - min) / (max - min))
                .modifier(Modifier::new().fill_max_width())
                .disabled(!enabled)
                .value_label(formatted.clone())
                .accessibility_label(label.clone())
                .on_change(move |fraction| {
                    let value = min + (max - min) * fraction;
                    settings.with_mut(|s| s.numbers[parameter as usize] = value);
                });
        });
}

#[tessera]
fn toggle(#[prop(into)] label: String, checked: bool, on_toggle: Option<CallbackWith<bool>>) {
    row()
        .modifier(Modifier::new().fill_max_width())
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            text(I18N.text(&label)).modifier(Modifier::new().weight(1.0));
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            switch(checked)
                .accessibility_label(I18N.text(&label))
                .on_toggle_optional(on_toggle);
        });
}

#[tessera]
fn editor_field(
    #[prop(into)] label: String,
    controller: State<TextInputController>,
    enabled: Option<bool>,
) {
    text_field()
        .outlined()
        .modifier(Modifier::new().fill_max_width())
        .label(I18N.text(&label))
        .controller(controller)
        .enabled(enabled.unwrap_or(true));
}

#[tessera]
fn glass_backdrop() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let palette: [Color; 6] = [
        scheme.primary,
        scheme.tertiary,
        scheme.secondary,
        scheme.primary_container,
        scheme.tertiary_container,
        scheme.secondary_container,
    ];
    flow_row()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_all(BACKDROP_PADDING),
        )
        .item_spacing(TILE_SPACING)
        .line_spacing(TILE_SPACING)
        .children(move || {
            for index in 0..TILE_COLUMNS * TILE_ROWS {
                let line = index / TILE_COLUMNS;
                let color = palette[(index % TILE_COLUMNS + line * 2) % palette.len()];
                spacer().modifier(
                    Modifier::new()
                        .size(TILE_SIZE, TILE_SIZE)
                        .background_with_shape(color, Shape::rounded_rectangle(Dp(16.0))),
                );
            }
        });
}
