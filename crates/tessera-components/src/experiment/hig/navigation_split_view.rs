//! Navigation split views with a sidebar, optional content column, and detail.
//!
//! ## Usage
//!
//! Build mail-, notes-, or settings-style windows that browse a hierarchy beside its detail.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Color, Dp, Modifier, RenderSlot,
    accesskit::Role,
    layout::{Stack, layout},
    remember, tessera, use_context,
};

use crate::{modifier::ModifierExt as _, shape_def::Shape};

use super::{
    split_view::{SplitAxis, split},
    theme::{Appearance, HigTheme},
};

#[cfg(test)]
#[path = "navigation_split_view_tests.rs"]
mod tests;

/// Which columns of a [`navigation_split_view`] are shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationSplitViewVisibility {
    /// The platform default, all columns on macOS.
    #[default]
    Automatic,
    /// Every column.
    All,
    /// The content and detail columns, hiding the sidebar.
    DoubleColumn,
    /// Only the detail column.
    DetailOnly,
}

/// How a [`navigation_split_view`] shares its width between the detail and the other columns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationSplitViewStyle {
    /// The platform default, balanced on macOS.
    #[default]
    Automatic,
    /// The leading columns keep their widths beside the detail.
    Balanced,
    /// The detail keeps the largest share: each leading column takes at most a third of the
    /// width it divides.
    ProminentDetail,
}

/// Largest share of the width a leading column takes in the prominent detail style.
const PROMINENT_SHARE: f64 = 1.0 / 3.0;

/// # navigation_split_view
///
/// Shows a sidebar, an optional content column, and a detail column side by side.
///
/// ## Usage
///
/// Browse mailboxes, folders, or settings categories in a sidebar beside their detail.
///
/// ## Parameters
///
/// - `sidebar` — the leading column, usually a sidebar [`list`](super::list::list).
/// - `content` — an optional middle column between the sidebar and the detail.
/// - `detail` — the trailing column.
/// - `column_visibility` — which columns are shown; defaults to all.
/// - `style` — how the detail shares the width; defaults to [`NavigationSplitViewStyle::Automatic`].
/// - `sidebar_width`, `content_width` — initial widths; default to 220pt and 260pt.
/// - `modifier` — size and placement; the view fills the space its container offers.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     list::{ListStyle, list, list_row},
///     navigation_split_view::navigation_split_view,
///     text::text,
/// };
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// navigation_split_view()
///     .sidebar(|| {
///         list().style(ListStyle::Sidebar).content(|| {
///             list_row(0).content(|| {
///                 text("Inbox");
///             });
///         });
///     })
///     .detail(|| {
///         text("No message selected");
///     });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn navigation_split_view(
    sidebar: Option<RenderSlot>,
    content: Option<RenderSlot>,
    detail: Option<RenderSlot>,
    column_visibility: Option<NavigationSplitViewVisibility>,
    style: Option<NavigationSplitViewStyle>,
    sidebar_width: Option<Dp>,
    content_width: Option<Dp>,
    modifier: Option<Modifier>,
) {
    let visibility = column_visibility.unwrap_or_default();
    let sidebar_extent = remember(|| sidebar_width.unwrap_or(Dp(220.0)).0);
    let content_extent = remember(|| content_width.unwrap_or(Dp(260.0)).0);
    let sidebar_hidden = matches!(
        visibility,
        NavigationSplitViewVisibility::DoubleColumn | NavigationSplitViewVisibility::DetailOnly
    );
    let content_hidden = visibility == NavigationSplitViewVisibility::DetailOnly;
    let max_share =
        (style == Some(NavigationSplitViewStyle::ProminentDetail)).then_some(PROMINENT_SHARE);
    let detail = detail.unwrap_or_else(RenderSlot::empty);
    let sidebar = sidebar.unwrap_or_else(RenderSlot::empty);
    let trailing = match content {
        Some(content) => RenderSlot::new(move || {
            split(
                SplitAxis::Horizontal,
                content_extent,
                160.0,
                content_hidden,
                content,
                detail,
            )
            .max_share_optional(max_share);
        }),
        None => detail,
    };
    split(
        SplitAxis::Horizontal,
        sidebar_extent,
        150.0,
        sidebar_hidden,
        RenderSlot::new(move || sidebar_panel(sidebar)),
        trailing,
    )
    .max_share_optional(max_share)
    .modifier(modifier.unwrap_or_default().semantics(SemanticsArgs {
        role: Some(Role::Group),
        ..Default::default()
    }));
}

/// The sidebar column: a rounded panel inset from the window edges.
#[tessera]
fn sidebar_panel(content: RenderSlot) {
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let fill = if dark {
        Color::from_rgb(0.03, 0.03, 0.03)
    } else {
        Color::from_rgb(0.86, 0.86, 0.86)
    };
    let shape = Shape::rounded_rectangle(Dp(12.0));
    layout(Stack)
        .modifier(
            Modifier::new()
                .fill_max_size()
                .padding_all(Dp(8.0))
                .background_with_shape(fill, shape)
                .clip_shape(shape),
        )
        .child(content);
}
