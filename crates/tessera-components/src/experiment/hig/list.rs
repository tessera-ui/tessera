//! Lists of rows with optional selection.
//!
//! ## Usage
//!
//! Show messages, files, settings, or a sidebar of destinations in rows.

use tessera_foundation::modifier::{ClickableArgs, ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    CallbackWith, Color, Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode,
    FocusDirection, FocusRequester, FocusState, FocusTraversalPolicy, Modifier, RenderSlot,
    SemanticsModifierNode,
    accesskit::Role,
    focus,
    layout::{Stack, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    provide_context, remember, tessera, use_context,
};

use crate::{
    alignment::CrossAxisAlignment, column::column,
    pipelines::perceptual_fill::PerceptualFillCommand, row::row, spacer::spacer,
};

use super::{
    scroll_view::scroll_view,
    text::text,
    theme::{Appearance, HigTheme, TextStyle, hig_theme},
};

#[cfg(test)]
#[path = "list_tests.rs"]
mod tests;

/// The appearance of a [`list`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListStyle {
    /// The platform default, an inset list on macOS.
    #[default]
    Automatic,
    /// Rows edge to edge, with square selection.
    Plain,
    /// Rows inset from the edges, with rounded selection.
    Inset,
    /// An inset list inside a border.
    Bordered,
    /// A source list for a window's sidebar, without its own background.
    Sidebar,
}

#[derive(Clone, Copy, PartialEq)]
struct ListContext {
    selection: Option<usize>,
    on_select: Option<CallbackWith<usize>>,
    style: ListStyle,
    alternating: bool,
    dark: bool,
}

#[derive(Clone, PartialEq)]
struct Fill {
    color: Color,
    radius: Dp,
    inset: Dp,
    outline: Dp,
}

impl DrawModifierNode for Fill {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        if self.color.a > 0.0 {
            context
                .render_input
                .metadata_mut()
                .fragment_mut()
                .push_draw_command(PerceptualFillCommand {
                    outline_width: self.outline,
                    snap_to_pixels: false,
                    path: None,
                    color: self.color,
                    radii: [self.radius; 2],
                    insets: [Dp::ZERO, self.inset, Dp::ZERO, self.inset],
                    clip_radius: None,
                    line: None,
                    edge_smoothness: 1.0,
                    stroke: None,
                });
        }
        content.draw(context.render_input);
    }
}

/// # list
///
/// Shows rows in a scrolling list, with optional single selection.
///
/// ## Usage
///
/// Present mail, files, settings, or sidebar destinations that people browse and select.
///
/// ## Parameters
///
/// - `content` — the list's [`list_row`]s and [`list_section`]s.
/// - `selection`, `on_select` — tag of the selected row and requested changes.
/// - `style` — edge-to-edge, inset, bordered, or sidebar; defaults to [`ListStyle::Automatic`].
/// - `alternating_row_backgrounds` — shades every other row; defaults to false.
/// - `modifier` — size and placement; the list fills the space its container offers.
/// - `accessibility_label` — spoken name of the list.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     list::{list, list_row},
///     text::text,
/// };
/// use tessera_ui::{remember, tessera};
/// # #[tessera]
/// # fn component() {
///
/// let selected = remember(|| 0usize);
/// list()
///     .selection(selected.get())
///     .on_select(move |tag| selected.set(tag))
///     .content(|| {
///         for (tag, name) in ["Inbox", "Drafts", "Sent"].into_iter().enumerate() {
///             list_row(tag).content(move || {
///                 text(name);
///             });
///         }
///     });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn list(
    content: Option<RenderSlot>,
    selection: Option<usize>,
    on_select: Option<CallbackWith<usize>>,
    style: Option<ListStyle>,
    alternating_row_backgrounds: Option<bool>,
    modifier: Option<Modifier>,
    #[prop(into)] accessibility_label: Option<String>,
) {
    let style = style.unwrap_or_default();
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let context = ListContext {
        selection,
        on_select,
        style,
        alternating: alternating_row_backgrounds.unwrap_or(false),
        dark,
    };
    let background = match style {
        ListStyle::Sidebar => Color::TRANSPARENT,
        _ if dark => Color::from_rgb(0.0129, 0.0129, 0.0129),
        _ => Color::WHITE,
    };
    let mut modifier = modifier
        .unwrap_or_default()
        .push_draw(Fill {
            color: background,
            radius: Dp::ZERO,
            inset: Dp::ZERO,
            outline: Dp::ZERO,
        })
        .focus_group(None)
        .focus_traversal_policy(FocusTraversalPolicy::linear().single_tab_stop(true))
        .semantics(SemanticsArgs {
            role: Some(if on_select.is_some() {
                Role::ListBox
            } else {
                Role::List
            }),
            label: accessibility_label,
            ..Default::default()
        });
    if style == ListStyle::Bordered {
        modifier = modifier.push_draw(Fill {
            color: if dark { Color::WHITE } else { Color::BLACK }.with_alpha(0.15),
            radius: Dp::ZERO,
            inset: Dp::ZERO,
            outline: Dp(1.0),
        });
    }
    let content = match content {
        Some(content) => content,
        None => RenderSlot::new(|| {}),
    };
    provide_context(context, move || {
        scroll_view()
            .modifier(modifier.clone())
            .child(move || list_body(style, content));
    });
}

#[tessera]
fn list_body(style: ListStyle, content: RenderSlot) {
    let vertical = if matches!(style, ListStyle::Plain) {
        Dp::ZERO
    } else {
        Dp(5.0)
    };
    column()
        .modifier(Modifier::new().padding_symmetric(Dp::ZERO, vertical))
        .children_shared(content);
}

