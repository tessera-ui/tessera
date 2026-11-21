//! Menus of commands presented from a button.
//!
//! ## Usage
//!
//! Offer commands, options, and nested actions without taking space in the window.

use tessera_foundation::modifier::{
    ClickableArgs, InteractionState, ModifierExt as _, SemanticsArgs,
};
use tessera_ui::{
    Callback, Color, ComputedData, Constraint, CursorEventContent, Dp, DrawModifierContent,
    DrawModifierContext, DrawModifierNode, FloatRect, FocusRequester, FocusScopeNode,
    FocusTraversalPolicy, KeyboardInput, KeyboardInputModifierNode, LayoutPolicy, LayoutResult,
    MeasurementError, Modifier, PointerEventPass, PointerInput, PointerInputModifierNode,
    PressKeyEventType, Px, PxPosition, RenderSlot, State,
    accesskit::{Action, Role, Toggled},
    focus,
    layout::{MeasureScope, Stack, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    provide_context, remember, tessera, use_context,
    winit::{
        event::ElementState,
        keyboard::{Key, NamedKey},
    },
};

use crate::{
    alignment::CrossAxisAlignment,
    pipelines::{
        perceptual_fill::{PerceptualFillCommand, PerceptualStroke},
        shape::command::ShapeCommand,
    },
    row::row,
    shape_def::ResolvedShape,
    spacer::spacer,
};

use super::{
    button::{ButtonIndicator, ButtonStyle, push_button},
    presentation::{Popup, PopupLease, PopupPlacement, Presentations, next_popup_key},
    text::text,
    theme::{Appearance, ControlSize, HigTheme, TextStyle},
};

#[cfg(test)]
#[path = "menu_tests.rs"]
mod tests;

/// Height of one menu item.
pub(super) const ITEM_HEIGHT: f32 = 22.0;
/// Space above the first and below the last item.
pub(super) const PANEL_INSET: f32 = 5.0;
const PANEL_RADIUS: f64 = 10.0;

/// The appearance of a [`menu`]'s button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuStyle {
    /// The platform default, a bordered pull-down button on macOS.
    #[default]
    Automatic,
    /// A push button that opens the menu.
    Button,
    /// A bordered button with a pull-down indicator.
    BorderedButton,
    /// A title with a pull-down indicator and no bezel.
    BorderlessButton,
}

/// The menu presenting the current items, for dismissal after activation.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct MenuContext {
    pub dismiss: Callback,
}

/// # menu
///
/// Shows a button that pulls down a menu of commands.
///
/// ## Usage
///
/// Group related commands, such as sharing or sorting options, behind one button.
///
/// ## Parameters
///
/// - `label` — button title and default accessible name.
/// - `content` — the menu's items, sections, and dividers.
/// - `style` — bordered or borderless pull-down button; defaults to [`MenuStyle::Automatic`].
/// - `control_size` — button size; defaults to regular.
/// - `enabled` — input and focus availability; defaults to true.
/// - `modifier` — layout and additional semantics.
/// - `accessibility_label`, `accessibility_description` — spoken content overrides.
///
/// The menu appears inside the nearest
/// [`presentation_host`](super::presentation::presentation_host).
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     menu::{menu, menu_divider, menu_item},
///     presentation::presentation_host,
/// };
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// presentation_host().child(|| {
///     menu("Sort").content(|| {
///         menu_item("By Name").checked(true).on_click(|| println!("name"));
///         menu_item("By Date").checked(false).on_click(|| println!("date"));
///         menu_divider();
///         menu_item("Reverse Order").on_click(|| println!("reverse"));
///     });
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn menu(
    #[prop(into)] label: String,
    content: Option<RenderSlot>,
    style: Option<MenuStyle>,
    control_size: Option<ControlSize>,
    enabled: Option<bool>,
    modifier: Option<Modifier>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let enabled = enabled.unwrap_or(true);
    let button_style = match style.unwrap_or_default() {
        MenuStyle::BorderlessButton => ButtonStyle::Borderless,
        _ => ButtonStyle::Bordered,
    };
    let indicator = (style != Some(MenuStyle::Button)).then_some(ButtonIndicator::PullDown);
    let open = remember(|| false);
    let anchor = remember(|| FloatRect::ZERO);
    let requester = remember(FocusRequester::new).get();
    let scope = remember(FocusScopeNode::new).get();
    let dismiss = Callback::new(move || {
        if open.get() {
            open.set(false);
            requester.request_focus();
        }
    });
    if !enabled && open.get() {
        open.set(false);
    }
    present(
        open.get() && enabled,
        anchor.get(),
        PopupPlacement::Below,
        match content {
            Some(slot) => slot,
            None => RenderSlot::new(|| {}),
        },
        dismiss,
        scope,
        anchor.get().width,
    );
    push_button(label)
        .style(button_style)
        .indicator_optional(indicator)
        .expanded(open.get())
        .control_size_optional(control_size)
        .enabled(enabled)
        .focus_requester(requester)
        .on_click(move || open.set(!open.get()))
        .accessibility_label_optional(accessibility_label)
        .accessibility_description_optional(accessibility_description)
        .modifier(
            modifier
                .unwrap_or_default()
                .on_globally_positioned_precise(move |rect| {
                    if anchor.get() != rect {
                        anchor.set(rect);
                    }
                }),
        );
}

