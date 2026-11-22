//! Search fields with tokens, suggestions, and scopes.
//!
//! ## Usage
//!
//! Search a mailbox or library, suggest completions, and narrow results by scope.

use tessera_foundation::modifier::{ClickableArgs, ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Callback, CallbackWith, Color, Dp, FloatRect, Modifier, RenderSlot, accesskit::Role,
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
    menu::{MenuContext, menu_item},
    picker::{PickerStyle, picker},
    presentation::{PopupPlacement, SurfaceStyle, present_surface},
    text::text,
    text_field::{TextFieldController, search_field},
    theme::{Appearance, HigTheme, TextStyle},
};

#[cfg(test)]
#[path = "searchable_tests.rs"]
mod tests;

/// # searchable
///
/// Shows a search field with optional tokens, completion suggestions, and search scopes.
///
/// ## Usage
///
/// Search a list or library, suggest completions while typing, and narrow results by scope.
///
/// ## Parameters
///
/// - `text`, `on_change` — the query and requested changes.
/// - `prompt` — hint shown while the field is empty; defaults to "Search".
/// - `tokens`, `on_tokens_change` — search tokens shown before the query; clicking a token
///   requests its removal.
/// - `suggestions`, `on_suggestion` — completions shown below the field while typing; choosing
///   one requests it as the query unless `on_suggestion` handles it.
/// - `scopes`, `scope`, `on_scope_change` — scope titles shown below the field while searching,
///   the selected scope, and requested changes.
/// - `on_submit` — called when the user submits with Enter.
/// - `modifier` — size and placement.
///
/// Suggestions appear in the nearest
/// [`presentation_host`](super::presentation::presentation_host).
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::searchable::searchable;
/// use tessera_ui::{remember, tessera};
/// # #[tessera]
/// # fn component() {
///
/// let query = remember(String::new);
/// searchable(query.get())
///     .suggestions(vec!["Invoices".into(), "Receipts".into()])
///     .scopes(vec!["All".into(), "Unread".into()])
///     .on_change(move |text| query.set(text));
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn searchable(
    #[prop(into)] text: String,
    on_change: Option<CallbackWith<String>>,
    #[prop(into)] prompt: Option<String>,
    tokens: Option<Vec<String>>,
    on_tokens_change: Option<CallbackWith<Vec<String>>>,
    suggestions: Option<Vec<String>>,
    on_suggestion: Option<CallbackWith<usize>>,
    scopes: Option<Vec<String>>,
    scope: Option<usize>,
    on_scope_change: Option<CallbackWith<usize>>,
    on_submit: Option<Callback>,
    modifier: Option<Modifier>,
) {
    let controller = remember(|| TextFieldController::new(Dp(13.0), Some(Dp(16.0))));
    if controller.with(|editor| editor.text() != text) {
        controller.with_mut(|editor| editor.set_text(&text));
    }
    let focused = controller.with(|editor| editor.focus_handler().is_focused());
    let anchor = remember(|| FloatRect::ZERO);
    // Dismissing hides the suggestions until the query changes.
    let dismissed = remember(|| None::<String>);
    let tokens = tokens.unwrap_or_default();
    let suggestions = suggestions.unwrap_or_default();
    let scopes = scopes.unwrap_or_default();
    let showing_suggestions = focused
        && !text.is_empty()
        && !suggestions.is_empty()
        && dismissed.get().as_deref() != Some(text.as_str());
    let change = CallbackWith::new(move |value: String| {
        if let Some(on_change) = on_change {
            on_change.call(value);
        }
    });
    let dismiss = {
        let text = text.clone();
        Callback::new(move || dismissed.set(Some(text.clone())))
    };
    let choose = {
        let suggestions = suggestions.clone();
        CallbackWith::new(move |index: usize| match on_suggestion {
            Some(on_suggestion) => on_suggestion.call(index),
            None => {
                if let Some(suggestion) = suggestions.get(index) {
                    change.call(suggestion.clone());
                }
            }
        })
    };
    let width = Dp::from_pixels_f32(anchor.get().width);
    let shown = suggestions.clone();
    present_surface(
        showing_suggestions,
        anchor.get(),
        SurfaceStyle {
            placement: PopupPlacement::Below,
            modal: false,
            radius: Dp(10.0),
            padding: Dp(5.0),
            focus: false,
        },
        RenderSlot::new(move || suggestion_list(shown.clone(), choose, dismiss, width)),
        dismiss,
    );
    let searching = focused || !text.is_empty();
    column()
        .modifier(modifier.unwrap_or_default().semantics(SemanticsArgs {
            role: Some(Role::Search),
            ..Default::default()
        }))
        .children(move || {
            let tokens = tokens.clone();
            let prompt = prompt.clone();
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    for (index, token) in tokens.iter().cloned().enumerate() {
                        let remaining: Vec<String> = tokens
                            .iter()
                            .enumerate()
                            .filter(|(other, _)| *other != index)
                            .map(|(_, token)| token.clone())
                            .collect();
                        let chip = search_token(token);
                        if let Some(on_tokens_change) = on_tokens_change {
                            chip.on_remove(move || on_tokens_change.call(remaining.clone()));
                        }
                        spacer().modifier(Modifier::new().width(Dp(4.0)));
                    }
                    search_field()
                        .controller(controller)
                        .placeholder_optional(prompt.clone())
                        .on_submit_optional(on_submit)
                        .on_change(move |value: String| {
                            change.call(value.clone());
                            value
                        })
                        .modifier(Modifier::new().weight(1.0).on_globally_positioned_precise(
                            move |rect| {
                                if anchor.get() != rect {
                                    anchor.set(rect);
                                }
                            },
                        ));
                });
            if searching && !scopes.is_empty() {
                spacer().modifier(Modifier::new().height(Dp(8.0)));
                let bar = picker(scope.unwrap_or(0), scopes.clone())
                    .style(PickerStyle::Segmented)
                    .accessibility_label("Search scope");
                if let Some(on_scope_change) = on_scope_change {
                    bar.on_select_shared(on_scope_change);
                }
            }
        });
}

