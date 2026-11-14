# Public API map

Read the relevant public module documentation for signatures, defaults, feature
gates, and platform support. Use `cargo doc --open` in the application workspace.

| Task | Public API entry points |
| --- | --- |
| Startup and configuration | `tessera_ui::EntryPoint`, `RuntimeSession`, `WeakRuntimeSession`, `EntryRegistry`, `TesseraPackage`, `renderer::TesseraConfig`; `tessera_components::ComponentsPackage` |
| Composition and configuration | `tessera_ui::tessera`, generated component handles, `Callback`, `CallbackWith`, `RenderSlot`, `RenderSlotWith`; component module entrypoints |
| State and inherited visual defaults | `tessera_ui::{State, Model, ModelInit, ModelStore, ModelStoreScope, remember, remember_with_key, retain, retain_with_key, key, Context, provide_context, use_context}` |
| Layout and geometry | `tessera_components::{row, column, boxed, flow_row, flow_column, modifier}`; `tessera_ui::{Dp, Px, AxisConstraint, Constraint, geometry}`; `tessera_foundation::{modifier, alignment, shape_def}` |
| Controls and virtualization | Material control modules and their controller types in `tessera_components::md3`, including `lazy_list`, `lazy_grid`, `lazy_staggered_grid`, and `scrollable`; `tessera_components::pager`; experimental `tessera_components::experiment::hig::glass` |
| Visuals and assets | `tessera_components::{painter, image, image_vector}`; `tessera_components::md3::{theme, icon, material_icons, shadow}`; `tessera_ui::{color, asset}`; `tessera_build` asset generation |
| Input, focus, and semantics | `tessera_ui::{PointerInput, KeyboardInput, ImeInput, ClipboardInput, focus, accessibility}`; `tessera_foundation::modifier` input and semantics extensions; `tessera_components::modifier` clickable, toggleable, and selectable extensions |
| Text editing | `tessera_components::md3::{text, text_input, text_field}`; editor controllers and documented transforms |
| Localization | `tessera_i18n::{Catalog, I18n, FluentValue, LanguageIdentifier, langid}`; generated `res::{I18N, i18n}` |
| Overlays and navigation | `tessera_components::md3::{dialog, menus, tooltip, bottom_sheet, side_sheet, snackbar, navigation_bar, navigation_rail, navigation_drawer}`; `tessera_shard::{shard, router}` |
| Tasks and animation | `tessera_ui::{launch, launch_once, spawn, spawn_local, on, Task, Dispatcher, with_frame_time}` |
| Platform integration | `tessera_platform::{PlatformPackage, clipboard, window}`; `tessera_ui::{Plugin, PluginContext, with_plugin}` |
| Public extensions | `tessera_ui::{modifier, Plugin}`; `tessera_ui::render::{RenderModule, PipelineContext, DrawCommand, DrawablePipeline, ComputeCommand, ComputablePipeline}`; documented sampling, clipping, and resource contracts |
| Build and verification | `cargo tessera` commands; `tessera_ui::{testing, headless, profiler}` with their feature gates |