/// Shows `content` in a menu panel while `open`, anchored to `anchor` in window coordinates.
#[tessera]
pub(super) fn present(
    open: bool,
    anchor: FloatRect,
    placement: PopupPlacement,
    content: RenderSlot,
    dismiss: Callback,
    scope: FocusScopeNode,
    min_width: f32,
) {
    let presentations = use_context::<Presentations>().map(|context| context.get());
    let theme = use_context::<HigTheme>()
        .map(|theme| theme.get())
        .unwrap_or_default();
    let key = remember(next_popup_key).get();
    let Some(presentations) = presentations else {
        return;
    };
    remember(|| PopupLease { presentations, key });
    if open {
        let min_width = Dp::from_pixels_f32(min_width);
        presentations.show(
            key,
            Popup {
                anchor,
                placement,
                content: RenderSlot::new(move || menu_panel(content, dismiss, scope, min_width)),
                dismiss,
                theme,
                modal: false,
            },
        );
    } else {
        presentations.hide(key);
    }
}

struct MenuKeys {
    dismiss: Callback,
}

impl KeyboardInputModifierNode for MenuKeys {
    fn on_keyboard_input(&self, input: KeyboardInput<'_>) {
        let mut dismiss = false;
        input.keyboard_events.retain(|event| {
            if event.logical_key == Key::Named(NamedKey::Escape) {
                dismiss |= event.state == ElementState::Pressed;
                return false;
            }
            true
        });
        if dismiss {
            self.dismiss.call();
        }
    }
}

/// Lays items out in a column as wide as the widest item, and at least `min_width`.
#[derive(Clone, Copy, PartialEq)]
struct MenuColumn {
    min_width: Dp,
}

impl LayoutPolicy for MenuColumn {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let children = scope.children();
        let mut width = Px::from(self.min_width);
        for child in &children {
            width = width.max(child.probe_intrinsic(scope, &Constraint::NONE)?.width);
        }
        let inset = Px::from(Dp(f64::from(PANEL_INSET)));
        let mut y = inset;
        let mut placed = Vec::with_capacity(children.len());
        for child in &children {
            let size = child
                .measure(
                    scope,
                    &Constraint::new(
                        tessera_ui::AxisConstraint::exact(width),
                        tessera_ui::AxisConstraint::new(Px::ZERO, None),
                    ),
                )?
                .size();
            placed.push((*child, y));
            y += size.height;
        }
        let mut result = LayoutResult::new(ComputedData {
            width,
            height: y + inset,
        });
        for (child, y) in placed {
            result.place_child(child, PxPosition::new(Px::ZERO, y));
        }
        Ok(result)
    }
}

#[derive(Clone, PartialEq)]
struct PanelBackground {
    color: Color,
    border: Color,
}

impl DrawModifierNode for PanelBackground {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        let mut metadata = context.render_input.metadata_mut();
        let fragment = metadata.fragment_mut();
        fragment.push_draw_command(ShapeCommand::Shadow {
            color: Color::BLACK.with_alpha(0.2),
            shape: ResolvedShape::Rounded {
                corner_radii: [Dp(PANEL_RADIUS).to_pixels_f32(); 4],
                corner_g2: [2.0; 4],
            },
            smoothness: Dp(20.0).to_pixels_f32(),
        });
        fragment.push_draw_command(fill(self.color, Dp(PANEL_RADIUS), [Dp::ZERO; 4]));
        fragment.push_draw_command(PerceptualFillCommand {
            outline_width: Dp(0.5),
            ..fill(self.border, Dp(PANEL_RADIUS), [Dp::ZERO; 4])
        });
        content.draw(context.render_input);
    }
}

