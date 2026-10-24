//! Paragraphs containing native interactive text links.
//!
//! ## Usage
//!
//! Explain a setting with inline navigation to related information.

use std::sync::Arc;

use parking_lot::Mutex;
use tessera_foundation::gesture::TapRecognizer;
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, Callback, Color, ComputedData, Constraint, Dp,
    FocusRequester, LayoutResult, MeasurementError, Modifier, PointerInput,
    PointerInputModifierNode, PxPosition, PxSize, SemanticsModifierNode, State,
    accesskit::{Action, Role},
    layout::{MeasureScope, layout},
    modifier::{CursorModifierExt as _, FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
    winit::{
        event::ElementState,
        keyboard::{Key, NamedKey},
        window::CursorIcon,
    },
};

use crate::{
    modifier::{ModifierExt as _, SemanticsArgs, with_keyboard_input},
    pipelines::{
        shape::command::ShapeCommand,
        text::{
            command::{TextCommand, TextConstraint},
            pipeline::{TextData, color_to_glyphon, write_font_system},
        },
    },
    theme::{ContentColor, MaterialTheme, TextStyle},
};

/// A text run in a paragraph, optionally linking to another task or location.
#[derive(Clone, Default, PartialEq)]
pub struct LinkedTextRun {
    /// Visible text, including any surrounding spaces or punctuation.
    pub text: String,
    /// Navigation invoked by pointer, keyboard, or accessibility activation.
    pub on_activate: Option<Callback>,
}

impl LinkedTextRun {
    /// Creates a non-interactive text run.
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            on_activate: None,
        }
    }

    /// Creates an inline link whose accessible name is its visible text.
    pub fn link(text: impl Into<String>, on_activate: Callback) -> Self {
        Self {
            text: text.into(),
            on_activate: Some(on_activate),
        }
    }
}

