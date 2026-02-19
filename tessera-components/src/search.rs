//! Search bar component - capture queries with optional suggestion content.
//!
//! ## Usage
//!
//! Use to collect search queries and show suggestions or results as the user
//! types.
use derive_setters::Setters;
use tessera_ui::{
    CallbackWith, Color, CursorEventContent, Dp, Modifier, PressKeyEventType, RenderSlot, State,
    remember, tessera, use_context, winit,
};

use crate::{
    column::{ColumnArgs, column},
    divider::{DividerArgs, horizontal_divider},
    modifier::ModifierExt as _,
    pos_misc::is_position_inside_bounds,
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text_field::{TextFieldArgs, TextFieldDefaults, TextFieldLineLimit, text_field},
    text_input::TextInputController,
    theme::MaterialTheme,
};

const DEFAULT_RESULTS_PADDING: Dp = Dp(16.0);

/// Color values used by search bars.
#[derive(Clone, PartialEq, Copy, Debug)]
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
#[derive(PartialEq, Clone, Setters)]
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
    pub leading_icon: Option<RenderSlot>,
    /// Optional trailing icon shown after the input text.
    #[setters(skip)]
    pub trailing_icon: Option<RenderSlot>,
    /// Called when the query changes. Return value is the text to keep.
    #[setters(skip)]
    pub on_query_change: CallbackWith<String, String>,
    /// Called when the user submits the query.
    #[setters(skip)]
    pub on_search: CallbackWith<String, ()>,
    /// Called when the active state changes due to user interaction.
    #[setters(skip)]
    pub on_active_change: CallbackWith<bool, ()>,
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
    /// Optional external controller for active/query state.
    #[setters(skip)]
    pub controller: Option<State<SearchBarController>>,
    /// Optional results content slot.
    #[setters(skip)]
    pub content: Option<RenderSlot>,
}

impl SearchBarArgs {
    /// Set the query change handler.
    pub fn on_query_change<F>(mut self, on_query_change: F) -> Self
    where
        F: Fn(String) -> String + Send + Sync + 'static,
    {
        self.on_query_change = CallbackWith::new(on_query_change);
        self
    }

    /// Set the query change handler using a shared callback.
    pub fn on_query_change_shared(mut self, on_query_change: CallbackWith<String, String>) -> Self {
        self.on_query_change = on_query_change;
        self
    }

    /// Set the search submit handler.
    pub fn on_search<F>(mut self, on_search: F) -> Self
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.on_search = CallbackWith::new(on_search);
        self
    }

    /// Set the search submit handler using a shared callback.
    pub fn on_search_shared(mut self, on_search: CallbackWith<String, ()>) -> Self {
        self.on_search = on_search;
        self
    }

    /// Set the active-state change handler.
    pub fn on_active_change<F>(mut self, on_active_change: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_active_change = CallbackWith::new(on_active_change);
        self
    }

    /// Set the active-state change handler using a shared callback.
    pub fn on_active_change_shared(mut self, on_active_change: CallbackWith<bool, ()>) -> Self {
        self.on_active_change = on_active_change;
        self
    }

    /// Set the leading icon slot.
    pub fn leading_icon<F>(mut self, leading_icon: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.leading_icon = Some(RenderSlot::new(leading_icon));
        self
    }

    /// Set the leading icon slot using a shared callback.
    pub fn leading_icon_shared(mut self, leading_icon: RenderSlot) -> Self {
        self.leading_icon = Some(leading_icon);
        self
    }

    /// Set the trailing icon slot.
    pub fn trailing_icon<F>(mut self, trailing_icon: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.trailing_icon = Some(RenderSlot::new(trailing_icon));
        self
    }

    /// Set the trailing icon slot using a shared callback.
    pub fn trailing_icon_shared(mut self, trailing_icon: RenderSlot) -> Self {
        self.trailing_icon = Some(trailing_icon);
        self
    }

    /// Sets an external search bar controller.
    pub fn controller(mut self, controller: State<SearchBarController>) -> Self {
        self.controller = Some(controller);
        self
    }

    /// Sets the results content slot.
    pub fn content<F>(mut self, content: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.content = Some(RenderSlot::new(content));
        self
    }

    /// Sets the results content slot using a shared render slot.
    pub fn content_shared(mut self, content: impl Into<RenderSlot>) -> Self {
        self.content = Some(content.into());
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
            on_query_change: CallbackWith::new(|text: String| text),
            on_search: CallbackWith::new(|_: String| {}),
            on_active_change: CallbackWith::new(|_: bool| {}),
            shape: SearchBarDefaults::input_shape(),
            colors: SearchBarDefaults::colors(),
            tonal_elevation: SearchBarDefaults::TONAL_ELEVATION,
            shadow_elevation: SearchBarDefaults::SHADOW_ELEVATION,
            dropdown_shape: SearchBarDefaults::docked_dropdown_shape(),
            dropdown_gap: SearchBarDefaults::DOCKED_DROPDOWN_GAP,
            content_padding: DEFAULT_RESULTS_PADDING,
            controller: None,
            content: None,
        }
    }
}

