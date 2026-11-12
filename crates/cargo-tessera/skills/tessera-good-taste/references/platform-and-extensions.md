# Platform and extensions

- Register packages and platform services through `EntryPoint`. Use `ComponentsPackage` for the standard component stack and one shared startup path for desktop, web, Android, and iOS entrypoints. Use `run_ios` on the iOS main thread.
- Select an integrated desktop title bar with `WindowConfig::with_app_titlebar` and designate its app bar with `window_chrome(true)`.
- Use `shard_home` and `#[shard]` pages for routing. A `State<RouterController>` page parameter receives the router; descendants read the same `State<RouterController>` context. Each route provides a model store, so `Model<T>` parameters below a route live until it is popped.
- Push destinations that users can return from. On Android, `shard_home` previews the previous route during a system back swipe and pops on commit.
- For custom Android destinations and overlays, mount `system_back_handler` consistently and enable it while there is an action to perform. Preview progress without changing navigation state, restore on cancellation, and navigate on commit. The generated Android host connects the handler to the system dispatcher.
- Use the basic plugin scaffold for Rust integrations on every platform. Access UIKit through `PluginContext::ui_view` on the main thread.
- Access window and clipboard services through their documented plugin/session interfaces. Register services at application startup and tie external resources to an explicit owner.
- Choose public Modifier, Plugin, Package, RenderModule, and graphics extension contracts for custom integrations. Keep internal node and replay machinery within the framework.
- For custom graphics, follow the pipeline's stencil, sampling-region, coordinate, and resource-reuse contracts. Use declared sampling bounds and register pipelines through the render module.