/// # linked_text
///
/// Present a paragraph with inline links for contextual help and navigation.
///
/// ## Usage
///
/// Connect an explanation to related settings without interrupting its text
/// flow.
///
/// ## Parameters
///
/// - `runs` — plain and linked text in reading order; spaces belong to the
///   runs.
/// - `modifier` — paragraph layout and semantics.
/// - `style`, `color` — optional paragraph typography and text color.
/// - `link_color`, `focus_color` — optional link and focus outline colors.
///
/// ## Examples
///
/// ```
/// use tessera_components::linked_text::{LinkedTextRun, linked_text};
/// use tessera_ui::{Callback, remember, tessera};
/// #[tessera]
/// fn explanation() {
///     let opened = remember(|| false);
///     let navigate = Callback::new(move || opened.set(true));
///     linked_text(vec![
///         LinkedTextRun::plain("Visit "),
///         LinkedTextRun::link("permission settings", navigate),
///         LinkedTextRun::plain(" for details."),
///     ]);
///     navigate.call();
///     assert!(opened.get());
/// }
/// explanation();
/// ```
#[tessera]
pub fn linked_text(
    runs: Vec<LinkedTextRun>,
    modifier: Option<Modifier>,
    style: Option<TextStyle>,
    color: Option<Color>,
    link_color: Option<Color>,
    focus_color: Option<Color>,
) {
    let theme = use_context::<MaterialTheme>()
        .map(|theme| theme.get())
        .unwrap_or_default();
    let style = style
        .or_else(|| use_context::<TextStyle>().map(|style| style.get()))
        .unwrap_or(theme.typography.body_large);
    let color = color
        .or_else(|| use_context::<ContentColor>().map(|color| color.get().current))
        .unwrap_or(theme.color_scheme.on_surface);
    let link_color = link_color.unwrap_or(theme.color_scheme.primary);
    let focus_color = focus_color.unwrap_or(theme.color_scheme.secondary);
    let cache = remember(|| Arc::new(Mutex::new(None))).get();
    let policy = ParagraphLayout {
        runs: runs
            .iter()
            .map(|run| (run.text.clone(), run.on_activate.is_some()))
            .collect(),
        style,
        color,
        link_color,
        cache: Arc::clone(&cache),
    };
    layout()
        .modifier(modifier.unwrap_or_default().semantics(SemanticsArgs {
            role: Some(Role::Paragraph),
            merge_descendants: true,
            ..Default::default()
        }))
        .policy(policy.clone())
        .draw(policy)
        .child(move || {
            for (index, run) in runs.iter().cloned().enumerate() {
                linked_run(index, run, Arc::clone(&cache), link_color, focus_color);
            }
        });
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Fragment {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    baseline: f32,
}

impl Fragment {
    fn contains(self, point: PxPosition) -> bool {
        point.x >= self.x
            && point.x < self.x + self.w
            && point.y >= self.y
            && point.y < self.y + self.h
    }
}

struct Paragraph {
    data: TextData,
    fragments: Vec<Vec<Fragment>>,
    bounds: Vec<Option<Fragment>>,
}

#[derive(Clone, PartialEq)]
struct ShapeKey {
    runs: Vec<(String, bool)>,
    style: TextStyle,
    link_color: Color,
    constraint: TextConstraint,
    scale: f32,
}

struct CachedParagraph {
    key: ShapeKey,
    paragraph: Arc<Paragraph>,
}

type ParagraphCache = Arc<Mutex<Option<CachedParagraph>>>;

fn shape_paragraph(key: &ShapeKey) -> Paragraph {
    let size = key.style.font_size.to_pixels_f32();
    let height = key
        .style
        .line_height
        .unwrap_or(Dp(key.style.font_size.0 * 1.2))
        .to_pixels_f32();
    let attrs = glyphon::Attrs::new()
        .family(
            key.style
                .font_family
                .map(glyphon::Family::Name)
                .unwrap_or(glyphon::Family::SansSerif),
        )
        .weight(glyphon::Weight(key.style.font_weight.clamp(1, 1000)))
        .letter_spacing(if size > 0.0 {
            key.style.letter_spacing.to_pixels_f32() / size
        } else {
            0.0
        });
    let mut font_system = write_font_system();
    let mut buffer = glyphon::Buffer::new(&mut font_system, glyphon::Metrics::new(size, height));
    buffer.set_wrap(glyphon::Wrap::WordOrGlyph);
    buffer.set_size(key.constraint.max_width, key.constraint.max_height);
    buffer.set_rich_text(
        key.runs.iter().enumerate().map(|(index, (text, linked))| {
            let attrs = attrs.clone().metadata(index + 1);
            (
                text.as_str(),
                if *linked {
                    attrs.color(color_to_glyphon(key.link_color))
                } else {
                    attrs
                },
            )
        }),
        &attrs,
        glyphon::Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut font_system, false);
    drop(font_system);
    let mut fragments: Vec<Vec<Fragment>> = vec![Vec::new(); key.runs.len()];
    for run in buffer.layout_runs() {
        for glyph in run.glyphs {
            if glyph.metadata == 0 || glyph.w <= 0.0 {
                continue;
            }
            fragments[glyph.metadata - 1].push(Fragment {
                x: glyph.x,
                y: run.line_top,
                w: glyph.w,
                h: run.line_height,
                baseline: run.line_y,
            });
        }
    }
    for run in &mut fragments {
        run.sort_by(|a, b| a.y.total_cmp(&b.y).then_with(|| a.x.total_cmp(&b.x)));
        let mut merged: Vec<Fragment> = Vec::new();
        for fragment in run.drain(..) {
            if let Some(previous) = merged.last_mut()
                && previous.y == fragment.y
                && fragment.x <= previous.x + previous.w + 0.01
            {
                previous.w = (fragment.x + fragment.w).max(previous.x + previous.w) - previous.x;
            } else {
                merged.push(fragment);
            }
        }
        *run = merged;
    }
    let bounds = fragments
        .iter()
        .map(|run| {
            run.iter()
                .copied()
                .reduce(|a, b| {
                    let x = a.x.min(b.x).floor();
                    let y = a.y.min(b.y).floor();
                    Fragment {
                        x,
                        y,
                        w: (a.x + a.w).max(b.x + b.w).ceil() - x,
                        h: (a.y + a.h).max(b.y + b.h).ceil() - y,
                        baseline: a.baseline,
                    }
                })
                .map(|rect| {
                    let x = rect.x.floor();
                    let y = rect.y.floor();
                    Fragment {
                        x,
                        y,
                        w: (rect.x + rect.w).ceil() - x,
                        h: (rect.y + rect.h).ceil() - y,
                        ..rect
                    }
                })
        })
        .collect();
    Paragraph {
        data: TextData::from_buffer(buffer),
        fragments,
        bounds,
    }
}

#[derive(Clone)]
struct ParagraphLayout {
    runs: Vec<(String, bool)>,
    style: TextStyle,
    color: Color,
    link_color: Color,
    cache: ParagraphCache,
}

impl PartialEq for ParagraphLayout {
    fn eq(&self, other: &Self) -> bool {
        self.runs == other.runs
            && self.style == other.style
            && self.color == other.color
            && self.link_color == other.link_color
    }
}

impl ParagraphLayout {
    fn paragraph(&self, constraint: TextConstraint) -> Arc<Paragraph> {
        let key = ShapeKey {
            runs: self.runs.clone(),
            style: self.style,
            link_color: self.link_color,
            constraint,
            scale: Dp(1.0).to_pixels_f32(),
        };
        let mut cache = self.cache.lock();
        if cache.as_ref().is_none_or(|cache| cache.key != key) {
            let paragraph = Arc::new(shape_paragraph(&key));
            *cache = Some(CachedParagraph { key, paragraph });
        }
        Arc::clone(&cache.as_ref().unwrap().paragraph)
    }
}

impl tessera_ui::LayoutModifierNode for ParagraphLayout {
    fn measure_eq(&self, other: &Self) -> bool {
        self.runs == other.runs && self.style == other.style
    }
    fn placement_eq(&self, other: &Self) -> bool {
        self.measure_eq(other)
    }
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let paragraph = self.paragraph(TextConstraint {
            max_width: input
                .parent_constraint()
                .width()
                .resolve_max()
                .map(|size| size.to_f32()),
            max_height: input
                .parent_constraint()
                .height()
                .resolve_max()
                .map(|size| size.to_f32()),
        });
        let mut result = LayoutResult::default();
        for (index, child) in input.children().into_iter().enumerate() {
            if let Some(bounds) = paragraph.bounds[index] {
                child.measure(
                    input,
                    &Constraint::new(
                        tessera_ui::AxisConstraint::exact(tessera_ui::Px(bounds.w.ceil() as i32)),
                        tessera_ui::AxisConstraint::exact(tessera_ui::Px(bounds.h.ceil() as i32)),
                    ),
                )?;
                result.place_child(child, PxPosition::from_f32_xy(bounds.x, bounds.y));
            }
        }
        Ok(result.with_size(ComputedData {
            width: input
                .parent_constraint()
                .width()
                .clamp(tessera_ui::Px(paragraph.data.size[0] as i32)),
            height: input
                .parent_constraint()
                .height()
                .clamp(tessera_ui::Px(paragraph.data.size[1] as i32)),
        }))
    }
}

