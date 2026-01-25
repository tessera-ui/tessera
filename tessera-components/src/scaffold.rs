//! Scaffold layout for persistent bars and app content.
//!
//! ## Usage
//!
//! Layer top/bottom bars, floating buttons, and snackbars above app content.
use std::sync::Arc;

use derive_setters::Setters;
use tessera_ui::{Dp, Modifier, tessera};

use crate::{
    alignment::Alignment,
    boxed::{BoxedArgs, boxed},
    modifier::{ModifierExt as _, Padding},
};

/// Configuration arguments for [`scaffold`].
#[derive(Clone, Setters)]
pub struct ScaffoldArgs {
    /// Modifier chain applied to the scaffold container.
    pub modifier: Modifier,
    /// Padding applied around the content area.
    pub content_padding: Padding,
    /// Reserved height for the top bar.
    pub top_bar_height: Dp,
    /// Reserved height for the bottom bar.
    pub bottom_bar_height: Dp,
    /// Optional top bar slot.
    #[setters(skip)]
    pub top_bar: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Optional bottom bar slot.
    #[setters(skip)]
    pub bottom_bar: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Optional floating action button slot.
    #[setters(skip)]
    pub floating_action_button: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Alignment used for the floating action button.
    pub floating_action_button_alignment: Alignment,
    /// Additional x/y offset applied to the floating action button.
    pub floating_action_button_offset: [Dp; 2],
    /// Optional snackbar host slot.
    #[setters(skip)]
    pub snackbar_host: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Alignment used for the snackbar host.
    pub snackbar_alignment: Alignment,
    /// Additional x/y offset applied to the snackbar host.
    pub snackbar_offset: [Dp; 2],
}

impl Default for ScaffoldArgs {
    fn default() -> Self {
        Self {
            modifier: Modifier::new().fill_max_size(),
            content_padding: Padding::all(Dp(0.0)),
            top_bar_height: Dp(0.0),
            bottom_bar_height: Dp(0.0),
            top_bar: None,
            bottom_bar: None,
            floating_action_button: None,
            floating_action_button_alignment: Alignment::BottomEnd,
            floating_action_button_offset: [Dp(0.0), Dp(0.0)],
            snackbar_host: None,
            snackbar_alignment: Alignment::BottomCenter,
            snackbar_offset: [Dp(0.0), Dp(0.0)],
        }
    }
}

impl ScaffoldArgs {
    /// Sets the top bar slot content.
    pub fn top_bar<F>(mut self, top_bar: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.top_bar = Some(Arc::new(top_bar));
        self
    }

    /// Sets the top bar slot content using a shared callback.
    pub fn top_bar_shared(mut self, top_bar: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.top_bar = Some(top_bar);
        self
    }

    /// Sets the bottom bar slot content.
    pub fn bottom_bar<F>(mut self, bottom_bar: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.bottom_bar = Some(Arc::new(bottom_bar));
        self
    }

    /// Sets the bottom bar slot content using a shared callback.
    pub fn bottom_bar_shared(mut self, bottom_bar: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.bottom_bar = Some(bottom_bar);
        self
    }

    /// Sets the floating action button slot content.
    pub fn floating_action_button<F>(mut self, floating_action_button: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.floating_action_button = Some(Arc::new(floating_action_button));
        self
    }

    /// Sets the floating action button slot content using a shared callback.
    pub fn floating_action_button_shared(
        mut self,
        floating_action_button: Arc<dyn Fn() + Send + Sync>,
    ) -> Self {
        self.floating_action_button = Some(floating_action_button);
        self
    }

    /// Sets the snackbar host slot content.
    pub fn snackbar_host<F>(mut self, snackbar_host: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.snackbar_host = Some(Arc::new(snackbar_host));
        self
    }

    /// Sets the snackbar host slot content using a shared callback.
    pub fn snackbar_host_shared(mut self, snackbar_host: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.snackbar_host = Some(snackbar_host);
        self
    }
}

