//! Hyperlinks for navigation to related content.
//!
//! ## Usage
//!
//! Link help text, related settings, and external resources from the current view.

use tessera_foundation::modifier::{ClickableArgs, ModifierExt as _};
use tessera_ui::{
    Callback, CallbackWith, Color, Dp, FocusRequester, Modifier, accesskit::Role,
    modifier::CursorModifierExt as _, remember, tessera, use_context, winit::window::CursorIcon,
};

use crate::text_shared::text_node;

use super::{
    text::metrics,
    theme::{Appearance, HigTheme, TextStyle},
};

#[cfg(test)]
#[path = "link_tests.rs"]
mod tests;

/// Opens the destinations of [`link`]s in its subtree.
///
/// Provide it with [`tessera_ui::provide_context`] to route links to a browser, a router, or a
/// help viewer.
#[derive(Clone, Copy, PartialEq)]
pub struct OpenUrlAction(CallbackWith<String>);

impl OpenUrlAction {
    /// Creates an action that receives each destination URL.
    pub fn new(open: impl Fn(String) + 'static) -> Self {
        Self(CallbackWith::new(open))
    }
}

pub(super) fn link_color(appearance: Appearance) -> Color {
    match appearance {
        Appearance::Light => Color::from_rgb(
            srgb_channel_to_linear(0),
            srgb_channel_to_linear(104),
            srgb_channel_to_linear(218),
        ),
        Appearance::Dark => Color::from_rgb(
            srgb_channel_to_linear(65),
            srgb_channel_to_linear(156),
            srgb_channel_to_linear(255),
        ),
    }
}

fn srgb_channel_to_linear(channel: u8) -> f32 {
    let value = f32::from(channel) / 255.0;
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

/// # link
///
/// Opens a destination URL from underlined, keyboard-accessible text.
///
/// ## Usage
///
/// Navigate to help, related settings, or a web page outside the current view.
///
/// ## Parameters
///
/// - `label` — visible and default spoken link text.
/// - `destination` — URL handed to the [`OpenUrlAction`] in context.
/// - `on_open` — opens the destination instead of the action in context.
/// - `enabled` — whether pointer, keyboard, and accessibility activation are available.
/// - `modifier` — layout and additional node behavior.
/// - `color` — optional foreground override; otherwise uses the system link color for the HIG appearance.
/// - `accessibility_label`, `accessibility_description` — optional spoken name and context.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::link::{OpenUrlAction, link};
/// use tessera_ui::provide_context;
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// provide_context(OpenUrlAction::new(|url| println!("open {url}")), || {
///     link("Privacy policy", "https://example.com/privacy");
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn link(
    #[prop(into)] label: String,
    #[prop(into)] destination: String,
    on_open: Option<CallbackWith<String>>,
    enabled: Option<bool>,
    color: Option<Color>,
    modifier: Option<Modifier>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let enabled = enabled.unwrap_or(true);
    let appearance = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance)
        .unwrap_or_default();
    let foreground = color.unwrap_or_else(|| {
        if enabled {
            link_color(appearance)
        } else {
            match appearance {
                Appearance::Light => Color::BLACK.with_alpha(63.0 / 255.0),
                Appearance::Dark => Color::WHITE.with_alpha(63.0 / 255.0),
            }
        }
    });
    let requester = remember(FocusRequester::new).get();
    let context = use_context::<OpenUrlAction>().map(|action| action.get().0);
    let open = on_open.or(context);
    let on_click = Callback::new(move || {
        if let Some(open) = open {
            open.call(destination.clone());
        }
    });
    let mut modifier = modifier.unwrap_or_default().clickable(ClickableArgs {
        on_click,
        enabled,
        role: Some(Role::Link),
        label: Some(accessibility_label.unwrap_or_else(|| label.clone())),
        description: accessibility_description,
        focus_requester: Some(requester),
        ..Default::default()
    });
    if enabled {
        modifier = modifier.hover_cursor_icon(CursorIcon::Pointer);
    }
    let style = use_context::<TextStyle>()
        .map(|style| style.get())
        .unwrap_or_default();
    let font_scale = style.font_size.0 / 13.0;
    let underline = (Dp(0.968 * font_scale), Dp(0.762 * font_scale));
    text_node(
        label.into(),
        modifier,
        foreground,
        metrics(style, None, None),
        true,
        true,
    )
    .underline(underline);
}
