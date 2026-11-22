//! Navigation stacks that title the current page and go back to earlier ones.
//!
//! ## Usage
//!
//! Drill into settings, articles, or details and return along the same path.

use tessera_ui::{
    Callback, KeyboardInput, KeyboardInputModifierNode, Modifier, RenderSlot,
    layout::{Stack, layout},
    modifier::ModifierCapabilityExt as _,
    tessera,
    winit::{event::ElementState, keyboard::Key},
};

use crate::column::column;

use super::toolbar::{ToolbarSymbol, toolbar, toolbar_button};

#[cfg(test)]
#[path = "navigation_stack_tests.rs"]
mod tests;

/// Goes back on Command-[.
struct BackShortcut(Callback);

impl KeyboardInputModifierNode for BackShortcut {
    fn on_keyboard_input(&self, input: KeyboardInput<'_>) {
        if !input.key_modifiers.super_key() {
            return;
        }
        let mut back = false;
        input.keyboard_events.retain(|event| {
            if event.logical_key == Key::Character("[".into()) {
                back |= event.state == ElementState::Pressed;
                return false;
            }
            true
        });
        if back {
            self.0.call();
        }
    }
}

/// # navigation_stack
///
/// Titles the current page of a stack of pages and offers a back button to the previous one.
///
/// ## Usage
///
/// Drive a window toolbar from a router's history while people drill into details.
///
/// ## Parameters
///
/// - `root_title` — title of the first page.
/// - `path` — titles of the pages pushed on top of the first, in order; the last is current.
/// - `on_back` — requests returning to the previous page; called by the back button and by
///   Command-[ while focus is in `content`.
/// - `leading` — toolbar items shown on the first page in place of the back button.
/// - `actions` — trailing toolbar items.
/// - `back_label` — spoken name of the back button; defaults to "Back".
/// - `window_chrome` — reserves room for the window controls; defaults to false.
/// - `content` — the current page below the toolbar; omit it to place the toolbar alone, such
///   as in a window's top bar.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{navigation_stack::navigation_stack, text::text};
/// use tessera_ui::{remember, tessera};
/// # #[tessera]
/// # fn component() {
///
/// let path = remember(Vec::<String>::new);
/// navigation_stack("Settings")
///     .path(path.get())
///     .on_back(move || path.with_mut(|path| {
///         path.pop();
///     }))
///     .content(move || {
///         text(path.get().last().cloned().unwrap_or_else(|| "General".into()));
///     });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn navigation_stack(
    #[prop(into)] root_title: String,
    path: Option<Vec<String>>,
    on_back: Option<Callback>,
    leading: Option<RenderSlot>,
    actions: Option<RenderSlot>,
    #[prop(into)] back_label: Option<String>,
    window_chrome: Option<bool>,
    content: Option<RenderSlot>,
) {
    let path = path.unwrap_or_default();
    let can_go_back = !path.is_empty();
    let title = path.last().cloned().unwrap_or(root_title);
    let back = on_back.unwrap_or_else(Callback::noop);
    let back_label = back_label.unwrap_or_else(|| "Back".to_owned());
    let shortcut = Callback::new(move || {
        if can_go_back {
            back.call();
        }
    });
    let bar = RenderSlot::new(move || {
        let bar = toolbar()
            .title(title.clone())
            .window_chrome(window_chrome.unwrap_or(false));
        if can_go_back {
            let back_label = back_label.clone();
            bar.navigation(move || {
                toolbar_button()
                    .symbol(ToolbarSymbol::Back)
                    .accessibility_label(back_label.clone())
                    .on_click_shared(back);
            });
        } else if let Some(leading) = leading {
            bar.navigation_shared(leading);
        }
        if let Some(actions) = actions {
            bar.actions_shared(actions);
        }
    });
    match content {
        Some(content) => {
            column().children(move || {
                bar.render();
                stack_page(shortcut, content);
            });
        }
        None => bar.render(),
    }
}

#[tessera]
fn stack_page(shortcut: Callback, content: RenderSlot) {
    layout(Stack)
        .modifier(Modifier::new().push_keyboard_input(BackShortcut(shortcut)))
        .child(content);
}
