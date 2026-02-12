//! Search bar component - capture queries with optional suggestion content.
//!
//! ## Usage
//!
//! Use to collect search queries and show suggestions or results as the user
//! types.
use std::sync::Arc;

use derive_setters::Setters;
use tessera_ui::{
    Color, CursorEventContent, Dp, Modifier, PressKeyEventType, State, remember, tessera,
    use_context, winit,
};

use crate::{
    column::{ColumnArgs, column},
    divider::{DividerArgs, horizontal_divider},
    modifier::ModifierExt as _,
    pos_misc::is_position_in_component,
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text_field::{
        TextFieldArgs, TextFieldDefaults, TextFieldLineLimit, text_field_with_controller,
    },
    text_input::TextInputController,
    theme::MaterialTheme,
};

const DEFAULT_RESULTS_PADDING: Dp = Dp(16.0);

/// Color values used by search bars.
#[derive(Clone, Copy, Debug)]
pub struct SearchBarColors {
    /// Container color for the search bar and results surface.
    pub container_color: Color,
    /// Divider color between the input field and results content.
    pub divider_color: Color,
}

/// Defaults for search bars.
pub struct SearchBarDefaults;

impl SearchBarDefaults {
    /// Default height for the search input field.
    pub const INPUT_HEIGHT: Dp = Dp(56.0);
    /// Default tonal elevation for search bars.
    pub const TONAL_ELEVATION: Dp = Dp(0.0);
    /// Default shadow elevation for search bars.
    pub const SHADOW_ELEVATION: Dp = Dp(0.0);
    /// Default gap between a docked input field and its results surface.
    pub const DOCKED_DROPDOWN_GAP: Dp = Dp(2.0);
    /// Default rounded corner radius for a docked results surface.
    pub const DOCKED_DROPDOWN_RADIUS: Dp = Dp(12.0);

    /// Default shape for a collapsed search input field.
    pub fn input_shape() -> Shape {
        Shape::capsule()
    }

    /// Default shape for a docked search bar container.
    pub fn docked_shape() -> Shape {
        use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .shapes
            .extra_large
    }

    /// Default shape for a docked results surface.
    pub fn docked_dropdown_shape() -> Shape {
        Shape::rounded_rectangle(Self::DOCKED_DROPDOWN_RADIUS)
    }

    /// Default colors for search bars.
    pub fn colors() -> SearchBarColors {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        SearchBarColors {
            container_color: scheme.surface_container_high,
            divider_color: scheme.outline,
        }
    }
}

/// Configuration arguments for search bars.
#[derive(Clone, Setters)]
pub struct SearchBarArgs {
    /// Modifier chain applied to the search bar container.
    pub modifier: Modifier,
    /// Whether the input is enabled.
    pub enabled: bool,
    /// Whether the input is read-only.
    pub read_only: bool,
    /// Whether the bar is active (expanded) for declarative usage.
    pub is_active: bool,
    /// Optional placeholder text shown when the query is empty.
    #[setters(strip_option, into)]
    pub placeholder: Option<String>,
    /// Optional leading icon shown before the input text.
    #[setters(skip)]
    pub leading_icon: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Optional trailing icon shown after the input text.
    #[setters(skip)]
    pub trailing_icon: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Called when the query changes. Return value is the text to keep.
    #[setters(skip)]
    pub on_query_change: Arc<dyn Fn(String) -> String + Send + Sync>,
    /// Called when the user submits the query.
    #[setters(skip)]
    pub on_search: Arc<dyn Fn(String) + Send + Sync>,
    /// Called when the active state changes due to user interaction.
    #[setters(skip)]
    pub on_active_change: Arc<dyn Fn(bool) + Send + Sync>,
    /// Shape used for the input field.
    pub shape: Shape,
    /// Colors for the search bar.
    pub colors: SearchBarColors,
    /// Tonal elevation for the search bar container.
    pub tonal_elevation: Dp,
    /// Shadow elevation for the search bar container.
    pub shadow_elevation: Dp,
    /// Shape used for docked results.
    pub dropdown_shape: Shape,
    /// Gap between input field and docked results.
    pub dropdown_gap: Dp,
    /// Padding inside the results container.
    pub content_padding: Dp,
}

impl SearchBarArgs {
    /// Set the query change handler.
    pub fn on_query_change<F>(mut self, on_query_change: F) -> Self
    where
        F: Fn(String) -> String + Send + Sync + 'static,
    {
        self.on_query_change = Arc::new(on_query_change);
        self
    }

    /// Set the query change handler using a shared callback.
    pub fn on_query_change_shared(
        mut self,
        on_query_change: Arc<dyn Fn(String) -> String + Send + Sync>,
    ) -> Self {
        self.on_query_change = on_query_change;
        self
    }

