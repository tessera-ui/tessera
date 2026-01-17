// Prevents additional console window on Windows in release, DO NOT REMOVE
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

use app::app;

tessera_ui::entry!(
    app,
    modules = [tessera_components::TesseraComponents::default()],
    plugins = []
);
