//! Icon-aware button helpers built by composing `button`, `glass_button`, and `icon`.
//!
//! These components wrap the lower-level primitives so consumers can render a tappable icon
//! (regular or glass-style) with a single call, while still exposing the underlying
//! configuration via `ButtonArgs` / `GlassButtonArgs` and `IconArgs`.

use std::sync::Arc;

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

/// A convenience wrapper that renders an [`icon`] inside a regular [`button`].
#[tessera]
pub fn icon_button(args: impl Into<IconButtonArgs>, ripple_state: Arc<RippleState>) {
    let args: IconButtonArgs = args.into();
    let icon_args = args.icon.clone();

    button(args.button, ripple_state, move || {
        icon(icon_args.clone());
    });
}

/// A glass-styled variant of [`icon_button`] built on top of [`glass_button`].
#[tessera]
pub fn glass_icon_button(args: impl Into<GlassIconButtonArgs>, ripple_state: Arc<RippleState>) {
    let args: GlassIconButtonArgs = args.into();
    let icon_args = args.icon.clone();

    glass_button(args.button, ripple_state, move || {
        icon(icon_args.clone());
    });
}
