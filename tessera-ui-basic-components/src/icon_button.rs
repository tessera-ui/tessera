//! An interactive button that displays an icon.
//!
//! ## Usage
//!
//! Use for compact actions where an icon is sufficient to convey the meaning.
use derive_builder::Builder;
use tessera_ui::tessera;

use crate::{
    RippleState,
    button::{ButtonArgs, button},
    glass_button::{GlassButtonArgs, glass_button},
    icon::{IconArgs, icon},
};

/// Arguments for [`icon_button`].
#[derive(Clone, Builder)]
#[builder(pattern = "owned")]
pub struct IconButtonArgs {
    /// Appearance/behavior settings for the underlying [`button`].
    #[builder(default = "ButtonArgs::default()", setter(custom))]
    pub button: ButtonArgs,
    /// Icon that will be rendered at the center of the button.
    #[builder(setter(into))]
    pub icon: IconArgs,
}

impl IconButtonArgsBuilder {
    /// Override the [`ButtonArgs`] using either a ready instance or a builder-produced value.
    pub fn button(mut self, button: impl Into<ButtonArgs>) -> Self {
        self.button = Some(button.into());
        self
    }
}

/// Lifted [`glass_button`] counterpart for icon buttons.
#[derive(Clone, Builder)]
#[builder(pattern = "owned")]
pub struct GlassIconButtonArgs {
    /// Appearance/behavior settings for the underlying [`glass_button`].
    #[builder(default = "GlassButtonArgs::default()", setter(custom))]
    pub button: GlassButtonArgs,
    /// Icon rendered at the center of the glass button.
    #[builder(setter(into))]
    pub icon: IconArgs,
}

impl GlassIconButtonArgsBuilder {
    /// Override the [`GlassButtonArgs`] using either a ready instance or a builder-produced value.
    pub fn button(mut self, button: impl Into<GlassButtonArgs>) -> Self {
        self.button = Some(button.into());
        self
    }
}

/// # icon_button
///
/// Renders a standard button with an icon as its content.
///
/// ## Usage
///
/// Use for common actions like "edit", "delete", or "settings" in a toolbar or list item.
///
/// ## Parameters
///
/// - `args` — configures the underlying button and the icon; see [`IconButtonArgs`].
/// - `ripple_state` — a clonable [`RippleState`] to manage the ripple animation.
///
/// ## Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use tessera_ui_basic_components::{
///     icon_button::{icon_button, IconButtonArgsBuilder},
///     button::ButtonArgsBuilder,
///     icon::IconArgsBuilder,
///     image_vector::{ImageVectorSource, load_image_vector_from_source},
///     ripple_state::RippleState,
/// };
///
/// let ripple_state = RippleState::new();
/// let svg_path = "../assets/emoji_u1f416.svg";
/// let vector_data = load_image_vector_from_source(
///     &ImageVectorSource::Path(svg_path.to_string())
/// ).unwrap();
///
/// icon_button(
///     IconButtonArgsBuilder::default()
///         .button(
///             ButtonArgsBuilder::default()
///                 .on_click(Arc::new(|| {}))
///                 .build()
///                 .unwrap()
///         )
///         .icon(IconArgsBuilder::default().content(vector_data.clone()).build().expect("builder construction failed"))
///         .build()
///         .unwrap(),
///     ripple_state,
/// );
/// ```
#[tessera]
pub fn icon_button(args: impl Into<IconButtonArgs>, ripple_state: RippleState) {
    let args: IconButtonArgs = args.into();
    let icon_args = args.icon.clone();

    button(args.button, ripple_state, move || {
        icon(icon_args.clone());
    });
}

/// # glass_icon_button
///
/// Renders a button with a glass effect and an icon as its content.
///
/// ## Usage
///
/// Use for prominent icon-based actions in a modern, layered UI.
///
/// ## Parameters
///
/// - `args` — configures the underlying glass button and the icon; see [`GlassIconButtonArgs`].
/// - `ripple_state` — a clonable [`RippleState`] to manage the ripple animation.
///
/// ## Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use tessera_ui_basic_components::{
///     icon_button::{glass_icon_button, GlassIconButtonArgsBuilder},
///     glass_button::GlassButtonArgsBuilder,
///     icon::IconArgsBuilder,
///     image_vector::{ImageVectorSource, load_image_vector_from_source},
///     ripple_state::RippleState,
/// };
///
/// let ripple_state = RippleState::new();
/// let svg_path = "../assets/emoji_u1f416.svg";
/// let vector_data = load_image_vector_from_source(
///     &ImageVectorSource::Path(svg_path.to_string())
/// ).unwrap();
///
/// glass_icon_button(
///     GlassIconButtonArgsBuilder::default()
///         .button(
///             GlassButtonArgsBuilder::default()
///                 .on_click(Arc::new(|| {}))
///                 .build()
///                 .unwrap()
///         )
///         .icon(IconArgsBuilder::default().content(vector_data).build().expect("builder construction failed"))
///         .build()
///         .unwrap(),
///     ripple_state,
/// );
/// ```
#[tessera]
pub fn glass_icon_button(args: impl Into<GlassIconButtonArgs>, ripple_state: RippleState) {
    let args: GlassIconButtonArgs = args.into();
    let icon_args = args.icon.clone();

    glass_button(args.button, ripple_state, move || {
        icon(icon_args.clone());
    });
}