fn fill(color: Color, radius: Dp, insets: [Dp; 4]) -> PerceptualFillCommand {
    PerceptualFillCommand {
        outline_width: Dp::ZERO,
        snap_to_pixels: false,
        path: None,
        color,
        radii: [radius; 2],
        insets,
        clip_radius: None,
        line: None,
        edge_smoothness: 1.0,
        stroke: None,
    }
}

#[tessera]
fn menu_panel(content: RenderSlot, dismiss: Callback, scope: FocusScopeNode, min_width: Dp) {
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let entered = remember(|| false);
    if !entered.get() {
        entered.set(true);
        focus::defer_keyboard_until_layout();
        scope.request_focus();
    }
    let background = if dark {
        Color::from_rgb(0.03, 0.03, 0.03).with_alpha(0.96)
    } else {
        Color::from_rgb(0.92, 0.92, 0.92).with_alpha(0.96)
    };
    let modifier = Modifier::new()
        .focus_scope(Some(scope))
        .focus_group(None)
        .focus_traversal_policy(FocusTraversalPolicy::linear().wrap(true))
        .push_keyboard_input(MenuKeys { dismiss })
        .push_draw(PanelBackground {
            color: background,
            border: if dark { Color::WHITE } else { Color::BLACK }.with_alpha(0.12),
        })
        .semantics(SemanticsArgs {
            role: Some(Role::Menu),
            ..Default::default()
        });
    provide_context(MenuContext { dismiss }, move || {
        layout(MenuColumn { min_width })
            .modifier(modifier.clone())
            .child(content);
    });
}

#[derive(Clone, PartialEq)]
struct ItemBackground(Color);

impl DrawModifierNode for ItemBackground {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        if self.0.a > 0.0 {
            context
                .render_input
                .metadata_mut()
                .fragment_mut()
                .push_draw_command(fill(
                    self.0,
                    Dp(5.0),
                    [Dp::ZERO, Dp(5.0), Dp::ZERO, Dp(5.0)],
                ));
        }
        content.draw(context.render_input);
    }
}

#[derive(Clone, PartialEq)]
struct CheckGlyph(Color);

impl DrawModifierNode for CheckGlyph {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(PerceptualFillCommand {
                stroke: Some(PerceptualStroke {
                    raster_mask: false,
                    color: self.0,
                    width: Dp(1.6),
                    points: [[0.15, 0.55], [0.4, 0.8], [0.85, 0.2]],
                }),
                ..fill(Color::TRANSPARENT, Dp::ZERO, [Dp::ZERO; 4])
            });
        content.draw(context.render_input);
    }
}

#[tessera]
fn check_glyph(color: Color) {
    layout(Stack).modifier(
        Modifier::new()
            .size(Dp(10.0), Dp(10.0))
            .push_draw(CheckGlyph(color)),
    );
}

#[tessera]
fn item_text(content: String, color: Color, size: f64) {
    text(content)
        .style(TextStyle {
            font_size: Dp(size),
            line_height: Dp(16.0),
            ..Default::default()
        })
        .color(color)
        .single_line(true)
        .modifier(Modifier::new().clear_and_set_semantics(SemanticsArgs {
            hidden: true,
            ..Default::default()
        }));
}