impl tessera_ui::DrawModifierNode for ParagraphLayout {
    fn draw(
        &self,
        ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        let constraint = self
            .cache
            .lock()
            .as_ref()
            .map(|cache| cache.key.constraint.clone());
        if let Some(constraint) = constraint {
            let paragraph = self.paragraph(constraint);
            let mut data = paragraph.data.clone();
            data.set_color(self.color);
            ctx.render_input
                .metadata_mut()
                .fragment_mut()
                .push_draw_command(TextCommand {
                    data,
                    offset: PxPosition::ZERO,
                });
        }
        content.draw(ctx.render_input);
    }
}

#[tessera]
fn linked_run(
    index: usize,
    run: LinkedTextRun,
    #[prop(skip_eq)] cache: ParagraphCache,
    link_color: Color,
    focus_color: Color,
) {
    let focus = remember(FocusRequester::new).get();
    let tap = remember(TapRecognizer::default);
    let hovered = remember(|| false);
    let policy = RunLayout {
        index,
        cache: Arc::clone(&cache),
        focus,
        link_color,
        focus_color,
        linked: run.on_activate.is_some(),
    };
    let modifier = if let Some(on_activate) = run.on_activate {
        let modifier = Modifier::new()
            .focus_requester(focus)
            .focusable()
            .push_semantics(LinkSemantics {
                label: run.text,
                on_activate,
                focus,
            })
            .push_pointer_input(LinkPointer {
                index,
                cache,
                tap,
                hovered,
                focus,
                on_activate,
            });
        let modifier = if hovered.get() {
            modifier.hover_cursor_icon(CursorIcon::Pointer)
        } else {
            modifier
        };
        with_keyboard_input(modifier, move |mut input| {
            if input.key_modifiers.control_key()
                || input.key_modifiers.alt_key()
                || input.key_modifiers.super_key()
            {
                return;
            }
            let mut activated = false;
            input.keyboard_events.retain(|event| {
                if matches!(
                    event.logical_key,
                    Key::Named(NamedKey::Enter | NamedKey::Space)
                ) {
                    if event.state == ElementState::Pressed && !event.repeat {
                        on_activate.call();
                        activated = true;
                    }
                    false
                } else {
                    true
                }
            });
            if activated {
                input.block_keyboard();
            }
        })
    } else {
        Modifier::new().semantics(SemanticsArgs {
            role: Some(Role::Label),
            label: Some(run.text),
            ..Default::default()
        })
    };
    layout()
        .modifier(modifier)
        .policy(policy.clone())
        .draw(policy);
}

struct LinkSemantics {
    label: String,
    on_activate: Callback,
    focus: FocusRequester,
}