/// Controller for search bars, managing active state and the current query.
#[derive(Clone, PartialEq)]
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

#[derive(Clone, PartialEq, Copy)]
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
/// let args = tessera_components::theme::MaterialThemeProviderArgs::new(
///     || MaterialTheme::default(),
///     || {
///         let args = SearchBarArgs::default()
///             .is_active(true)
///             .content(|| { /* results */ });
///         assert!(args.is_active);
///         search_bar(&args);
///     },
/// );
/// material_theme(&args);
/// # }
/// # component();
/// ```
#[tessera]
pub fn search_bar(args: &SearchBarArgs) {
    let render_args = build_search_bar_render_args(args.clone(), SearchBarLayoutKind::FullScreen);
    search_bar_node(render_args);
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
/// let args = tessera_components::theme::MaterialThemeProviderArgs::new(
///     || MaterialTheme::default(),
///     || {
///         let args = SearchBarArgs::default()
///             .is_active(true)
///             .content(|| { /* results */ });
///         assert!(args.is_active);
///         docked_search_bar(&args);
///     },
/// );
/// material_theme(&args);
/// # }
/// # component();
/// ```
#[tessera]
pub fn docked_search_bar(args: &SearchBarArgs) {
    let render_args = build_search_bar_render_args(args.clone(), SearchBarLayoutKind::Docked);
    search_bar_node(render_args);
}

fn build_search_bar_render_args(
    args: SearchBarArgs,
    kind: SearchBarLayoutKind,
) -> SearchBarRenderArgs {
    let content = args.content.unwrap_or_else(|| RenderSlot::new(|| {}));
    let controller = args
        .controller
        .unwrap_or_else(|| remember(|| SearchBarController::new(args.is_active)));
    if args.is_active != controller.with(|c| c.is_active()) {
        if args.is_active {
            controller.with_mut(|c| c.open());
        } else {
            controller.with_mut(|c| c.close());
        }
    }

    SearchBarRenderArgs {
        kind,
        modifier: args.modifier,
        enabled: args.enabled,
        read_only: args.read_only,
        placeholder: args.placeholder,
        leading_icon: args.leading_icon,
        trailing_icon: args.trailing_icon,
        on_query_change: args.on_query_change,
        on_search: args.on_search,
        on_active_change: args.on_active_change,
        shape: args.shape,
        colors: args.colors,
        tonal_elevation: args.tonal_elevation,
        shadow_elevation: args.shadow_elevation,
        dropdown_shape: args.dropdown_shape,
        dropdown_gap: args.dropdown_gap,
        content_padding: args.content_padding,
        controller,
        content,
    }
}

fn search_bar_node(args: SearchBarRenderArgs) {
    let modifier = args.modifier.clone();
    modifier.run(move || {
        search_bar_inner_node(&args);
    });
}

#[derive(Clone, PartialEq)]
struct SearchBarRenderArgs {
    kind: SearchBarLayoutKind,
    modifier: Modifier,
    enabled: bool,
    read_only: bool,
    placeholder: Option<String>,
    leading_icon: Option<RenderSlot>,
    trailing_icon: Option<RenderSlot>,
    on_query_change: CallbackWith<String, String>,
    on_search: CallbackWith<String, ()>,
    on_active_change: CallbackWith<bool, ()>,
    shape: Shape,
    colors: SearchBarColors,
    tonal_elevation: Dp,
    shadow_elevation: Dp,
    dropdown_shape: Shape,
    dropdown_gap: Dp,
    content_padding: Dp,
    controller: State<SearchBarController>,
    content: RenderSlot,
}

#[derive(Clone, PartialEq)]
struct SearchResultsSurfaceArgs {
    kind: SearchBarLayoutKind,
    container_color: Color,
    divider_color: Color,
    dropdown_shape: Shape,
    tonal_elevation: Dp,
    shadow_elevation: Dp,
    content_padding: Dp,
    content: RenderSlot,
}

#[tessera]
fn search_bar_inner_node(args: &SearchBarRenderArgs) {
    let args = args.clone();
    let kind = args.kind;
    let controller = args.controller;
    let content = args.content;
    let mut field_args = TextFieldArgs::filled();
    let font_size = field_args.font_size;
    let line_height = field_args.line_height;

    let input_controller = remember(|| TextInputController::new(font_size, line_height));
    let synced_query = remember(String::new);

    sync_query(&controller, &input_controller, &synced_query);

    let on_query_change = args.on_query_change.clone();
    field_args = field_args.on_change(move |text| {
        let next = on_query_change.call(text);
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
        field_args = field_args.leading_icon(move || {
            leading_icon.render();
        });
    }

    if let Some(trailing_icon) = args.trailing_icon.clone() {
        field_args = field_args.trailing_icon(move || {
            trailing_icon.render();
        });
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
            && is_position_inside_bounds(input.computed_data, cursor_pos)
            && !is_active
        {
            controller.with_mut(|c| c.open());
            on_active_change.call(true);
        }

        for event in input.keyboard_events.iter() {
            if event.state == winit::event::ElementState::Pressed {
                if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                    match code {
                        winit::keyboard::KeyCode::Escape => {
                            if is_active {
                                controller.with_mut(|c| c.close());
                                on_active_change.call(false);
                            }
                        }
                        winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                            if is_active {
                                let query = controller.with(|c| c.query().to_string());
                                on_search.call(query);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    let dropdown_gap = args.dropdown_gap;
    let results_surface_args = SearchResultsSurfaceArgs {
        kind,
        container_color: args.colors.container_color,
        divider_color: args.colors.divider_color,
        dropdown_shape: args.dropdown_shape,
        tonal_elevation: args.tonal_elevation,
        shadow_elevation: args.shadow_elevation,
        content_padding: args.content_padding,
        content: content.clone(),
    };
    column(ColumnArgs::default(), move |scope| {
        scope.child(move || {
            let field_args = field_args.clone().controller(input_controller);
            text_field(&field_args);
        });

        if controller.with(|c| c.is_active()) {
            if matches!(kind, SearchBarLayoutKind::Docked) {
                scope.child(move || {
                    spacer(&crate::spacer::SpacerArgs::new(
                        Modifier::new().height(dropdown_gap),
                    ))
                });
            }

            scope.child(move || {
                render_results_surface(&results_surface_args);
            });
        }
    });
}

fn render_results_surface(args: &SearchResultsSurfaceArgs) {
    let kind = args.kind;
    let container_color = args.container_color;
    let divider_color = args.divider_color;
    let dropdown_shape = args.dropdown_shape;
    let tonal_elevation = args.tonal_elevation;
    let shadow_elevation = args.shadow_elevation;
    let content_padding = args.content_padding;
    let content = args.content.clone();

    let shape = match kind {
        SearchBarLayoutKind::FullScreen => Shape::RECTANGLE,
        SearchBarLayoutKind::Docked => dropdown_shape,
    };

    let modifier = match kind {
        SearchBarLayoutKind::FullScreen => Modifier::new().fill_max_width().fill_max_height(),
        SearchBarLayoutKind::Docked => Modifier::new().fill_max_width(),
    };

    surface(&crate::surface::SurfaceArgs::with_child(
        SurfaceArgs::default()
            .style(container_color.into())
            .shape(shape)
            .modifier(modifier)
            .tonal_elevation(tonal_elevation)
            .block_input(true)
            .elevation(shadow_elevation),
        move || {
            let content = content.clone();
            column(ColumnArgs::default(), |scope| {
                scope.child(move || {
                    horizontal_divider(&DividerArgs::default().color(divider_color));
                });
                scope.child(move || {
                    let content = content.clone();
                    Modifier::new().padding_all(content_padding).run(move || {
                        content.render();
                    });
                });
            });
        },
    ));
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