/// # menu_item
///
/// Runs a command from a menu and closes it.
///
/// ## Usage
///
/// Add commands, with optional checkmarks and keyboard shortcut hints, to a menu.
///
/// ## Parameters
///
/// - `title` — visible title and default accessible name.
/// - `on_click` — command run when the item is chosen.
/// - `checked` — shows a checkmark when true; makes the item a checkable item.
/// - `shortcut` — keyboard shortcut text shown at the trailing edge, such as "⌘N".
/// - `enabled` — whether the item can be chosen; defaults to true.
/// - `accessibility_label`, `accessibility_description` — spoken content overrides.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     menu::{menu, menu_item},
///     presentation::presentation_host,
/// };
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// presentation_host().child(|| {
///     menu("File").content(|| {
///         menu_item("New Window").shortcut("⌘N").on_click(|| println!("new"));
///         menu_item("Close").enabled(false);
///     });
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn menu_item(
    #[prop(into)] title: String,
    on_click: Option<Callback>,
    checked: Option<bool>,
    #[prop(into)] shortcut: Option<String>,
    enabled: Option<bool>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let enabled = enabled.unwrap_or(true);
    let context = use_context::<MenuContext>().map(|context| context.get());
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let requester = remember(FocusRequester::new).get();
    let interaction = remember(InteractionState::new);
    let hovered = interaction.with(InteractionState::is_hovered);
    if enabled && hovered && !requester.is_focused() {
        requester.request_focus();
    }
    // Keyboard selection shows once the keyboard is used; the pointer selects by hovering.
    let selected = enabled && (hovered || requester.is_focus_visible());
    let action = on_click.unwrap_or_else(Callback::noop);
    let activate = Callback::new(move || {
        if let Some(context) = context {
            context.dismiss.call();
        }
        action.call();
    });
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    let ink = if selected {
        Color::WHITE
    } else if enabled {
        tint.with_alpha(0.85)
    } else {
        tint.with_alpha(0.25)
    };
    let secondary = if selected {
        Color::WHITE.with_alpha(0.75)
    } else {
        tint.with_alpha(if enabled { 0.5 } else { 0.25 })
    };
    let highlight = if selected {
        Color::from_rgb(0.0, 0.19461783, 1.0)
    } else {
        Color::TRANSPARENT
    };
    let role = if checked.is_some() {
        Role::MenuItemCheckBox
    } else {
        Role::MenuItem
    };
    let modifier = Modifier::new()
        .clickable(ClickableArgs {
            on_click: activate,
            enabled,
            role: Some(role),
            label: Some(accessibility_label.unwrap_or_else(|| title.clone())),
            description: accessibility_description,
            interaction_state: Some(interaction),
            focus_requester: Some(requester),
            ..Default::default()
        })
        .push_semantics(ItemSemantics(checked))
        .height(Dp(f64::from(ITEM_HEIGHT)))
        .push_draw(ItemBackground(highlight));
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(modifier)
        .children(move || {
            spacer().modifier(Modifier::new().width(Dp(9.0)));
            if checked == Some(true) {
                check_glyph(ink);
            } else {
                spacer().modifier(Modifier::new().width(Dp(10.0)));
            }
            spacer().modifier(Modifier::new().width(Dp(5.0)));
            item_text(title.clone(), ink, 13.0);
            if let Some(shortcut) = shortcut.clone() {
                spacer().modifier(Modifier::new().width(Dp(24.0)));
                spacer().modifier(Modifier::new().weight(1.0));
                item_text(shortcut, secondary, 13.0);
            }
            spacer().modifier(Modifier::new().width(Dp(14.0)));
        });
}

struct ItemSemantics(Option<bool>);

impl tessera_ui::SemanticsModifierNode for ItemSemantics {
    fn apply(
        &self,
        node: &mut tessera_ui::AccessibilityNode,
        _: &mut Option<tessera_ui::AccessibilityActionHandler>,
    ) {
        if let Some(checked) = self.0 {
            node.toggled = Some(if checked {
                Toggled::True
            } else {
                Toggled::False
            });
        }
    }
}

/// # menu_divider
///
/// Separates groups of related items in a menu.
///
/// ## Usage
///
/// Divide a menu's commands into groups, such as editing and sharing commands.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     menu::{menu, menu_divider, menu_item},
///     presentation::presentation_host,
/// };
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// presentation_host().child(|| {
///     menu("Edit").content(|| {
///         menu_item("Copy");
///         menu_divider();
///         menu_item("Share");
///     });
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn menu_divider() {
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    layout(Stack).modifier(
        Modifier::new()
            .height(Dp(11.0))
            .push_draw(ItemBackground(Color::TRANSPARENT))
            .push_draw(DividerLine(tint.with_alpha(0.1))),
    );
}

#[derive(Clone, PartialEq)]
struct DividerLine(Color);

impl DrawModifierNode for DividerLine {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(fill(
                self.0,
                Dp::ZERO,
                [Dp(5.0), Dp(14.0), Dp(5.0), Dp(14.0)],
            ));
        content.draw(context.render_input);
    }
}

/// # menu_section
///
/// Groups related menu items under an optional header.
///
/// ## Usage
///
/// Label a group of related commands, such as the sort orders in a view menu.
///
/// ## Parameters
///
/// - `header` — optional header text above the items.
/// - `content` — the section's items.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     menu::{menu, menu_item, menu_section},
///     presentation::presentation_host,
/// };
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// presentation_host().child(|| {
///     menu("View").content(|| {
///         menu_section().header("Sort By").content(|| {
///             menu_item("Name").checked(true);
///             menu_item("Date").checked(false);
///         });
///     });
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn menu_section(#[prop(into)] header: Option<String>, content: Option<RenderSlot>) {
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    if let Some(header) = header {
        section_header(header, tint.with_alpha(0.5));
    }
    if let Some(content) = content {
        content.render();
    }
}

