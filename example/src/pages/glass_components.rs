use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    button::button,
    chip::chip,
    column::column,
    flow_row::flow_row,
    fluid_glass::{GlassAppearance, GlassBorder, GlassMaterial, fluid_glass},
    lazy_list::lazy_column,
    modifier::{DragDelta, ModifierExt},
    row::row,
    scrollable::{ScrollBarBehavior, ScrollableController, ScrollbarDefaults, scrollbar_v},
    shape_def::Shape,
    slider::slider,
    spacer::spacer,
    surface::surface,
    switch::switch,
    text::text,
    text_field::text_field,
    text_input::TextInputController,
    theme::MaterialTheme,
};
use tessera_shard::shard;
use tessera_ui::{
    Callback, CallbackWith, Color, Dp, LayoutCoordinates, Modifier, Px, RenderSlotWith, State,
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

const MATERIALS: [(&str, GlassMaterial); 3] = [
    ("ui-glass-regular", GlassMaterial::Regular),
    ("ui-glass-clear", GlassMaterial::Clear),
    ("ui-glass-custom", GlassMaterial::Custom),
];
const APPEARANCES: [(&str, Option<GlassAppearance>); 3] = [
    ("ui-glass-theme", None),
    ("ui-glass-light", Some(GlassAppearance::Light)),
    ("ui-glass-dark", Some(GlassAppearance::Dark)),
];
const SHAPES: [(&str, LensShape); 5] = [
    ("ui-glass-auto", LensShape::Automatic),
    ("ui-glass-capsule", LensShape::Capsule),
    ("ui-glass-rectangle", LensShape::Rectangle),
    ("ui-glass-rounded-rectangle", LensShape::RoundedRectangle),
    ("ui-glass-ellipse", LensShape::Ellipse),
];
const INTERACTIONS: [(&str, Option<bool>); 3] = [
    ("ui-glass-auto", None),
    ("ui-glass-on", Some(true)),
    ("ui-glass-off", Some(false)),
];
const ROLES: [(&str, Option<Role>); 4] = [
    ("ui-glass-auto", None),
    ("ui-glass-role-button", Some(Role::Button)),
    ("ui-glass-role-group", Some(Role::Group)),
    ("ui-glass-role-image", Some(Role::Image)),
];

#[derive(Clone, Copy, PartialEq)]
enum LensShape {
    Automatic,
    Capsule,
    Rectangle,
    RoundedRectangle,
    Ellipse,
}

const LAYOUT_NUMBERS: [Number; 4] = [
    Number::Width,
    Number::Height,
    Number::Opacity,
    Number::Padding,
];
const TINT_NUMBERS: [Number; 7] = [
    Number::TintRed,
    Number::TintGreen,
    Number::TintBlue,
    Number::TintAlpha,
    Number::Noise,
    Number::NoiseScale,
    Number::Time,
];
const RIPPLE_NUMBERS: [Number; 5] = [
    Number::RippleX,
    Number::RippleY,
    Number::RippleRadius,
    Number::RippleAlpha,
    Number::RippleStrength,
];

#[derive(Clone, Copy, PartialEq)]
enum Number {
    Width,
    Height,
    Opacity,
    Corner,
    Padding,
    Border,
    Blur,
    Contrast,
    TintRed,
    TintGreen,
    TintBlue,
    TintAlpha,
    Noise,
    NoiseScale,
    Time,
    RippleX,
    RippleY,
    RippleRadius,
    RippleAlpha,
    RippleStrength,
}

impl Number {
    const COUNT: usize = Self::RippleStrength as usize + 1;

    fn spec(self) -> (&'static str, f32, f32, &'static str) {
        match self {
            Self::Width => ("ui-glass-width", 48.0, 224.0, " dp"),
            Self::Height => ("ui-glass-height", 36.0, 144.0, " dp"),
            Self::Opacity => ("ui-glass-opacity", 0.0, 1.0, ""),
            Self::Corner => ("ui-glass-corner-radius", 0.0, 72.0, " dp"),
            Self::Padding => ("ui-glass-padding", 0.0, 32.0, " dp"),
            Self::Border => ("ui-glass-border-width", 0.0, 12.0, " px"),
            Self::Blur => ("ui-blur-radius", 0.0, 32.0, " dp"),
            Self::Contrast => ("ui-glass-contrast", 0.0, 2.0, ""),
            Self::TintRed => ("ui-glass-tint-red", 0.0, 1.0, ""),
            Self::TintGreen => ("ui-glass-tint-green", 0.0, 1.0, ""),
            Self::TintBlue => ("ui-glass-tint-blue", 0.0, 1.0, ""),
            Self::TintAlpha => ("ui-glass-tint-alpha", 0.0, 1.0, ""),
            Self::Noise => ("ui-glass-noise-amount", 0.0, 0.3, ""),
            Self::NoiseScale => ("ui-glass-noise-scale", 0.1, 4.0, ""),
            Self::Time => ("ui-glass-noise-time", 0.0, 30.0, ""),
            Self::RippleX => ("ui-glass-ripple-x", 0.0, 1.0, ""),
            Self::RippleY => ("ui-glass-ripple-y", 0.0, 1.0, ""),
            Self::RippleRadius => ("ui-glass-ripple-radius", 0.0, 1.5, ""),
            Self::RippleAlpha => ("ui-glass-ripple-alpha", 0.0, 1.0, ""),
            Self::RippleStrength => ("ui-glass-ripple-strength", 0.0, 2.0, ""),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct LensSettings {
    numbers: [f32; Number::COUNT],
    material: usize,
    appearance: usize,
    shape: usize,
    interaction: usize,
    role: usize,
    custom_blur: bool,
    custom_border: bool,
    manual_ripple: bool,
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
            (Number::Border, 2.0),
            (Number::Blur, 4.5),
            (Number::Contrast, 1.0),
            (Number::TintRed, 0.03),
            (Number::TintGreen, 0.22),
            (Number::TintBlue, 0.9),
            (Number::NoiseScale, 1.0),
            (Number::RippleX, 0.5),
            (Number::RippleY, 0.5),
            (Number::RippleRadius, 0.65),
            (Number::RippleAlpha, 0.2),
            (Number::RippleStrength, 1.0),
        ] {
            numbers[parameter as usize] = value;
        }
        Self {
            numbers,
            material: 0,
            appearance: 0,
            shape: 0,
            interaction: 0,
            role: 0,
            custom_blur: false,
            custom_border: false,
            manual_ripple: false,
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

    fn shape(self) -> Option<Shape> {
        match SHAPES[self.shape].1 {
            LensShape::Automatic => None,
            LensShape::Capsule => Some(Shape::CAPSULE),
            LensShape::Rectangle => Some(Shape::RECTANGLE),
            LensShape::RoundedRectangle => Some(Shape::rounded_rectangle(self.dp(Number::Corner))),
            LensShape::Ellipse => Some(Shape::Ellipse),
        }
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
                        .item(move || {
                            toggle("ui-glass-custom-blur", settings.with(|s| s.custom_blur))
                                .on_toggle(move |checked| {
                                    settings.with_mut(|s| s.custom_blur = checked)
                                });
                        })
                        .item(move || {
                            number_control(settings, Number::Blur)
                                .enabled(settings.with(|s| s.custom_blur));
                        })
                        .item(move || {
                            number_control(settings, Number::Contrast);
                        })
                        .item(|| {
                            section_title("ui-glass-shape-section");
                        })
                        .item(move || {
                            choices(
                                "ui-glass-shape",
                                labels(&SHAPES),
                                settings.with(|s| s.shape),
                            )
                            .on_select(move |index| settings.with_mut(|s| s.shape = index));
                        })
                        .item(move || {
                            number_control(settings, Number::Corner).enabled(
                                settings.with(|s| SHAPES[s.shape].1 == LensShape::RoundedRectangle),
                            );
                        })
                        .items_with_key_shared(
                            LAYOUT_NUMBERS.len(),
                            |index| ("glass-number", LAYOUT_NUMBERS[index] as usize),
                            RenderSlotWith::new(move |index| {
                                number_control(settings, LAYOUT_NUMBERS[index]);
                            }),
                        )
                        .item(move || {
                            toggle("ui-glass-custom-border", settings.with(|s| s.custom_border))
                                .on_toggle(move |checked| {
                                    settings.with_mut(|s| s.custom_border = checked)
                                });
                        })
                        .item(move || {
                            number_control(settings, Number::Border)
                                .enabled(settings.with(|s| s.custom_border));
                        })
                        .item(|| {
                            section_title("ui-glass-tint-section");
                        })
                        .items_with_key_shared(
                            TINT_NUMBERS.len(),
                            |index| ("glass-number", TINT_NUMBERS[index] as usize),
                            RenderSlotWith::new(move |index| {
                                number_control(settings, TINT_NUMBERS[index]);
                            }),
                        )
                        .item(|| {
                            section_title("ui-glass-interaction-section");
                        })
                        .item(move || {
                            choices(
                                "ui-glass-interactive",
                                labels(&INTERACTIONS),
                                settings.with(|s| s.interaction),
                            )
                            .on_select(move |index| settings.with_mut(|s| s.interaction = index));
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
                        .item(move || {
                            toggle("ui-glass-manual-ripple", settings.with(|s| s.manual_ripple))
                                .on_toggle(move |checked| {
                                    settings.with_mut(|s| s.manual_ripple = checked)
                                });
                        })
                        .items_with_key_shared(
                            RIPPLE_NUMBERS.len(),
                            |index| ("glass-number", RIPPLE_NUMBERS[index] as usize),
                            RenderSlotWith::new(move |index| {
                                number_control(settings, RIPPLE_NUMBERS[index])
                                    .enabled(settings.with(|s| s.manual_ripple));
                            }),
                        )
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
                            scrollbar_v().state(scroll);
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
    let dark = match appearance {
        Some(appearance) => appearance == GlassAppearance::Dark,
        None => theme.with(|theme| theme.color_scheme.is_dark),
    };
    let ripple = |parameter| values.manual_ripple.then(|| values.value(parameter));
    let lens = fluid_glass()
        .modifier(
            Modifier::new()
                .size(width, height)
                .offset(x, y)
                .alpha(values.value(Number::Opacity))
                .draggable(move |delta: DragDelta| {
                    let width = stage_width.get();
                    let size = settings.get().lens_size(width);
                    let [x, y] = bounded_position(position.get(), size, width);
                    position.set(bounded_position(
                        [Dp(x.0 + delta.x.to_dp().0), Dp(y.0 + delta.y.to_dp().0)],
                        size,
                        width,
                    ));
                }),
        )
        .material(MATERIALS[values.material].1)
        .appearance_optional(appearance)
        .shape_optional(values.shape())
        .blur_radius_optional(values.custom_blur.then(|| values.dp(Number::Blur)))
        .border_optional(
            values
                .custom_border
                .then(|| GlassBorder::new(Px(values.value(Number::Border).round() as i32))),
        )
        .padding(values.dp(Number::Padding))
        .contrast(values.value(Number::Contrast))
        .tint_color(Color::new(
            values.value(Number::TintRed),
            values.value(Number::TintGreen),
            values.value(Number::TintBlue),
            values.value(Number::TintAlpha),
        ))
        .noise_amount(values.value(Number::Noise))
        .noise_scale(values.value(Number::NoiseScale))
        .time(values.value(Number::Time))
        .ripple_center_optional(
            values
                .manual_ripple
                .then(|| [values.value(Number::RippleX), values.value(Number::RippleY)]),
        )
        .ripple_radius_optional(ripple(Number::RippleRadius))
        .ripple_alpha_optional(ripple(Number::RippleAlpha))
        .ripple_strength_optional(ripple(Number::RippleStrength))
        .interactive_optional(INTERACTIONS[values.interaction].1)
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
    let formatted = if parameter == Number::Border {
        format!("{value:.0}{unit}")
    } else {
        format!("{value:.2}{unit}")
    };
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
            let slider = slider()
                .modifier(Modifier::new().fill_max_width())
                .value((value - min) / (max - min))
                .disabled(!enabled)
                .value_label(formatted.clone())
                .accessibility_label(label.clone())
                .on_change(move |fraction| {
                    let value = min + (max - min) * fraction;
                    settings.with_mut(|s| s.numbers[parameter as usize] = value);
                });
            if parameter == Number::Border {
                slider.steps((max - min) as usize - 1);
            }
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
            switch()
                .checked(checked)
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