impl SemanticsModifierNode for LinkSemantics {
    fn apply(
        &self,
        node: &mut AccessibilityNode,
        handler: &mut Option<AccessibilityActionHandler>,
    ) {
        node.role = Some(Role::Link);
        node.label = Some(self.label.clone());
        node.focusable = true;
        node.actions.push(Action::Click);
        let on_activate = self.on_activate;
        let focus = self.focus;
        *handler = Some(Box::new(move |action| {
            if action == Action::Click {
                focus.request_focus();
                on_activate.call();
            }
        }));
    }
}

struct LinkPointer {
    index: usize,
    cache: ParagraphCache,
    tap: State<TapRecognizer>,
    hovered: State<bool>,
    focus: FocusRequester,
    on_activate: Callback,
}

impl PointerInputModifierNode for LinkPointer {
    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
        let within = input.cursor_position_rel.is_some_and(|position| {
            self.cache.lock().as_ref().is_some_and(|cache| {
                cache.paragraph.bounds[self.index].is_some_and(|bounds| {
                    cache.paragraph.fragments[self.index]
                        .iter()
                        .any(|fragment| {
                            fragment.contains(PxPosition::from_f32_xy(
                                position.x + bounds.x,
                                position.y + bounds.y,
                            ))
                        })
                })
            })
        });
        if self.hovered.get() != within {
            self.hovered.set(within);
        }
        let tap = self.tap.with_mut(|tap| {
            tap.update_exclusive(input.pass, input.pointer_changes.as_mut_slice(), within)
        });
        if tap.pressed {
            self.focus.request_focus();
        }
        if tap.tapped {
            self.on_activate.call();
        }
        if within {
            input.consume_pointer_changes();
        }
    }
}

#[derive(Clone)]
struct RunLayout {
    index: usize,
    cache: ParagraphCache,
    focus: FocusRequester,
    link_color: Color,
    focus_color: Color,
    linked: bool,
}

impl PartialEq for RunLayout {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
            && self.focus == other.focus
            && self.link_color == other.link_color
            && self.focus_color == other.focus_color
            && self.linked == other.linked
    }
}

impl tessera_ui::LayoutModifierNode for RunLayout {
    fn measure_eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
    fn placement_eq(&self, other: &Self) -> bool {
        self.measure_eq(other)
    }
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        Ok(LayoutResult::new(ComputedData {
            width: input
                .parent_constraint()
                .width()
                .clamp(tessera_ui::Px::ZERO),
            height: input
                .parent_constraint()
                .height()
                .clamp(tessera_ui::Px::ZERO),
        }))
    }
}

impl tessera_ui::DrawModifierNode for RunLayout {
    fn draw(
        &self,
        ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        content.draw(ctx.render_input);
        if !self.linked {
            return;
        }
        let focused = self.focus.has_focus();
        let cache = self.cache.lock();
        let Some(cache) = cache.as_ref() else {
            return;
        };
        let Some(bounds) = cache.paragraph.bounds[self.index] else {
            return;
        };
        let mut metadata = ctx.render_input.metadata_mut();
        let fragment = metadata.fragment_mut();
        for rect in &cache.paragraph.fragments[self.index] {
            let index = fragment.push_draw_command(ShapeCommand::Rect {
                color: self.link_color,
                corner_radii: [0.0; 4],
                corner_g2: [2.0; 4],
            });
            let op = &mut fragment.ops_mut()[index as usize];
            op.position_override = Some(PxPosition::from_f32_xy(
                rect.x - bounds.x,
                rect.baseline + Dp(1.0).to_pixels_f32() - bounds.y,
            ));
            op.size_override = Some(PxSize::new(
                tessera_ui::Px(rect.w.ceil() as i32),
                Dp(1.0).to_px(),
            ));
            if focused {
                let padding = Dp(2.0).to_pixels_f32();
                let index = fragment.push_draw_command(ShapeCommand::OutlinedRect {
                    color: self.focus_color,
                    corner_radii: [Dp(4.0).to_pixels_f32(); 4],
                    corner_g2: [2.0; 4],
                    border_width: Dp(2.0).to_pixels_f32(),
                });
                let op = &mut fragment.ops_mut()[index as usize];
                op.position_override = Some(PxPosition::from_f32_xy(
                    rect.x - bounds.x - padding,
                    rect.y - bounds.y - padding,
                ));
                op.size_override = Some(PxSize::new(
                    tessera_ui::Px((rect.w + padding * 2.0).ceil() as i32),
                    tessera_ui::Px((rect.h + padding * 2.0).ceil() as i32),
                ));
            }
        }
    }
}
