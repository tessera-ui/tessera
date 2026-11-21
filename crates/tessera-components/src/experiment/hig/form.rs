//! Forms of labeled controls, grouped into sections.
//!
//! ## Usage
//!
//! Lay out settings and data entry with aligned labels.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Color, Dp, FloatRect, Modifier, RenderSlot, State, accesskit::Role,
    modifier::FocusModifierExt as _, provide_context, remember, tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    modifier::ModifierExt as _,
    row::row,
    shape_def::Shape,
    spacer::spacer,
};

use super::{
    text::text,
    theme::{Appearance, HigTheme, TextStyle},
};

#[cfg(test)]
#[path = "form_tests.rs"]
mod tests;

/// The appearance of a [`form`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FormStyle {
    /// The platform default, columns on macOS.
    #[default]
    Automatic,
    /// Trailing-aligned labels in a leading column beside their controls.
    Columns,
    /// Sections in rounded boxes, with labels leading and controls trailing in each row.
    Grouped,
}

#[derive(Clone, Copy, PartialEq)]
struct FormContext {
    grouped: bool,
    /// Width of the widest label, in physical pixels; labels report their widths here.
    label_width: State<f32>,
    dark: bool,
}

/// # form
///
/// Lays out labeled controls with aligned labels, optionally grouped into sections.
///
/// ## Usage
///
/// Build settings panes and data entry forms from [`labeled_content`] rows.
///
/// ## Parameters
///
/// - `content` — the form's [`labeled_content`] rows and [`form_section`]s.
/// - `style` — columns or grouped sections; defaults to [`FormStyle::Automatic`].
/// - `modifier` — size and placement.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     form::{form, labeled_content},
///     text_field::text_field,
///     toggle::toggle,
/// };
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// form().content(|| {
///     labeled_content("Name").content(|| {
///         text_field().placeholder("Full name");
///     });
///     labeled_content("Updates").content(|| {
///         toggle(true).label("Install automatically");
///     });
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn form(content: Option<RenderSlot>, style: Option<FormStyle>, modifier: Option<Modifier>) {
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let label_width = remember(|| 0.0f32);
    let context = FormContext {
        grouped: style == Some(FormStyle::Grouped),
        label_width,
        dark,
    };
    let content = match content {
        Some(content) => content,
        None => RenderSlot::new(|| {}),
    };
    provide_context(context, move || {
        column()
            .modifier(
                modifier
                    .clone()
                    .unwrap_or_default()
                    .semantics(SemanticsArgs {
                        role: Some(Role::Form),
                        ..Default::default()
                    }),
            )
            .children_shared(content);
    });
}

/// # labeled_content
///
/// Shows a control or value beside its label.
///
/// ## Usage
///
/// Label each field, toggle, or read-only value in a [`form`].
///
/// ## Parameters
///
/// - `label` — the row's label.
/// - `content` — the control or value.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     form::{form, labeled_content},
///     text::text,
/// };
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// form().content(|| {
///     labeled_content("Version").content(|| {
///         text("27.1");
///     });
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn labeled_content(#[prop(into)] label: String, content: Option<RenderSlot>) {
    let context = use_context::<FormContext>().map(|context| context.get());
    let grouped = context.is_some_and(|context| context.grouped);
    let dark = context.is_some_and(|context| context.dark);
    let ink = if dark { Color::WHITE } else { Color::BLACK }.with_alpha(0.85);
    let content = match content {
        Some(content) => content,
        None => RenderSlot::new(|| {}),
    };
    let label_width = context.map(|context| context.label_width);
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_symmetric(if grouped { Dp(10.0) } else { Dp::ZERO }, Dp(4.0)),
        )
        .children(move || {
            form_label(label.clone(), ink, grouped).width_optional(label_width);
            if grouped {
                spacer().modifier(Modifier::new().weight(1.0));
            } else {
                spacer().modifier(Modifier::new().width(Dp(8.0)));
            }
            content.render();
        });
}

#[tessera]
fn form_label(label: String, ink: Color, grouped: bool, width: Option<State<f32>>) {
    let column_width = width.map(|width| width.get()).unwrap_or(0.0);
    let modifier = Modifier::new().on_globally_positioned_precise(move |rect: FloatRect| {
        if let Some(width) = width
            && rect.width > width.get()
        {
            width.set(rect.width);
        }
    });
    if grouped || column_width <= 0.0 {
        text(label).color(ink).single_line(true).modifier(modifier);
    } else {
        boxed()
            .alignment(Alignment::CenterEnd)
            .modifier(Modifier::new().width(Dp::from_pixels_f32(column_width)))
            .children(move || {
                text(label.clone())
                    .color(ink)
                    .single_line(true)
                    .modifier(modifier.clone());
            });
    }
}

/// # form_section
///
/// Groups related rows of a [`form`] under an optional header.
///
/// ## Usage
///
/// Separate a settings pane into groups, such as Notifications and Privacy.
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
///     form::{FormStyle, form, form_section, labeled_content},
///     toggle::toggle,
/// };
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// form().style(FormStyle::Grouped).content(|| {
///     form_section().header("Notifications").content(|| {
///         labeled_content("Sounds").content(|| {
///             toggle(true);
///         });
///     });
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn form_section(#[prop(into)] header: Option<String>, content: Option<RenderSlot>) {
    let context = use_context::<FormContext>().map(|context| context.get());
    let grouped = context.is_some_and(|context| context.grouped);
    let dark = context.is_some_and(|context| context.dark);
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    let content = match content {
        Some(content) => content,
        None => RenderSlot::new(|| {}),
    };
    column()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_symmetric(Dp::ZERO, Dp(6.0)),
        )
        .children(move || {
            if let Some(header) = header.clone() {
                text(header)
                    .style(TextStyle {
                        font_weight: 600,
                        ..Default::default()
                    })
                    .color(tint.with_alpha(0.85))
                    .modifier(
                        Modifier::new()
                            .padding_symmetric(Dp(if grouped { 10.0 } else { 0.0 }), Dp(4.0)),
                    );
            }
            if grouped {
                let shape = Shape::rounded_rectangle(Dp(10.0));
                column()
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .background_with_shape(tint.with_alpha(0.04), shape)
                            .border_with_shape(Dp(1.0), tint.with_alpha(0.06), shape)
                            .padding_symmetric(Dp::ZERO, Dp(4.0)),
                    )
                    .children_shared(content);
            } else {
                content.render();
            }
        });
}