#[tessera]
fn suggestion_list(
    suggestions: Vec<String>,
    choose: CallbackWith<usize>,
    dismiss: Callback,
    width: Dp,
) {
    provide_context(MenuContext { dismiss }, move || {
        let suggestions = suggestions.clone();
        column()
            .modifier(Modifier::new().width(Dp((width.0 - 10.0).max(0.0))))
            .children(move || {
                for (index, suggestion) in suggestions.iter().cloned().enumerate() {
                    menu_item(suggestion).on_click(move || choose.call(index));
                }
            });
    });
}

/// A token in a search field; clicking it requests its removal.
#[tessera]
fn search_token(title: String, on_remove: Option<Callback>) {
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let accent = Color::from_rgb(0.0, 0.19461783, 1.0);
    let shape = Shape::rounded_rectangle(Dp(4.0));
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .height(Dp(18.0))
                .background_with_shape(accent.with_alpha(if dark { 0.35 } else { 0.18 }), shape)
                .padding_symmetric(Dp(5.0), Dp::ZERO)
                .clickable(ClickableArgs {
                    on_click: on_remove.unwrap_or_else(Callback::noop),
                    enabled: on_remove.is_some(),
                    role: Some(Role::Button),
                    label: Some(title.clone()),
                    description: Some("Remove token".into()),
                    ..Default::default()
                }),
        )
        .children(move || {
            text(title.clone())
                .style(TextStyle {
                    font_size: Dp(11.0),
                    line_height: Dp(14.0),
                    ..Default::default()
                })
                .color(if dark { Color::WHITE } else { Color::BLACK }.with_alpha(0.85))
                .single_line(true)
                .modifier(Modifier::new().clear_and_set_semantics(SemanticsArgs {
                    hidden: true,
                    ..Default::default()
                }));
        });
}