    /// Set the search submit handler.
    pub fn on_search<F>(mut self, on_search: F) -> Self
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.on_search = Arc::new(on_search);
        self
    }

    /// Set the search submit handler using a shared callback.
    pub fn on_search_shared(mut self, on_search: Arc<dyn Fn(String) + Send + Sync>) -> Self {
        self.on_search = on_search;
        self
    }

    /// Set the active-state change handler.
    pub fn on_active_change<F>(mut self, on_active_change: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_active_change = Arc::new(on_active_change);
        self
    }

    /// Set the active-state change handler using a shared callback.
    pub fn on_active_change_shared(
        mut self,
        on_active_change: Arc<dyn Fn(bool) + Send + Sync>,
    ) -> Self {
        self.on_active_change = on_active_change;
        self
    }

    /// Set the leading icon slot.
    pub fn leading_icon<F>(mut self, leading_icon: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.leading_icon = Some(Arc::new(leading_icon));
        self
    }

    /// Set the leading icon slot using a shared callback.
    pub fn leading_icon_shared(mut self, leading_icon: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.leading_icon = Some(leading_icon);
        self
    }

    /// Set the trailing icon slot.
    pub fn trailing_icon<F>(mut self, trailing_icon: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.trailing_icon = Some(Arc::new(trailing_icon));
        self
    }

    /// Set the trailing icon slot using a shared callback.
    pub fn trailing_icon_shared(mut self, trailing_icon: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.trailing_icon = Some(trailing_icon);
        self
    }
}

impl Default for SearchBarArgs {
    fn default() -> Self {
        Self {
            modifier: Modifier::new(),
            enabled: true,
            read_only: false,
            is_active: false,
            placeholder: None,
            leading_icon: None,
            trailing_icon: None,
            on_query_change: Arc::new(|text| text),
            on_search: Arc::new(|_| {}),
            on_active_change: Arc::new(|_| {}),
            shape: SearchBarDefaults::input_shape(),
            colors: SearchBarDefaults::colors(),
            tonal_elevation: SearchBarDefaults::TONAL_ELEVATION,
            shadow_elevation: SearchBarDefaults::SHADOW_ELEVATION,
            dropdown_shape: SearchBarDefaults::docked_dropdown_shape(),
            dropdown_gap: SearchBarDefaults::DOCKED_DROPDOWN_GAP,
            content_padding: DEFAULT_RESULTS_PADDING,
        }
    }
}

/// Controller for search bars, managing active state and the current query.
#[derive(Clone)]
pub struct SearchBarController {
    is_active: bool,
    query: String,
}

impl SearchBarController {
    /// Creates a new controller.
    pub fn new(initial_active: bool) -> Self {
        Self {
            is_active: initial_active,
            query: String::new(),
        }
    }

    /// Opens the search bar.
    pub fn open(&mut self) {
        self.is_active = true;
    }

    /// Closes the search bar.
    pub fn close(&mut self) {
        self.is_active = false;
    }

    /// Toggles the active state.
    pub fn toggle(&mut self) {
        self.is_active = !self.is_active;
    }

    /// Returns whether the search bar is active.
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Updates the stored query.
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
    }

    /// Returns the current query.
    pub fn query(&self) -> &str {
        &self.query
    }
}

impl Default for SearchBarController {
    fn default() -> Self {
        Self::new(false)
    }
}

#[derive(Clone, Copy)]
enum SearchBarLayoutKind {
    FullScreen,
    Docked,
}

/// # search_bar
///
/// Show a full-screen search bar that expands to display results.
///
/// ## Usage
///
/// Use for immersive search flows where results take over the screen.
///
/// ## Parameters
///
/// - `args` - configures search behavior and appearance; see [`SearchBarArgs`].
/// - `content` - renders search results when the bar is active.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// # #[tessera]
/// # fn component() {
/// use tessera_components::search::{SearchBarArgs, search_bar};
/// material_theme(MaterialTheme::default, || {
///     let args = SearchBarArgs::default().is_active(true);
///     assert!(args.is_active);
///     search_bar(args, || { /* results */ });
/// });
/// # }
/// # component();
/// ```
#[tessera]
pub fn search_bar(args: impl Into<SearchBarArgs>, content: impl FnOnce() + Send + Sync + 'static) {
    let args: SearchBarArgs = args.into();
    let controller = remember(|| SearchBarController::new(args.is_active));
    if args.is_active != controller.with(|c| c.is_active()) {
        if args.is_active {
            controller.with_mut(|c| c.open());
        } else {
            controller.with_mut(|c| c.close());
        }
    }
    search_bar_with_controller(args, controller, content);
}

