# Platform and extensions

- Register packages and platform services through `EntryPoint`. Use `ComponentsPackage` for the standard component stack and one shared startup path for desktop, web, and Android entrypoints.
- Use `shard_home` and `RouterController` for routing; select `ShardStateLifeCycle` to match route-instance or router-scope ownership.
- Access window and clipboard services through their documented plugin/session interfaces. Register services at application startup and tie external resources to an explicit owner.
- Choose public Modifier, Plugin, Package, RenderModule, and graphics extension contracts for custom integrations. Keep internal node and replay machinery within the framework.
- For custom graphics, follow the pipeline's stencil, sampling-region, coordinate, and resource-reuse contracts. Use declared sampling bounds and register pipelines through the render module.