struct RowSelection(bool);

impl SemanticsModifierNode for RowSelection {
    fn apply(
        &self,
        node: &mut tessera_ui::AccessibilityNode,
        _: &mut Option<tessera_ui::AccessibilityActionHandler>,
    ) {
        node.selected = Some(self.0);
    }
}

/// # list_row
///
/// Shows one row of a [`list`], selectable by its tag.
///
/// ## Usage
///
/// Show one message, file, setting, or destination in a list.
///
/// ## Parameters
///
/// - `tag` — identifies the row for the list's selection.
/// - `content` — the row's content, laid out in a row.
/// - `accessibility_label` — spoken name of the row.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     list::{list, list_row},
///     text::text,
/// };
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// list().content(|| {
///     list_row(0).content(|| {
///         text("Documents");
///     });
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn list_row(
    tag: usize,
    content: Option<RenderSlot>,
    #[prop(into)] accessibility_label: Option<String>,
) {
    let context = use_context::<ListContext>().map(|context| context.get());
    let style = context.map(|context| context.style).unwrap_or_default();
    let dark = context.is_some_and(|context| context.dark);
    let selected = context.is_some_and(|context| context.selection == Some(tag));
    let requester = remember(FocusRequester::new).get();
    let select = context.and_then(|context| context.on_select);
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    let rounded = !matches!(style, ListStyle::Plain);
    let fill = if selected {
        Color::from_rgb(0.0, 0.19461783, 1.0)
    } else if context.is_some_and(|context| context.alternating) && tag % 2 == 1 {
        tint.with_alpha(0.04)
    } else {
        Color::TRANSPARENT
    };
    let height = if style == ListStyle::Sidebar {
        28.0
    } else {
        24.0
    };
    let mut modifier = Modifier::new()
        .fill_max_width()
        .height(Dp(height))
        .push_draw(Fill {
            color: fill,
            radius: if rounded { Dp(5.0) } else { Dp::ZERO },
            inset: if rounded { Dp(10.0) } else { Dp::ZERO },
            outline: Dp::ZERO,
        });
    if let Some(select) = select {
        modifier = modifier
            .clickable(ClickableArgs {
                on_click: tessera_ui::Callback::new(move || select.call(tag)),
                role: Some(Role::ListBoxOption),
                label: accessibility_label.clone(),
                focus_requester: Some(requester),
                ..Default::default()
            })
            .preferred_tab_stop(selected)
            .on_focus_changed(move |state: FocusState| {
                if state.is_focused()
                    && matches!(
                        focus::focus_change_direction(),
                        Some(FocusDirection::Up | FocusDirection::Down)
                    )
                {
                    select.call(tag);
                }
            })
            .push_semantics(RowSelection(selected));
    } else {
        modifier = modifier.semantics(SemanticsArgs {
            role: Some(Role::ListItem),
            label: accessibility_label,
            ..Default::default()
        });
    }
    let content = match content {
        Some(content) => content,
        None => RenderSlot::new(|| {}),
    };
    let inset = if rounded { 20.0 } else { 10.0 };
    // Selected rows show their content in white on the accent fill.
    let appearance = if selected || dark {
        Appearance::Dark
    } else {
        Appearance::Light
    };
    layout(Stack).modifier(modifier).child(move || {
        hig_theme()
            .theme(HigTheme { appearance })
            .child(move || row_content(inset, content));
    });
}

#[tessera]
fn row_content(inset: f64, content: RenderSlot) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_size()
                .padding_symmetric(Dp(inset), Dp::ZERO),
        )
        .children_shared(content);
}

/// # list_section
///
/// Groups rows of a [`list`] under an optional header.
///
/// ## Usage
///
/// Divide a sidebar into Favorites and Locations, or a list into related groups.
///
/// ## Parameters
///
/// - `header` — optional header text above the rows.
/// - `content` — the section's rows.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     list::{ListStyle, list, list_row, list_section},
///     text::text,
/// };
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// list().style(ListStyle::Sidebar).content(|| {
///     list_section().header("Favorites").content(|| {
///         list_row(0).content(|| {
///             text("Desktop");
///         });
///     });
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn list_section(#[prop(into)] header: Option<String>, content: Option<RenderSlot>) {
    let context = use_context::<ListContext>().map(|context| context.get());
    let dark = context.is_some_and(|context| context.dark);
    let rounded = !matches!(
        context.map(|context| context.style).unwrap_or_default(),
        ListStyle::Plain
    );
    if let Some(header) = header {
        section_header(header, dark, if rounded { 20.0 } else { 10.0 });
    }
    if let Some(content) = content {
        content.render();
    }
}

#[tessera]
fn section_header(header: String, dark: bool, inset: f64) {
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().height(Dp(28.0)).semantics(SemanticsArgs {
            role: Some(Role::Heading),
            label: Some(header.clone()),
            ..Default::default()
        }))
        .children(move || {
            spacer().modifier(Modifier::new().width(Dp(inset - 10.0 + 8.0)));
            text(header.clone())
                .style(TextStyle {
                    font_size: Dp(11.0),
                    line_height: Dp(14.0),
                    font_weight: 600,
                    ..Default::default()
                })
                .color(tint.with_alpha(0.5))
                .modifier(Modifier::new().clear_and_set_semantics(SemanticsArgs {
                    hidden: true,
                    ..Default::default()
                }));
        });
}
