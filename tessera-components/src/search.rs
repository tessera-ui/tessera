//! Search bar component - capture queries with optional suggestion content.
//!
//! ## Usage
//!
//! Use to collect search queries and show suggestions or results as the user
//! types.
use tessera_ui::{
    CallbackWith, Color, Dp, Modifier, RenderSlot, State, layout::layout_primitive, remember,
    tessera, use_context, winit,
};

use crate::{
    column::column,
    divider::horizontal_divider,
    gesture_recognizer::TapRecognizer,
    modifier::{ModifierExt as _, with_keyboard_input, with_pointer_input},
    pos_misc::is_position_inside_bounds,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text_field::{TextFieldBuilder, TextFieldDefaults, TextFieldLineLimit},
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

impl Default for SearchBarColors {
    fn default() -> Self {
        SearchBarDefaults::colors()
    }
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
        Shape::CAPSULE
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

#[derive(Clone)]
struct SearchBarProps {
    modifier: Modifier,
    enabled: bool,
    read_only: bool,
    is_active: bool,
    placeholder: Option<String>,
    leading_icon: Option<RenderSlot>,
    trailing_icon: Option<RenderSlot>,
    on_query_change: Option<CallbackWith<String, String>>,
    on_search: Option<CallbackWith<String, ()>>,
    on_active_change: Option<CallbackWith<bool, ()>>,
    shape: Shape,
    colors: SearchBarColors,
    tonal_elevation: Dp,
    shadow_elevation: Dp,
    dropdown_shape: Shape,
    dropdown_gap: Dp,
    content_padding: Dp,
    controller: Option<State<SearchBarController>>,
    content: Option<RenderSlot>,
}

impl Default for SearchBarProps {
    fn default() -> Self {
        Self {
            modifier: Modifier::new(),
            enabled: true,
            read_only: false,
            is_active: false,
            placeholder: None,
            leading_icon: None,
            trailing_icon: None,
            on_query_change: None,
            on_search: None,
            on_active_change: None,
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

#[derive(Clone, PartialEq, Copy, Default)]
enum SearchBarLayoutKind {
    #[default]
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
/// - `modifier` - modifier chain applied to the search bar container.
/// - `enabled` - whether the input is enabled.
/// - `read_only` - whether the input is read-only.
/// - `is_active` - whether the search bar is currently expanded.
/// - `placeholder` - optional placeholder text.
/// - `leading_icon` - optional leading icon slot.
/// - `trailing_icon` - optional trailing icon slot.
/// - `on_query_change` - optional callback for query changes.
/// - `on_search` - optional callback invoked when the query is submitted.
/// - `on_active_change` - optional callback invoked when active state changes.
/// - `shape` - shape used for the search input.
/// - `colors` - search bar color configuration.
/// - `tonal_elevation` - tonal elevation for the search container.
/// - `shadow_elevation` - shadow elevation for the search container.
/// - `dropdown_shape` - shape used for docked result surfaces.
/// - `dropdown_gap` - gap between a docked field and its results.
/// - `content_padding` - padding inside the result surface.
/// - `controller` - optional external controller for active/query state.
/// - `content` - optional results content slot.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// # #[tessera]
/// # fn component() {
/// use tessera_components::search::search_bar;
/// material_theme()
///     .theme(|| MaterialTheme::default())
///     .child(|| {
///         search_bar().is_active(true).content(|| {});
///     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn search_bar(
    modifier: Modifier,
    #[default(true)] enabled: bool,
    read_only: bool,
    is_active: bool,
    #[prop(into)] placeholder: Option<String>,
    leading_icon: Option<RenderSlot>,
    trailing_icon: Option<RenderSlot>,
    on_query_change: Option<CallbackWith<String, String>>,
    on_search: Option<CallbackWith<String, ()>>,
    on_active_change: Option<CallbackWith<bool, ()>>,
    #[default(SearchBarProps::default().shape)] shape: Shape,
    #[default(SearchBarProps::default().colors)] colors: SearchBarColors,
    #[default(SearchBarProps::default().tonal_elevation)] tonal_elevation: Dp,
    #[default(SearchBarProps::default().shadow_elevation)] shadow_elevation: Dp,
    #[default(SearchBarProps::default().dropdown_shape)] dropdown_shape: Shape,
    #[default(SearchBarProps::default().dropdown_gap)] dropdown_gap: Dp,
    #[default(SearchBarProps::default().content_padding)] content_padding: Dp,
    controller: Option<State<SearchBarController>>,
    content: Option<RenderSlot>,
) {
    render_search_bar(
        SearchBarProps {
            modifier,
            enabled,
            read_only,
            is_active,
            placeholder,
            leading_icon,
            trailing_icon,
            on_query_change,
            on_search,
            on_active_change,
            shape,
            colors,
            tonal_elevation,
            shadow_elevation,
            dropdown_shape,
            dropdown_gap,
            content_padding,
            controller,
            content,
        },
        SearchBarLayoutKind::FullScreen,
    );
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
/// - Parameters are the same as [`search_bar`], but docked results render below
///   the field instead of taking over the screen.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// # #[tessera]
/// # fn component() {
/// use tessera_components::search::docked_search_bar;
/// material_theme()
///     .theme(|| MaterialTheme::default())
///     .child(|| {
///         docked_search_bar().is_active(true).content(|| {});
///     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn docked_search_bar(
    modifier: Modifier,
    #[default(true)] enabled: bool,
    read_only: bool,
    is_active: bool,
    #[prop(into)] placeholder: Option<String>,
    leading_icon: Option<RenderSlot>,
    trailing_icon: Option<RenderSlot>,
    on_query_change: Option<CallbackWith<String, String>>,
    on_search: Option<CallbackWith<String, ()>>,
    on_active_change: Option<CallbackWith<bool, ()>>,
    #[default(SearchBarProps::default().shape)] shape: Shape,
    #[default(SearchBarProps::default().colors)] colors: SearchBarColors,
    #[default(SearchBarProps::default().tonal_elevation)] tonal_elevation: Dp,
    #[default(SearchBarProps::default().shadow_elevation)] shadow_elevation: Dp,
    #[default(SearchBarProps::default().dropdown_shape)] dropdown_shape: Shape,
    #[default(SearchBarProps::default().dropdown_gap)] dropdown_gap: Dp,
    #[default(SearchBarProps::default().content_padding)] content_padding: Dp,
    controller: Option<State<SearchBarController>>,
    content: Option<RenderSlot>,
) {
    render_search_bar(
        SearchBarProps {
            modifier,
            enabled,
            read_only,
            is_active,
            placeholder,
            leading_icon,
            trailing_icon,
            on_query_change,
            on_search,
            on_active_change,
            shape,
            colors,
            tonal_elevation,
            shadow_elevation,
            dropdown_shape,
            dropdown_gap,
            content_padding,
            controller,
            content,
        },
        SearchBarLayoutKind::Docked,
    );
}

fn render_search_bar(args: SearchBarProps, kind: SearchBarLayoutKind) {
    let render_args = build_search_bar_props(args, kind);
    let modifier = render_args.modifier.clone();
    layout_primitive().modifier(modifier).child(move || {
        let mut builder = search_bar_inner()
            .kind(render_args.kind)
            .enabled(render_args.enabled)
            .read_only(render_args.read_only)
            .shape(render_args.shape)
            .colors(render_args.colors)
            .tonal_elevation(render_args.tonal_elevation)
            .shadow_elevation(render_args.shadow_elevation)
            .dropdown_shape(render_args.dropdown_shape)
            .dropdown_gap(render_args.dropdown_gap)
            .content_padding(render_args.content_padding)
            .controller(render_args.controller)
            .content_shared(render_args.content)
            .on_query_change_shared(render_args.on_query_change)
            .on_search_shared(render_args.on_search)
            .on_active_change_shared(render_args.on_active_change);
        if let Some(placeholder) = render_args.placeholder.clone() {
            builder = builder.placeholder(placeholder);
        }
        if let Some(leading_icon) = render_args.leading_icon {
            builder = builder.leading_icon_shared(leading_icon);
        }
        if let Some(trailing_icon) = render_args.trailing_icon {
            builder = builder.trailing_icon_shared(trailing_icon);
        }
        drop(builder);
    });
}

fn build_search_bar_props(args: SearchBarProps, kind: SearchBarLayoutKind) -> SearchBarRenderArgs {
    let content = args.content.unwrap_or_else(RenderSlot::empty);
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
        on_query_change: args.on_query_change.unwrap_or_else(CallbackWith::identity),
        on_search: args.on_search.unwrap_or_else(CallbackWith::default_value),
        on_active_change: args
            .on_active_change
            .unwrap_or_else(CallbackWith::default_value),
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

#[derive(Clone)]
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

#[derive(Clone)]
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
fn search_bar_inner(
    kind: SearchBarLayoutKind,
    enabled: bool,
    read_only: bool,
    #[prop(into)] placeholder: Option<String>,
    leading_icon: Option<RenderSlot>,
    trailing_icon: Option<RenderSlot>,
    on_query_change: Option<CallbackWith<String, String>>,
    on_search: Option<CallbackWith<String, ()>>,
    on_active_change: Option<CallbackWith<bool, ()>>,
    shape: Shape,
    colors: SearchBarColors,
    tonal_elevation: Dp,
    shadow_elevation: Dp,
    dropdown_shape: Shape,
    dropdown_gap: Dp,
    content_padding: Dp,
    controller: Option<State<SearchBarController>>,
    content: Option<RenderSlot>,
) {
    let controller = controller.expect("search_bar_inner requires controller");
    let content = content.unwrap_or_else(RenderSlot::empty);
    let on_query_change = on_query_change.unwrap_or_else(CallbackWith::identity);
    let on_search = on_search.unwrap_or_else(CallbackWith::default_value);
    let on_active_change = on_active_change.unwrap_or_else(CallbackWith::default_value);
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();

    let font_size = theme.typography.body_large.font_size;
    let line_height = theme.typography.body_large.line_height;
    let input_controller = remember(|| TextInputController::new(font_size, line_height));
    let synced_query = remember(String::new);

    sync_query(&controller, &input_controller, &synced_query);

    let tap_recognizer = remember(TapRecognizer::default);
    let modifier = with_keyboard_input(
        with_pointer_input(Modifier::new(), {
            move |input| {
                if !enabled {
                    return;
                }
                let is_active = controller.with(|c| c.is_active());
                let cursor_pos = input.cursor_position_rel;
                let within_bounds = cursor_pos
                    .map(|pos| is_position_inside_bounds(input.computed_data, pos))
                    .unwrap_or(false);
                let tap_result = tap_recognizer.with_mut(|recognizer| {
                    recognizer.update(
                        input.pass,
                        input.pointer_changes.as_mut_slice(),
                        input.cursor_position_rel,
                        within_bounds,
                    )
                });

                if tap_result.pressed && within_bounds && !is_active {
                    controller.with_mut(|c| c.open());
                    on_active_change.call(true);
                }
            }
        }),
        move |mut input| {
            if !enabled {
                return;
            }
            let is_active = controller.with(|c| c.is_active());
            for event in input.keyboard_events.iter() {
                if event.state == winit::event::ElementState::Pressed
                    && let winit::keyboard::PhysicalKey::Code(code) = event.physical_key
                {
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
            if is_active {
                input.block_keyboard();
            }
        },
    );

    let results_surface_args = SearchResultsSurfaceArgs {
        kind,
        container_color: colors.container_color,
        divider_color: colors.divider_color,
        dropdown_shape,
        tonal_elevation,
        shadow_elevation,
        content_padding,
        content,
    };

    layout_primitive().modifier(modifier).child(move || {
        let placeholder = placeholder.clone();
        let leading_icon = leading_icon;
        let trailing_icon = trailing_icon;
        let results_surface_args = results_surface_args.clone();
        column().children(move || {
            build_search_field(SearchFieldArgs {
                enabled,
                read_only,
                placeholder: placeholder.clone(),
                leading_icon,
                trailing_icon,
                shape,
                colors,
                controller,
                input_controller,
                synced_query,
                on_query_change,
            });

            if controller.with(|c| c.is_active()) {
                if matches!(kind, SearchBarLayoutKind::Docked) {
                    spacer().modifier(Modifier::new().height(dropdown_gap));
                }

                render_results_surface(&results_surface_args);
            }
        });
    });
}

fn render_results_surface(args: &SearchResultsSurfaceArgs) {
    let shape = match args.kind {
        SearchBarLayoutKind::FullScreen => Shape::RECTANGLE,
        SearchBarLayoutKind::Docked => args.dropdown_shape,
    };

    let modifier = match args.kind {
        SearchBarLayoutKind::FullScreen => Modifier::new().fill_max_width().fill_max_height(),
        SearchBarLayoutKind::Docked => Modifier::new().fill_max_width(),
    };

    let content = args.content;
    let divider_color = args.divider_color;
    let content_padding = args.content_padding;

    surface()
        .style(args.container_color.into())
        .shape(shape)
        .modifier(modifier)
        .tonal_elevation(args.tonal_elevation)
        .block_input(true)
        .elevation(args.shadow_elevation)
        .with_child(move || {
            let content = content;
            column().children(move || {
                horizontal_divider().color(divider_color);
                let content = content;
                layout_primitive()
                    .modifier(Modifier::new().padding_all(content_padding))
                    .child(move || {
                        content.render();
                    });
            });
        });
}

struct SearchFieldArgs {
    enabled: bool,
    read_only: bool,
    placeholder: Option<String>,
    leading_icon: Option<RenderSlot>,
    trailing_icon: Option<RenderSlot>,
    shape: Shape,
    colors: SearchBarColors,
    controller: State<SearchBarController>,
    input_controller: State<TextInputController>,
    synced_query: State<String>,
    on_query_change: CallbackWith<String, String>,
}

fn build_search_field(args: SearchFieldArgs) {
    let mut builder = TextFieldBuilder::filled()
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
        .line_limit(TextFieldLineLimit::SingleLine)
        .on_change(move |text| {
            let next = args.on_query_change.call(text);
            args.controller.with_mut(|c| c.set_query(next.clone()));
            args.synced_query.set(next.clone());
            next
        })
        .controller(args.input_controller);
    if let Some(placeholder) = args.placeholder {
        builder = builder.placeholder(placeholder);
    }
    if let Some(leading_icon) = args.leading_icon {
        builder = builder.leading_icon(move || {
            leading_icon.render();
        });
    }
    if let Some(trailing_icon) = args.trailing_icon {
        builder = builder.trailing_icon(move || {
            trailing_icon.render();
        });
    }
    drop(builder);
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