#[tessera]
fn section_header(header: String, color: Color) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .height(Dp(f64::from(ITEM_HEIGHT)))
                .semantics(SemanticsArgs {
                    role: Some(Role::Heading),
                    label: Some(header.clone()),
                    ..Default::default()
                }),
        )
        .children(move || {
            spacer().modifier(Modifier::new().width(Dp(14.0)));
            item_text(header.clone(), color, 11.0);
        });
}

struct ContextPress {
    open: State<bool>,
    point: State<FloatRect>,
    enabled: bool,
}

impl PointerInputModifierNode for ContextPress {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if !self.enabled || input.pass != PointerEventPass::Main {
            return;
        }
        let position = input.cursor_position_abs();
        for change in input.pointer_changes.iter_mut() {
            if change.is_consumed()
                || !matches!(
                    change.content,
                    CursorEventContent::Pressed(PressKeyEventType::Right)
                )
            {
                continue;
            }
            change.consume();
            if let Some(position) = position {
                self.point.set(FloatRect {
                    x: position.x,
                    y: position.y,
                    width: 0.0,
                    height: 0.0,
                });
                self.open.set(true);
            }
        }
    }
}

struct ContextSemantics {
    open: State<bool>,
    point: State<FloatRect>,
    bounds: State<FloatRect>,
}

impl tessera_ui::SemanticsModifierNode for ContextSemantics {
    fn apply(
        &self,
        node: &mut tessera_ui::AccessibilityNode,
        handler: &mut Option<tessera_ui::AccessibilityActionHandler>,
    ) {
        node.actions.push(Action::ShowContextMenu);
        let previous = handler.take();
        let (open, point, bounds) = (self.open, self.point, self.bounds);
        *handler = Some(Box::new(move |action| {
            if action == Action::ShowContextMenu {
                let origin = bounds.get();
                point.set(FloatRect {
                    width: 0.0,
                    height: 0.0,
                    ..origin
                });
                open.set(true);
            } else if let Some(previous) = &previous {
                previous(action);
            }
        }));
    }
}

/// # context_menu
///
/// Shows a menu of commands for its content when the content is secondary-clicked.
///
/// ## Usage
///
/// Offer commands for a list row, file, or other item without showing buttons for them.
///
/// ## Parameters
///
/// - `child` — the content the menu acts on.
/// - `menu` — the menu's items, sections, and dividers.
/// - `enabled` — whether secondary clicks open the menu; defaults to true.
///
/// The menu appears inside the nearest
/// [`presentation_host`](super::presentation::presentation_host).
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     menu::{context_menu, menu_item},
///     presentation::presentation_host,
///     text::text,
/// };
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// presentation_host().child(|| {
///     context_menu()
///         .child(|| {
///             text("Report.pdf");
///         })
///         .menu(|| {
///             menu_item("Open").on_click(|| println!("open"));
///             menu_item("Move to Trash").on_click(|| println!("trash"));
///         });
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn context_menu(child: Option<RenderSlot>, menu: Option<RenderSlot>, enabled: Option<bool>) {
    let enabled = enabled.unwrap_or(true);
    let open = remember(|| false);
    let point = remember(|| FloatRect::ZERO);
    let bounds = remember(|| FloatRect::ZERO);
    let scope = remember(FocusScopeNode::new).get();
    let dismiss = Callback::new(move || {
        if open.get() {
            open.set(false);
        }
    });
    if !enabled && open.get() {
        open.set(false);
    }
    present(
        open.get() && enabled,
        point.get(),
        PopupPlacement::At,
        match menu {
            Some(slot) => slot,
            None => RenderSlot::new(|| {}),
        },
        dismiss,
        scope,
        0.0,
    );
    let mut modifier = Modifier::new()
        .push_pointer_input(ContextPress {
            open,
            point,
            enabled,
        })
        .on_globally_positioned_precise(move |rect| {
            if bounds.get() != rect {
                bounds.set(rect);
            }
        });
    if enabled {
        modifier = modifier.push_semantics(ContextSemantics {
            open,
            point,
            bounds,
        });
    }
    layout(Stack).modifier(modifier).child(move || {
        if let Some(child) = child {
            child.render();
        }
    });
}
