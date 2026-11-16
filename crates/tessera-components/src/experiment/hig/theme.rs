//! Appearance and typography shared by HIG components.

use tessera_ui::{Color, Dp, RenderSlot, provide_context, tessera};

/// The light or dark appearance of a HIG interface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Appearance {
    /// Dark content on light surfaces.
    #[default]
    Light,
    /// Light content on dark surfaces.
    Dark,
}

/// Font metrics used by HIG text and control labels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyle {
    /// Font size in density-independent pixels.
    pub font_size: Dp,
    /// Distance between adjacent baselines.
    pub line_height: Dp,
    /// CSS-compatible font weight, from 1 to 1000.
    pub font_weight: u16,
    /// Extra spacing between letters.
    pub letter_spacing: Dp,
    /// Named font family; `None` selects the default sans-serif font.
    pub font_family: Option<&'static str>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: Dp(13.0),
            line_height: Dp(16.0),
            font_weight: 400,
            letter_spacing: Dp(0.0),
            font_family: Some(".SF NS"),
        }
    }
}

/// Colors and appearance inherited by HIG components.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HigTheme {
    /// Light or dark surface treatment.
    pub appearance: Appearance,
}

impl HigTheme {
    /// The primary text color in this appearance.
    pub fn label_color(self) -> Color {
        match self.appearance {
            Appearance::Light => Color::BLACK.with_alpha(0.85),
            Appearance::Dark => Color::WHITE.with_alpha(0.85),
        }
    }
}

/// Text color inherited from a containing HIG control.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContentColor(pub Color);

/// # hig_theme
///
/// Provides a consistent appearance for HIG controls and content.
///
/// ## Usage
///
/// Apply a light or dark appearance to an application or a section of it.
///
/// ## Parameters
///
/// - `theme` — appearance to inherit; defaults to light.
/// - `child` — themed content.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::theme::{Appearance, HigTheme, hig_theme};
/// use tessera_ui::{tessera, use_context};
///
/// #[tessera]
/// fn app() {
///     hig_theme().theme(HigTheme { appearance: Appearance::Dark }).child(|| {
///         assert_eq!(use_context::<HigTheme>().unwrap().get().appearance, Appearance::Dark);
///     });
/// }
/// tessera_ui::testing::UiTest::new().content(app).frame();
/// ```
#[tessera]
pub fn hig_theme(theme: Option<HigTheme>, child: Option<RenderSlot>) {
    provide_context(theme.unwrap_or_default(), move || {
        if let Some(child) = child {
            child.render();
        }
    });
}