fn scaffold_content_padding(base: Padding, top_bar_height: Dp, bottom_bar_height: Dp) -> Padding {
    Padding::new(
        base.left,
        Dp(base.top.0 + top_bar_height.0),
        base.right,
        Dp(base.bottom.0 + bottom_bar_height.0),
    )
}

fn overlay_offset(alignment: Alignment, offset: [Dp; 2], bottom_bar_height: Dp) -> [Dp; 2] {
    let base_y = match alignment {
        Alignment::BottomStart | Alignment::BottomCenter | Alignment::BottomEnd => {
            Dp(-bottom_bar_height.0)
        }
        _ => Dp(0.0),
    };
    [offset[0], Dp(offset[1].0 + base_y.0)]
}

/// # scaffold
///
/// Layout top/bottom bars with floating content for app screens with persistent
/// actions.
///
/// ## Usage
///
/// Use for screens with app bars, floating actions, and transient messages.
///
/// ## Parameters
///
/// - `args` — configures slots, insets, and padding; see [`ScaffoldArgs`].
/// - `content` — closure that renders the main content.
///
/// ## Examples
///
/// ```
/// use tessera_components::app_bar::{AppBarDefaults, TopAppBarArgs, top_app_bar};
/// use tessera_components::scaffold::{ScaffoldArgs, scaffold};
/// use tessera_components::text::{TextArgs, text};
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     material_theme(
///         || MaterialTheme::default(),
///         || {
///             let counter = remember(|| 1u32);
///             scaffold(
///                 ScaffoldArgs::default()
///                     .top_bar_height(AppBarDefaults::TOP_APP_BAR_HEIGHT)
///                     .top_bar(|| {
///                         top_app_bar(TopAppBarArgs::new("Inbox"));
///                     }),
///                 || {
///                     text(TextArgs::default().text("Hello scaffold"));
///                 },
///             );
///             assert_eq!(counter.get(), 1);
///         },
///     );
/// }
/// ```
#[tessera]
pub fn scaffold(args: impl Into<ScaffoldArgs>, content: impl FnOnce() + Send + Sync + 'static) {
    let args: ScaffoldArgs = args.into();
    let modifier = args.modifier;
    let content_padding = scaffold_content_padding(
        args.content_padding,
        args.top_bar_height,
        args.bottom_bar_height,
    );
    let top_bar = args.top_bar;
    let bottom_bar = args.bottom_bar;
    let floating_action_button = args.floating_action_button;
    let fab_alignment = args.floating_action_button_alignment;
    let fab_offset = overlay_offset(
        fab_alignment,
        args.floating_action_button_offset,
        args.bottom_bar_height,
    );
    let snackbar_host = args.snackbar_host;
    let snackbar_alignment = args.snackbar_alignment;
    let snackbar_offset = overlay_offset(
        snackbar_alignment,
        args.snackbar_offset,
        args.bottom_bar_height,
    );

    modifier.run(move || {
        boxed(BoxedArgs::default(), |scope| {
            scope.child(move || {
                Modifier::new()
                    .padding(content_padding)
                    .fill_max_size()
                    .run(content);
            });
            if let Some(bottom_bar) = bottom_bar {
                scope.child_with_alignment(Alignment::BottomCenter, move || {
                    bottom_bar();
                });
            }
            if let Some(top_bar) = top_bar {
                scope.child_with_alignment(Alignment::TopCenter, move || {
                    top_bar();
                });
            }
            if let Some(snackbar_host) = snackbar_host {
                scope.child_with_alignment(snackbar_alignment, move || {
                    Modifier::new()
                        .offset(snackbar_offset[0], snackbar_offset[1])
                        .run(move || {
                            snackbar_host();
                        });
                });
            }
            if let Some(floating_action_button) = floating_action_button {
                scope.child_with_alignment(fab_alignment, move || {
                    Modifier::new()
                        .offset(fab_offset[0], fab_offset[1])
                        .run(move || {
                            floating_action_button();
                        });
                });
            }
        });
    });
}