/// # search_bar_with_controller
///
/// Controlled version of [`search_bar`] that accepts an external controller.
///
/// ## Usage
///
/// Use when you need to control the active state programmatically.
///
/// ## Parameters
///
/// - `args` - configures search behavior and appearance; see [`SearchBarArgs`].
/// - `controller` - a [`SearchBarController`] managing active state and query.
/// - `content` - renders search results when the bar is active.
///
/// ## Examples
///
/// ```
/// use tessera_components::search::{
///     SearchBarArgs, SearchBarController, search_bar_with_controller,
/// };
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn component() {
///     let controller = remember(|| SearchBarController::new(false));
///     assert!(!controller.with(|c| c.is_active()));
///     controller.with_mut(|c| c.open());
///     assert!(controller.with(|c| c.is_active()));
///     material_theme(MaterialTheme::default, || {
///         search_bar_with_controller(SearchBarArgs::default(), controller, || {});
///     });
/// }
/// component();
/// ```
#[tessera]
pub fn search_bar_with_controller(
    args: impl Into<SearchBarArgs>,
    controller: State<SearchBarController>,
    content: impl FnOnce() + Send + Sync + 'static,
) {
    let args: SearchBarArgs = args.into();
    let modifier = args.modifier;
    modifier.run(move || {
        search_bar_inner(SearchBarLayoutKind::FullScreen, args, controller, content);
    });
}

/// # docked_search_bar
///
/// Show a docked search bar with a results surface below the input.
///
/// ## Usage
///
/// Use for search experiences embedded within existing layouts.
///
/// ## Parameters
///
/// - `args` - configures search behavior and appearance; see [`SearchBarArgs`].
/// - `content` - renders search results when the bar is active.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// # #[tessera]
/// # fn component() {
/// use tessera_components::search::{SearchBarArgs, docked_search_bar};
/// material_theme(MaterialTheme::default, || {
///     let args = SearchBarArgs::default().is_active(true);
///     assert!(args.is_active);
///     docked_search_bar(args, || { /* results */ });
/// });
/// # }
/// # component();
/// ```
#[tessera]
pub fn docked_search_bar(
    args: impl Into<SearchBarArgs>,
    content: impl FnOnce() + Send + Sync + 'static,
) {
    let args: SearchBarArgs = args.into();
    let controller = remember(|| SearchBarController::new(args.is_active));
    if args.is_active != controller.with(|c| c.is_active()) {
        if args.is_active {
            controller.with_mut(|c| c.open());
        } else {
            controller.with_mut(|c| c.close());
        }
    }
    docked_search_bar_with_controller(args, controller, content);
}

/// # docked_search_bar_with_controller
///
/// Controlled version of [`docked_search_bar`] that accepts an external
/// controller.
///
/// ## Usage
///
/// Use when you need to control the active state programmatically.
///
/// ## Parameters
///
/// - `args` - configures search behavior and appearance; see [`SearchBarArgs`].
/// - `controller` - a [`SearchBarController`] managing active state and query.
/// - `content` - renders search results when the bar is active.
///
/// ## Examples
///
/// ```
/// use tessera_components::search::{
///     SearchBarArgs, SearchBarController, docked_search_bar_with_controller,
/// };
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn component() {
///     let controller = remember(|| SearchBarController::new(false));
///     assert!(!controller.with(|c| c.is_active()));
///     controller.with_mut(|c| c.open());
///     assert!(controller.with(|c| c.is_active()));
///     material_theme(MaterialTheme::default, || {
///         docked_search_bar_with_controller(SearchBarArgs::default(), controller, || {});
///     });
/// }
/// component();
/// ```
#[tessera]
pub fn docked_search_bar_with_controller(
    args: impl Into<SearchBarArgs>,
    controller: State<SearchBarController>,
    content: impl FnOnce() + Send + Sync + 'static,
) {
    let args: SearchBarArgs = args.into();
    let modifier = args.modifier;
    modifier.run(move || {
        search_bar_inner(SearchBarLayoutKind::Docked, args, controller, content);
    });
}

#[tessera]
fn search_bar_inner(
    kind: SearchBarLayoutKind,
    args: SearchBarArgs,
    controller: State<SearchBarController>,
    content: impl FnOnce() + Send + Sync + 'static,
) {
    let mut field_args = TextFieldArgs::filled();
    let font_size = field_args.font_size;
    let line_height = field_args.line_height;

    let input_controller = remember(|| TextInputController::new(font_size, line_height));
    let synced_query = remember(String::new);

    sync_query(&controller, &input_controller, &synced_query);

    let on_query_change = args.on_query_change.clone();
    field_args = field_args.on_change(move |text| {
        let next = (on_query_change)(text);
        controller.with_mut(|c| c.set_query(next.clone()));
        synced_query.set(next.clone());
        next
    });

    field_args = field_args
        .enabled(args.enabled)
        .read_only(args.read_only)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(SearchBarDefaults::INPUT_HEIGHT),
        )
        .min_height(SearchBarDefaults::INPUT_HEIGHT)
        .shape(args.shape)
        .background_color(args.colors.container_color)
        .focus_background_color(args.colors.container_color)
        .border_width(Dp(0.0))
        .border_color(Color::TRANSPARENT)
        .focus_border_width(Dp(0.0))
        .focus_border_color(Color::TRANSPARENT)
        .show_indicator(false)
        .padding(TextFieldDefaults::CONTENT_PADDING)
        .line_limit(TextFieldLineLimit::SingleLine);

    if let Some(placeholder) = args.placeholder.clone() {
        field_args = field_args.placeholder(placeholder);
    }

    if let Some(leading_icon) = args.leading_icon.clone() {
        field_args = field_args.leading_icon_shared(leading_icon);
    }

    if let Some(trailing_icon) = args.trailing_icon.clone() {
        field_args = field_args.trailing_icon_shared(trailing_icon);
    }

    let on_active_change = args.on_active_change.clone();
    let on_search = args.on_search.clone();
    let enabled = args.enabled;
    input_handler(move |input| {
        if !enabled {
            return;
        }
        let is_active = controller.with(|c| c.is_active());
        let cursor_pos = input.cursor_position_rel;
        let has_left_click = input.cursor_events.iter().any(|event| {
            matches!(
                event.content,
                CursorEventContent::Pressed(PressKeyEventType::Left)
            )
        });

        if has_left_click
            && let Some(cursor_pos) = cursor_pos
            && is_position_in_component(input.computed_data, cursor_pos)
            && !is_active
        {
            controller.with_mut(|c| c.open());
            (on_active_change)(true);
        }

        for event in input.keyboard_events.iter() {
            if event.state == winit::event::ElementState::Pressed {
                if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                    match code {
                        winit::keyboard::KeyCode::Escape => {
                            if is_active {
                                controller.with_mut(|c| c.close());
                                (on_active_change)(false);
                            }
                        }
                        winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                            if is_active {
                                let query = controller.with(|c| c.query().to_string());
                                (on_search)(query);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    let dropdown_gap = args.dropdown_gap;
    let results_container_color = args.colors.container_color;
    let results_divider_color = args.colors.divider_color;
    let results_dropdown_shape = args.dropdown_shape;
    let results_tonal_elevation = args.tonal_elevation;
    let results_shadow_elevation = args.shadow_elevation;
    let results_content_padding = args.content_padding;
    column(ColumnArgs::default(), move |scope| {
        scope.child(move || {
            text_field_with_controller(field_args, input_controller);
        });

        if controller.with(|c| c.is_active()) {
            if matches!(kind, SearchBarLayoutKind::Docked) {
                scope.child(move || spacer(Modifier::new().height(dropdown_gap)));
            }

            scope.child(move || {
                render_results_surface(
                    kind,
                    results_container_color,
                    results_divider_color,
                    results_dropdown_shape,
                    results_tonal_elevation,
                    results_shadow_elevation,
                    results_content_padding,
                    content,
                );
            });
        }
    });
}

fn render_results_surface(
    kind: SearchBarLayoutKind,
    container_color: Color,
    divider_color: Color,
    dropdown_shape: Shape,
    tonal_elevation: Dp,
    shadow_elevation: Dp,
    content_padding: Dp,
    content: impl FnOnce() + Send + Sync + 'static,
) {
    let shape = match kind {
        SearchBarLayoutKind::FullScreen => Shape::RECTANGLE,
        SearchBarLayoutKind::Docked => dropdown_shape,
    };

    let modifier = match kind {
        SearchBarLayoutKind::FullScreen => Modifier::new().fill_max_width().fill_max_height(),
        SearchBarLayoutKind::Docked => Modifier::new().fill_max_width(),
    };

    surface(
        SurfaceArgs::default()
            .style(container_color.into())
            .shape(shape)
            .modifier(modifier)
            .tonal_elevation(tonal_elevation)
            .block_input(true)
            .elevation(shadow_elevation),
        move || {
            column(ColumnArgs::default(), |scope| {
                scope.child(move || {
                    horizontal_divider(DividerArgs::default().color(divider_color));
                });
                scope.child(move || {
                    Modifier::new().padding_all(content_padding).run(content);
                });
            });
        },
    );
}

fn sync_query(
    controller: &State<SearchBarController>,
    input: &State<TextInputController>,
    synced_query: &State<String>,
) {
    let needs_sync =
        controller.with(|c| synced_query.with(|current| current.as_str() != c.query()));
    if needs_sync {
        let query = controller.with(|c| c.query().to_owned());
        input.with_mut(|c| c.set_text(&query));
        synced_query.set(query);
    }
}
