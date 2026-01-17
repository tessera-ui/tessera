// Prevents additional console window on Windows in release, DO NOT REMOVE
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod example_components;

use crate::app::app;

tessera_ui::entry!(app, packages = [tessera_components::ComponentsPackage],);
