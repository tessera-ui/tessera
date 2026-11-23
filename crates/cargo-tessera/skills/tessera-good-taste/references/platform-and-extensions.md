# Platform and extensions

- Register packages and platform services through `EntryPoint`. Use `ComponentsPackage` for the standard component stack and one shared startup path for desktop, web, Android, and iOS entrypoints. Use `run_ios` on the iOS main thread.
- Select an integrated desktop title bar with `WindowConfig::with_app_titlebar` and designate its app bar with `window_chrome(true)`.
- Use `shard_home` and `#[shard]` pages for routing. Pages and their descendants get the router with `use_router()`. Each route provides a model store, so `Model<T>` parameters below a route live until it is popped.
- Push destinations that users can return from. On Android, `shard_home` previews the previous route during a system back swipe and pops on commit.
- For custom Android destinations and overlays, mount `system_back_handler` consistently and enable it while there is an action to perform. Preview progress without changing navigation state, restore on cancellation, and navigate on commit.
- Use the basic plugin scaffold for Rust integrations on every platform. Access UIKit through `PluginContext::ui_view` on the main thread.
- Embed Android or iOS controls with `tessera_ui::platform_view::platform_view`. Check its API docs before use: support is limited to iOS and Android API 30+, Tessera assigns the view size, and the first factory is retained for the component instance. Send changing data through the factory's update path and use `key` when a different item needs a fresh native view.
- Native views support translation and uniform scale, but not shaped ancestor clips, rotation, or shear. Draw effects cannot sample native pixels. On Android 11–14, fractional opacity is unsupported for independently composited `SurfaceView` pixels.
- Access window and clipboard services through their documented plugin/session interfaces. Register services at application startup and tie external resources to an explicit owner.
- Choose public Modifier, Plugin, Package, RenderModule, and graphics extension contracts for custom integrations.
- For custom graphics, follow the public drawing and sampling contracts and register pipelines through the render module.
