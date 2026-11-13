//! Material Design 3 components, theme, typography, and interaction feedback.

pub mod app_bar;
pub mod badge;
pub mod bottom_sheet;
pub mod button;
pub mod button_groups;
pub mod card;
pub mod carousel;
pub mod checkbox;
mod checkmark;
pub mod chip;
pub mod date_picker;
pub mod dialog;
pub mod divider;
pub mod extended_floating_action_button;
pub mod floating_action_button;
pub mod floating_action_button_menu;
pub mod icon;
pub mod icon_button;
pub mod lazy_grid;
pub mod lazy_list;
pub mod lazy_staggered_grid;
pub mod list_item;
pub mod loading_indicator;
pub mod material_icons;
pub mod menus;
pub mod modifier;
pub mod navigation_bar;
pub mod navigation_drawer;
#[cfg(all(test, not(target_family = "wasm")))]
mod navigation_focus_tests;
pub mod navigation_rail;
pub mod progress;
pub mod pull_refresh;
pub mod radio_button;
pub mod ripple_state;
pub mod scaffold;
pub mod scrollable;
pub mod search;
pub mod segmented_buttons;
pub mod shadow;
pub mod side_sheet;
pub mod slider;
pub mod snackbar;
pub mod split_buttons;
pub mod surface;
pub mod switch;
pub mod tabs;
pub mod text;
mod text_edit_core;
pub mod text_field;
pub mod text_input;
pub mod text_table;
pub mod theme;
pub mod time_picker;
pub mod toolbar;
pub mod tooltip;
pub mod wavy_progress;

#[cfg(test)]
mod overlay_motion_tests;

#[cfg(test)]
mod sheet_drag_tests;

#[cfg(all(test, not(target_family = "wasm")))]
mod scaled_controls_tests;

#[cfg(all(test, not(target_family = "wasm")))]
mod overlay_precision_tests;
